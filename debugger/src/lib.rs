use std::collections::HashMap;
use std::io::{self, Write};
use std::time::Duration;

use common::code::{self as code, Code};
use common::debug_info::DebugInfo;
use common::error::{NovaError, NovaResult};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    style::{Attribute, Color, Print, SetAttribute, SetForegroundColor},
    terminal::{self, ClearType},
    ExecutableCommand, QueueableCommand,
};

use vm::memory_manager::{MemoryManager, Object, ObjectType, VmData};
use vm::state::State;
use vm::{StepResult, Vm};

/// Run the interactive debugger TUI.
///
/// This takes ownership-borrow of the VM and drives it step-by-step,
/// recording every state snapshot so the user can rewind, inspect the
/// heap, scroll the stack, and use command mode for advanced navigation.
pub fn run_debug(vm: &mut Vm, info: DebugInfo) -> NovaResult<()> {

        // ── Heap cell info for heap inspect mode ─────────────────
        #[derive(Clone)]
        struct HeapCellInfo {
            index: usize,
            alive: bool,
            ref_count: usize,
            type_name: String,    // e.g. "List", "Struct(Point)", "Closure@42"
            data_preview: String, // short preview of .data contents
            fields: Vec<(String, String)>, // field_name → value (for structs)
            data_len: usize,
        }

        // ── Snapshot ────────────────────────────────────────────────
        #[derive(Clone)]
        struct Snapshot {
            ip: usize,           // IP of the instruction that was executed
            opname: String,
            named_op: String,
            stack: Vec<String>,
            callstack_depth: usize,
            offset: usize,
            locals: Vec<(String, String)>,
            scope_name: String,  // name of current function scope ("top-level", "fib", etc.)
            globals_changed: Vec<(String, String)>,
            output_len: usize,
            // Heap / GC stats captured at this step
            heap_live: usize,
            heap_capacity: usize,
            heap_free: usize,
            gc_threshold: usize,
            gc_base: usize,
            gc_locked: bool,
            stack_depth: usize,
            // Heap cell snapshot for inspect mode
            heap_cells: Vec<HeapCellInfo>,
        }

        fn object_type_name(obj: &Object) -> String {
            match &obj.object_type {
                ObjectType::List => "List".to_string(),
                ObjectType::String => "String".to_string(),
                ObjectType::Tuple => "Tuple".to_string(),
                ObjectType::Struct(name) => format!("Struct({})", name),
                ObjectType::Enum { name, tag } => format!("Enum({}::{})", name, tag),
                ObjectType::Closure(addr) => format!("Closure@{}", addr),
            }
        }

        fn heap_data_preview(obj: &Object) -> String {
            match &obj.object_type {
                ObjectType::String => {
                    // String data is stored as Char values
                    let s: String = obj.data.iter().filter_map(|v| {
                        if let VmData::Char(c) = v { Some(*c) } else { None }
                    }).collect();
                    let escaped: String = s.chars().map(|c| match c {
                        '\n' => '␊', '\r' => '␍', '\t' => '␉', '\0' => '␀',
                        c if c.is_control() => '·',
                        c => c,
                    }).collect();
                    if escaped.chars().count() > 30 {
                        let preview: String = escaped.chars().take(28).collect();
                        format!("\"{}..\"", preview)
                    } else {
                        format!("\"{}\"", escaped)
                    }
                }
                ObjectType::List | ObjectType::Tuple => {
                    let items: Vec<String> = obj.data.iter().take(8).map(fmt_vmdata).collect();
                    let mut s = format!("[{}]", items.join(", "));
                    if obj.data.len() > 8 {
                        s = format!("[{}, ...+{}]", items.join(", "), obj.data.len() - 8);
                    }
                    s
                }
                ObjectType::Struct(_) => {
                    let mut pairs: Vec<String> = obj.table.iter().take(6).map(|(k, idx)| {
                        let val = obj.data.get(*idx).map(fmt_vmdata).unwrap_or_else(|| "?".into());
                        format!("{}={}", k, val)
                    }).collect();
                    if obj.table.len() > 6 {
                        pairs.push(format!("...+{}", obj.table.len() - 6));
                    }
                    format!("{{{}}}", pairs.join(", "))
                }
                ObjectType::Enum { .. } => {
                    if obj.data.is_empty() {
                        String::new()
                    } else {
                        let items: Vec<String> = obj.data.iter().take(6).map(fmt_vmdata).collect();
                        format!("({})", items.join(", "))
                    }
                }
                ObjectType::Closure(addr) => {
                    if obj.data.is_empty() {
                        format!("@{}", addr)
                    } else {
                        let captures: Vec<String> = obj.data.iter().take(6).map(fmt_vmdata).collect();
                        format!("@{} [{}]", addr, captures.join(", "))
                    }
                }
            }
        }

        fn capture_heap_cells(memory: &MemoryManager) -> Vec<HeapCellInfo> {
            let mut cells = Vec::new();
            for i in 0..memory.heap_capacity() {
                if let Some(obj) = memory.ref_from_heap(i) {
                    // Live cell
                    let rc = memory.ref_count(i);
                    let fields: Vec<(String, String)> = if matches!(obj.object_type, ObjectType::Struct(_)) {
                        let mut f: Vec<(String, String)> = obj.table.iter().map(|(k, idx)| {
                            let val = obj.data.get(*idx).map(fmt_vmdata).unwrap_or_else(|| "?".into());
                            (k.clone(), val)
                        }).collect();
                        f.sort_by(|a, b| a.0.cmp(&b.0));
                        f
                    } else {
                        Vec::new()
                    };
                    cells.push(HeapCellInfo {
                        index: i,
                        alive: true,
                        ref_count: rc,
                        type_name: object_type_name(obj),
                        data_preview: heap_data_preview(obj),
                        fields,
                        data_len: obj.data.len(),
                    });
                } else {
                    cells.push(HeapCellInfo {
                        index: i,
                        alive: false,
                        ref_count: 0,
                        type_name: String::new(),
                        data_preview: String::new(),
                        fields: Vec::new(),
                        data_len: 0,
                    });
                }
            }
            cells
        }

        fn fmt_vmdata(v: &VmData) -> String {
            match v {
                VmData::Int(i) => format!("{}", i),
                VmData::Float(f) => format!("{}", f),
                VmData::Bool(b) => format!("{}", b),
                VmData::Char(c) => format!("'{}'", c),
                VmData::Function(f) => format!("<fn@{}>", f),
                VmData::Object(o) => format!("obj#{}", o),
                VmData::StackAddress(a) => format!("&{}", a),
                VmData::None => "None".to_string(),
            }
        }

        fn fmt_vmdata_typed(v: &VmData) -> String {
            match v {
                VmData::Int(i) => format!("Int({})", i),
                VmData::Float(f) => format!("Float({})", f),
                VmData::Bool(b) => format!("Bool({})", b),
                VmData::Char(c) => format!("Char('{}')", c),
                VmData::Function(f) => format!("Fn@{}", f),
                VmData::Object(o) => format!("Obj#{}", o),
                VmData::StackAddress(a) => format!("Addr({})", a),
                VmData::None => "None".to_string(),
            }
        }

    let take_snapshot = |state: &State,
                 executed_ip: usize,
                 opname: &str,
                 named_op: &str,
                 info: &DebugInfo,
                 output_len: usize,
                 fn_byte_ranges: &[(usize, usize, u64)]|
     -> Snapshot {
            // Build a reverse map: function byte-address → global name
            // so we can label Function values on the stack.
            let mut fn_addr_to_name: HashMap<usize, String> = HashMap::new();
            for (idx, name) in &info.global_names {
                let i = *idx as usize;
                if i < state.memory.stack.len() {
                    if let VmData::Function(addr) = &state.memory.stack[i] {
                        fn_addr_to_name.insert(*addr, name.clone());
                    }
                }
            }

            // Determine the current function scope from the IP
            let scope = scope_for_ip(executed_ip, fn_byte_ranges);

            // Build local_name lookup for current scope
            let local_name_map: HashMap<usize, String> = info
                .local_names
                .get(&scope)
                .map(|v| {
                    v.iter()
                        .map(|(idx, name)| (state.offset + *idx as usize, name.clone()))
                        .collect()
                })
                .unwrap_or_default();

            let stack: Vec<String> = state
                .memory
                .stack
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    let base = fmt_vmdata_typed(v);
                    // Try to attach a name
                    if let Some(gname) = info.global_names.get(&(i as u32)) {
                        format!("{} ({})", base, gname)
                    } else if let Some(lname) = local_name_map.get(&i) {
                        format!("{} ({})", base, lname)
                    } else if let VmData::Function(addr) = v {
                        if let Some(fname) = fn_addr_to_name.get(addr) {
                            format!("{} ({})", base, fname)
                        } else {
                            base
                        }
                    } else {
                        base
                    }
                })
                .collect();

            let mut locals = Vec::new();
            if let Some(local_names) = info.local_names.get(&scope) {
                for (idx, name) in local_names {
                    let abs_idx = state.offset + *idx as usize;
                    if abs_idx < state.memory.stack.len() {
                        let val = fmt_vmdata(&state.memory.stack[abs_idx]);
                        locals.push((name.clone(), val));
                    }
                }
            }

            let mut globals_changed = Vec::new();
            for (idx, name) in &info.global_names {
                let i = *idx as usize;
                if i < state.memory.stack.len() {
                    let v = &state.memory.stack[i];
                    match v {
                        VmData::Function(_) | VmData::None => {}
                        _ => {
                            globals_changed.push((name.clone(), fmt_vmdata(v)));
                        }
                    }
                }
            }

            // Determine scope name for display
            let scope_name = if scope == 0 {
                "top-level".to_string()
            } else {
                info.label_name(scope)
            };

            Snapshot {
                ip: executed_ip,
                opname: opname.to_string(),
                named_op: named_op.to_string(),
                stack,
                callstack_depth: state.callstack.len(),
                offset: state.offset,
                locals,
                scope_name,
                globals_changed,
                output_len,
                heap_live: state.memory.live_count(),
                heap_capacity: state.memory.heap_capacity(),
                heap_free: state.memory.free_count(),
                gc_threshold: state.memory.gc_threshold(),
                gc_base: state.memory.gc_base_threshold(),
                gc_locked: state.memory.gc_lock_depth() > 0,
                stack_depth: state.memory.stack.len(),
                heap_cells: capture_heap_cells(&state.memory),
            }
        };

        let mut history: Vec<Snapshot> = Vec::new();
        let mut cursor: usize = 0;
        let mut finished = false;
        let mut error_msg: Option<String> = None;
        let mut output_buf: Vec<String> = Vec::new();
        let mut show_help = false;
        let mut playing = false;
        let mut play_speed_ms: u64 = 100; // ms between auto-steps
        let mut heap_mode = false;       // Tab toggles between stack/heap view
        let mut heap_scroll: usize = 0;  // scroll offset in heap inspect panel
        let mut stack_scroll: usize = 0; // scroll offset in stack view (u/d keys)
        let mut command_mode = false;    // : key enters command mode
        let mut command_buf = String::new(); // command input buffer
        let mut command_error: Option<String> = None; // error from last command

        // Pre-decode bytecode listing for the left panel
        struct BytecodeLine {
            addr: usize,
            opname: String,
            operand: String,
        }

        let mut bc_lines: Vec<BytecodeLine> = Vec::new();
        {
            let prog = &vm.state.program;
            let mut pc = 0usize;
            while pc < prog.len() {
                let opcode = prog[pc];
                let opname = code::byte_to_string(opcode);
                let addr = pc;
                pc += 1;
                let operand = match opcode {
                    Code::INTEGER => {
                        let v = if pc + 8 <= prog.len() {
                            i64::from_le_bytes(
                                prog[pc..pc + 8].try_into().unwrap_or_default(),
                            )
                        } else {
                            0
                        };
                        pc += 8;
                        format!("{}", v)
                    }
                    Code::FLOAT => {
                        let v = if pc + 8 <= prog.len() {
                            f64::from_le_bytes(
                                prog[pc..pc + 8].try_into().unwrap_or_default(),
                            )
                        } else {
                            0.0
                        };
                        pc += 8;
                        format!("{}", v)
                    }
                    Code::STORE | Code::GET | Code::STOREFAST => {
                        let v = if pc + 4 <= prog.len() {
                            u32::from_le_bytes(
                                prog[pc..pc + 4].try_into().unwrap_or_default(),
                            )
                        } else {
                            0
                        };
                        pc += 4;
                        let name = info.local_name(0, v).unwrap_or_default();
                        if name.is_empty() {
                            format!("local[{}]", v)
                        } else {
                            name
                        }
                    }
                    Code::STOREGLOBAL | Code::GETGLOBAL => {
                        let v = if pc + 4 <= prog.len() {
                            u32::from_le_bytes(
                                prog[pc..pc + 4].try_into().unwrap_or_default(),
                            )
                        } else {
                            0
                        };
                        pc += 4;
                        info.global_name(v)
                    }
                    Code::DIRECTCALL => {
                        let v = if pc + 4 <= prog.len() {
                            u32::from_le_bytes(
                                prog[pc..pc + 4].try_into().unwrap_or_default(),
                            )
                        } else {
                            0
                        };
                        pc += 4;
                        info.global_name(v)
                    }
                    Code::ALLOCLOCALS | Code::ALLOCATEGLOBAL | Code::JUMPIFFALSE
                    | Code::JMP => {
                        let v = if pc + 4 <= prog.len() {
                            u32::from_le_bytes(
                                prog[pc..pc + 4].try_into().unwrap_or_default(),
                            )
                        } else {
                            0
                        };
                        pc += 4;
                        format!("+{}", v)
                    }
                    Code::BJMP => {
                        let v = if pc + 4 <= prog.len() {
                            u32::from_le_bytes(
                                prog[pc..pc + 4].try_into().unwrap_or_default(),
                            )
                        } else {
                            0
                        };
                        pc += 4;
                        format!("-{}", v)
                    }
                    Code::FUNCTION | Code::CLOSURE => {
                        let v = if pc + 4 <= prog.len() {
                            u32::from_le_bytes(
                                prog[pc..pc + 4].try_into().unwrap_or_default(),
                            )
                        } else {
                            0
                        };
                        pc += 4;
                        format!("+{}", v)
                    }
                    Code::OFFSET => {
                        let a = if pc + 4 <= prog.len() {
                            i32::from_le_bytes(
                                prog[pc..pc + 4].try_into().unwrap_or_default(),
                            )
                        } else {
                            0
                        };
                        let b = if pc + 8 <= prog.len() {
                            i32::from_le_bytes(
                                prog[pc + 4..pc + 8].try_into().unwrap_or_default(),
                            )
                        } else {
                            0
                        };
                        pc += 8;
                        format!("{},{}", a, b)
                    }
                    Code::RET => {
                        let v = if pc < prog.len() { prog[pc] } else { 0 };
                        pc += 1;
                        if v != 0 {
                            "(val)".into()
                        } else {
                            String::new()
                        }
                    }
                    Code::NATIVE => {
                        let v = if pc + 8 <= prog.len() {
                            u64::from_le_bytes(
                                prog[pc..pc + 8].try_into().unwrap_or_default(),
                            )
                        } else {
                            0
                        };
                        pc += 8;
                        info.native_name(v)
                    }
                    Code::CHAR => {
                        let v = if pc < prog.len() { prog[pc] as char } else { '?' };
                        pc += 1;
                        format!("'{}'", v)
                    }
                    Code::BYTE => {
                        let v = if pc < prog.len() { prog[pc] as i64 } else { 0 };
                        pc += 1;
                        format!("{}", v)
                    }
                    Code::NEWLIST | Code::NEWSTRUCT | Code::GETBIND | Code::CID
                    | Code::STACKREF => {
                        let v = if pc + 8 <= prog.len() {
                            u64::from_le_bytes(
                                prog[pc..pc + 8].try_into().unwrap_or_default(),
                            )
                        } else {
                            0
                        };
                        pc += 8;
                        format!("{}", v)
                    }
                    Code::STRING => {
                        let sz = if pc + 8 <= prog.len() {
                            u64::from_le_bytes(
                                prog[pc..pc + 8].try_into().unwrap_or_default(),
                            ) as usize
                        } else {
                            0
                        };
                        pc += 8;
                        let s = if pc + sz <= prog.len() {
                            let raw =
                                String::from_utf8_lossy(&prog[pc..pc + sz]).to_string();
                            // Escape control chars so they don't break TUI layout
                            let escaped: String = raw.chars().map(|c| match c {
                                '\n' => '␊',
                                '\r' => '␍',
                                '\t' => '␉',
                                '\0' => '␀',
                                c if c.is_control() => '·',
                                c => c,
                            }).collect();
                            let char_count = escaped.chars().count();
                            if char_count > 20 {
                                let preview: String = escaped.chars().take(18).collect();
                                format!("\"{}..\"", preview)
                            } else {
                                format!("\"{}\"", escaped)
                            }
                        } else {
                            "\"?\"".into()
                        };
                        pc += sz;
                        s
                    }
                    Code::REFID => {
                        pc += 2;
                        String::new()
                    }
                    _ => String::new(),
                };
                bc_lines.push(BytecodeLine {
                    addr,
                    opname,
                    operand,
                });
            }
        }

        // Map: byte address → bc_lines index
        let mut addr_to_line: HashMap<usize, usize> = HashMap::new();
        for (i, line) in bc_lines.iter().enumerate() {
            addr_to_line.insert(line.addr, i);
        }

        // Build function bytecode ranges: (body_start, body_end, label_id)
        // Used to determine which function scope the current IP belongs to.
        let addr_to_label = info.addr_to_label();
        let mut fn_byte_ranges: Vec<(usize, usize, u64)> = Vec::new();
        {
            let prog = &vm.state.program;
            let mut pc = 0usize;
            while pc < prog.len() {
                let opcode = prog[pc];
                match opcode {
                    Code::FUNCTION | Code::CLOSURE => {
                        let fn_addr = pc;
                        pc += 1; // skip opcode
                        let v = if pc + 4 <= prog.len() {
                            u32::from_le_bytes(
                                prog[pc..pc + 4].try_into().unwrap_or_default(),
                            ) as usize
                        } else {
                            0
                        };
                        pc += 4;
                        let body_end = fn_addr + 5 + v;
                        if let Some(&label_id) = addr_to_label.get(&(body_end as u64)) {
                            fn_byte_ranges.push((fn_addr + 5, body_end, label_id));
                        }
                    }
                    _ => {
                        // Skip this instruction (same logic as bc_lines pre-decode)
                        pc += 1;
                        match opcode {
                            Code::INTEGER | Code::FLOAT => pc += 8,
                            Code::STORE | Code::GET | Code::STOREFAST
                            | Code::STOREGLOBAL | Code::GETGLOBAL
                            | Code::DIRECTCALL | Code::ALLOCLOCALS
                            | Code::ALLOCATEGLOBAL | Code::JUMPIFFALSE
                            | Code::JMP | Code::BJMP => pc += 4,
                            Code::OFFSET => pc += 8,
                            Code::RET => pc += 1,
                            Code::NATIVE | Code::NEWLIST | Code::NEWSTRUCT
                            | Code::GETBIND | Code::CID | Code::STACKREF
                            | Code::TAILCALL => pc += 8,
                            Code::CHAR | Code::BYTE => pc += 1,
                            Code::STRING => {
                                let sz = if pc + 8 <= prog.len() {
                                    u64::from_le_bytes(
                                        prog[pc..pc + 8].try_into().unwrap_or_default(),
                                    ) as usize
                                } else {
                                    0
                                };
                                pc += 8 + sz;
                            }
                            Code::REFID => pc += 2,
                            _ => {}
                        }
                    }
                }
            }
        }

        /// Given the current IP, find the innermost enclosing function scope.
        /// Returns the label_id of the function, or 0 for top-level.
        fn scope_for_ip(ip: usize, fn_byte_ranges: &[(usize, usize, u64)]) -> u64 {
            fn_byte_ranges
                .iter()
                .filter(|(start, end, _)| ip >= *start && ip < *end)
                .min_by_key(|(start, end, _)| end - start)
                .map(|(_, _, label_id)| *label_id)
                .unwrap_or(0)
        }

        // Fix up bc_lines: re-resolve STORE/GET local names with correct function scope
        {
            let prog = &vm.state.program;
            for bl in bc_lines.iter_mut() {
                let opcode = prog[bl.addr];
                if matches!(opcode, Code::STORE | Code::GET | Code::STOREFAST) {
                    let v = if bl.addr + 1 + 4 <= prog.len() {
                        u32::from_le_bytes(
                            prog[bl.addr + 1..bl.addr + 5].try_into().unwrap_or_default(),
                        )
                    } else {
                        continue;
                    };
                    let scope = scope_for_ip(bl.addr, &fn_byte_ranges);
                    let name = info.local_name(scope, v).unwrap_or_default();
                    bl.operand = if name.is_empty() {
                        format!("local[{}]", v)
                    } else {
                        name
                    };
                }
            }
        }

        // Initial snapshot
        let ip0 = vm.state.current_instruction;
        let op0 = if ip0 < vm.state.program.len() {
            code::byte_to_string(vm.state.program[ip0])
        } else {
            "END".into()
        };
        let named0 = if ip0 < vm.state.program.len() {
            vm.peek_operand_named(vm.state.program[ip0], &info)
        } else {
            String::new()
        };
    history.push(take_snapshot(&vm.state, ip0, &op0, &named0, &info, 0, &fn_byte_ranges));

        // ── Display-width helpers ───────────────────────────────────
        // `str.len()` counts UTF-8 bytes, but terminals measure columns
        // by character count.  Our special chars (►, •, ─) are each 3
        // bytes but 1 column wide.  Control characters (\n, \t, etc.)
        // occupy 0 columns and must be skipped so they don't break
        // alignment.

        /// Pad `s` with trailing spaces to exactly `w` visible columns.
        /// If already wider, truncate to `w` columns.  Control characters
        /// are stripped to prevent layout breakage.
        fn pad(s: &str, w: usize) -> String {
            let mut out = String::with_capacity(w);
            let mut cols = 0usize;
            for c in s.chars() {
                if c.is_control() { continue; }
                if cols >= w { break; }
                out.push(c);
                cols += 1;
            }
            while cols < w {
                out.push(' ');
                cols += 1;
            }
            out
        }

        /// Truncate `s` to at most `w` visible columns.
        /// Control characters are stripped.
        fn trunc(s: &str, w: usize) -> String {
            let mut out = String::new();
            let mut cols = 0usize;
            for c in s.chars() {
                if c.is_control() { continue; }
                if cols >= w { break; }
                out.push(c);
                cols += 1;
            }
            out
        }

        // ── Render function ─────────────────────────────────────────
        let render = |stdout: &mut io::Stdout,
                      history: &[Snapshot],
                      cursor: usize,
                      finished: bool,
                      error_msg: &Option<String>,
                      output_buf: &[String],
                      bc_lines: &[BytecodeLine],
                      addr_to_line: &HashMap<usize, usize>,
                      show_help: bool,
                      playing: bool,
                      play_speed_ms: u64,
                      heap_mode: bool,
                      heap_scroll: usize,
                      stack_scroll: usize,
                      command_mode: bool,
                      command_buf: &str,
                      command_error: &Option<String>|
         -> io::Result<()> {
            let (cols, rows) = terminal::size()?;
            let w = cols as usize;
            let h = rows as usize;

            stdout.execute(terminal::Clear(ClearType::All))?;
            stdout.execute(cursor::MoveTo(0, 0))?;

            let snap = &history[cursor];

            if show_help {
                stdout
                    .queue(SetForegroundColor(Color::Cyan))?
                    .queue(SetAttribute(Attribute::Bold))?
                    .queue(Print("  Nova Debugger — Help\r\n"))?
                    .queue(SetAttribute(Attribute::Reset))?
                    .queue(Print("\r\n"))?;
                let help_lines = [
                    ("↑ / k", "Step backward (view previous state)"),
                    ("↓ / j", "Step forward (execute next instruction)"),
                    ("Space", "Step forward"),
                    ("u / d", "Scroll stack view up / down"),
                    ("PgUp", "Jump back 20 steps"),
                    ("PgDn", "Jump forward 20 steps"),
                    ("Home", "Go to beginning"),
                    ("End", "Go to latest step"),
                    ("Tab", "Toggle Stack / Heap inspect view"),
                    ("p", "Play / pause (auto-step visually)"),
                    ("+ / =", "Speed up playback"),
                    ("- / _", "Slow down playback"),
                    ("r", "Run to end (execute all remaining)"),
                    ("n", "Step over (run until callstack returns)"),
                    (":", "Enter command mode"),
                    ("?", "Toggle this help screen"),
                    ("q / Esc", "Quit debugger"),
                ];
                for (key, desc) in &help_lines {
                    stdout
                        .queue(SetForegroundColor(Color::Yellow))?
                        .queue(Print(format!("  {:>12}", key)))?
                        .queue(SetForegroundColor(Color::White))?
                        .queue(Print(format!("  {}\r\n", desc)))?
                        .queue(SetAttribute(Attribute::Reset))?;
                }
                stdout
                    .queue(Print("\r\n"))?
                    .queue(SetForegroundColor(Color::Cyan))?
                    .queue(Print("  Commands (press : to enter command mode):\r\n"))?
                    .queue(SetAttribute(Attribute::Reset))?;
                let cmd_lines = [
                    (":goto <addr>", "Jump bytecode view to address"),
                    (":step <n>", "Execute n steps forward"),
                    (":find <text>", "Search bytecode for text (opname/operand)"),
                    (":speed <ms>", "Set playback speed in milliseconds"),
                    (":heap", "Switch to heap inspect view"),
                    (":stack", "Switch to stack view"),
                    (":help", "Show this help screen"),
                    (":quit", "Quit debugger"),
                ];
                for (cmd, desc) in &cmd_lines {
                    stdout
                        .queue(SetForegroundColor(Color::Yellow))?
                        .queue(Print(format!("  {:>16}", cmd)))?
                        .queue(SetForegroundColor(Color::White))?
                        .queue(Print(format!("  {}\r\n", desc)))?
                        .queue(SetAttribute(Attribute::Reset))?;
                }
                stdout
                    .queue(Print("\r\n"))?
                    .queue(SetForegroundColor(Color::Cyan))?
                    .queue(Print("  Layout:\r\n"))?
                    .queue(SetAttribute(Attribute::Reset))?
                    .queue(SetForegroundColor(Color::White))?
                    .queue(Print(
                        "    Left:    Bytecode listing (> = current)\r\n",
                    ))?
                    .queue(Print(
                        "    Middle:  Stack (top-of-stack first, • = local)\r\n",
                    ))?
                    .queue(Print("    Right:   Variables (locals + globals)\r\n"))?
                    .queue(Print("    Tab:     Heap inspect (scroll through heap cells)\r\n"))?
                    .queue(Print("    Bottom:  Program output + heap/GC status\r\n"))?
                    .queue(SetAttribute(Attribute::Reset))?
                    .queue(Print("\r\n"))?
                    .queue(SetForegroundColor(Color::Cyan))?
                    .queue(Print("  Heap/GC bar:\r\n"))?
                    .queue(SetAttribute(Attribute::Reset))?
                    .queue(SetForegroundColor(Color::White))?
                    .queue(Print("    live    — heap objects currently alive\r\n"))?
                    .queue(Print("    slots   — total heap array size (live + freed)\r\n"))?
                    .queue(Print("    free    — recycled slots on the free-list\r\n"))?
                    .queue(Print("    GC next — alloc count that triggers next collection\r\n"))?
                    .queue(Print("    base    — adaptive threshold (grows/shrinks with load)\r\n"))?
                    .queue(Print("    LOCKED  — GC inhibited (mid-opcode safety window)\r\n"))?
                    .queue(SetAttribute(Attribute::Reset))?
                    .queue(Print("\r\n"))?
                    .queue(SetForegroundColor(Color::DarkGrey))?
                    .queue(Print("  Press ? or any key to return.\r\n"))?
                    .queue(SetAttribute(Attribute::Reset))?;
                stdout.flush()?;
                return Ok(());
            }

            // Layout: 3 columns — Bytecode | Stack | Variables
            let col1_w = (w * 35 / 100).max(28).min(w.saturating_sub(40));
            let remaining = w.saturating_sub(col1_w).saturating_sub(2); // 2 for │ separators
            let col2_w = (remaining * 55 / 100).max(15);
            let col3_w = remaining.saturating_sub(col2_w);

            // Header bar
            let status = if let Some(ref e) = error_msg {
                format!(" ERROR: {}", e)
            } else if finished {
                " ✓ FINISHED".to_string()
            } else if playing {
                format!(" ▶ PLAYING ({}ms)", play_speed_ms)
            } else {
                String::new()
            };
            let mode_tag = if heap_mode { " [HEAP]" } else { "" };
            let header = format!(
                " Nova Debugger │ Step {}/{} │ IP:{} │ Depth:{} │ Offset:{}{}{}",
                cursor + 1,
                history.len(),
                snap.ip,
                snap.callstack_depth,
                snap.offset,
                mode_tag,
                status
            );
            let hdr_display = trunc(&header, w);
            stdout
                .queue(SetForegroundColor(Color::Cyan))?
                .queue(SetAttribute(Attribute::Bold))?
                .queue(Print(&hdr_display))?
                .queue(SetAttribute(Attribute::Reset))?
                .queue(Print("\r\n"))?;

            // Current instruction highlight
            stdout
                .queue(SetForegroundColor(Color::Yellow))?
                .queue(SetAttribute(Attribute::Bold))?
                .queue(Print(format!(" ► {} {}", snap.opname, snap.named_op)))?
                .queue(SetAttribute(Attribute::Reset))?
                .queue(Print("\r\n"))?;

            let sep = "─".repeat(w);
            stdout
                .queue(SetForegroundColor(Color::DarkGrey))?
                .queue(Print(&sep))?
                .queue(Print("\r\n"))?
                .queue(SetAttribute(Attribute::Reset))?;

            // Available rows for the main panels
            // Header(1) + instruction(1) + sep(1) + sep(1) + output(2) + heap_bar(1) + controls(1) = 8
            let panel_rows = h.saturating_sub(8);

            // Column 1: Bytecode listing
            let current_bc_line =
                addr_to_line.get(&snap.ip).copied().unwrap_or(0);
            let half = panel_rows / 2;
            let bc_start = current_bc_line.saturating_sub(half);

            // Right-side panel width (everything after bytecode column + 1 separator)
            let right_w = w.saturating_sub(col1_w).saturating_sub(1);

            // Build heap inspect lines (only used when heap_mode is true)
            let mut heap_lines: Vec<(Color, String)> = Vec::new();
            if heap_mode {
                let live_count = snap.heap_cells.iter().filter(|c| c.alive).count();
                let total = snap.heap_cells.len();
                heap_lines.push((Color::Magenta, format!(
                    "─ Heap Inspect ─  {} live / {} slots  (Tab=stack view, ↑↓=scroll)",
                    live_count, total
                )));

                if snap.heap_cells.is_empty() {
                    heap_lines.push((Color::DarkGrey, " (heap empty)".to_string()));
                } else {
                    // Build expanded lines for all cells, then window
                    let mut cell_lines: Vec<(Color, String)> = Vec::new();
                    for cell in &snap.heap_cells {
                        if !cell.alive {
                            cell_lines.push((Color::DarkGrey, format!(
                                " [{:>4}]  (free)", cell.index
                            )));
                            continue;
                        }
                        let main = format!(
                            " [{:>4}]  {}  rc={}  len={}  {}",
                            cell.index, cell.type_name, cell.ref_count,
                            cell.data_len, cell.data_preview
                        );
                        let color = match cell.type_name.as_str() {
                            "String" => Color::Green,
                            "List" => Color::Cyan,
                            "Tuple" => Color::Blue,
                            _ if cell.type_name.starts_with("Struct") => Color::Yellow,
                            _ if cell.type_name.starts_with("Enum") => Color::Magenta,
                            _ if cell.type_name.starts_with("Closure") => Color::DarkYellow,
                            _ => Color::White,
                        };
                        cell_lines.push((color, main));
                        for (fname, fval) in &cell.fields {
                            cell_lines.push((Color::DarkGrey, format!(
                                "          .{} = {}", fname, fval
                            )));
                        }
                    }
                    let visible = panel_rows.saturating_sub(1);
                    let scroll = heap_scroll.min(cell_lines.len().saturating_sub(1));
                    for line in cell_lines.iter().skip(scroll).take(visible) {
                        heap_lines.push(line.clone());
                    }
                    if cell_lines.len() > visible + scroll {
                        heap_lines.push((Color::DarkGrey, format!(
                            " ... {} more below (↓ to scroll)",
                            cell_lines.len().saturating_sub(visible + scroll)
                        )));
                    }
                }
            }

            // Build stack + variables lines (only used when heap_mode is false)
            let mut stack_lines: Vec<(Color, String)> = Vec::new();
            let mut var_lines: Vec<(Color, String)> = Vec::new();
            if !heap_mode {
                stack_lines.push((Color::Cyan, "─ Stack ─  (u/d scroll)".to_string()));
                let stack_label = format!(
                    " ({} entries, offset={})",
                    snap.stack.len(),
                    snap.offset
                );
                stack_lines.push((Color::DarkGrey, stack_label));

                let max_stack = panel_rows.saturating_sub(3).max(3);
                // Build all stack entries top-of-stack first
                let all_stack: Vec<_> = snap
                    .stack
                    .iter()
                    .enumerate()
                    .rev()
                    .collect();
                // Apply scroll offset (stack_scroll scrolls from the top-of-stack downward)
                let scroll = stack_scroll.min(all_stack.len().saturating_sub(1));
                let visible: Vec<_> = all_stack.iter().skip(scroll).take(max_stack).copied().collect();
                for (i, entry) in &visible {
                    let marker = if *i == snap.stack.len().saturating_sub(1) {
                        "►"
                    } else {
                        " "
                    };
                    let local_tag = if *i >= snap.offset { "•" } else { " " };
                    let s = format!("{}{} [{:>3}] {}", marker, local_tag, i, entry);
                    let color = if *i == snap.stack.len().saturating_sub(1) {
                        Color::Green
                    } else if *i >= snap.offset {
                        Color::White
                    } else {
                        Color::DarkGrey
                    };
                    stack_lines.push((color, s));
                }
                if scroll > 0 {
                    stack_lines.push((
                        Color::DarkGrey,
                        format!(" ... {} above (u to scroll up)", scroll),
                    ));
                }
                let remaining_below = all_stack.len().saturating_sub(scroll + max_stack);
                if remaining_below > 0 {
                    stack_lines.push((
                        Color::DarkGrey,
                        format!(" ... {} below (d to scroll down)", remaining_below),
                    ));
                }
                if snap.stack.is_empty() {
                    stack_lines.push((Color::DarkGrey, " (empty)".to_string()));
                }

                var_lines.push((Color::Cyan, "─ Variables ─".to_string()));
                if snap.locals.is_empty() && snap.globals_changed.is_empty() {
                    var_lines.push((Color::DarkGrey, " (none)".to_string()));
                }
                if !snap.locals.is_empty() {
                    var_lines.push((Color::DarkGrey, format!(" locals ({})", snap.scope_name)));
                    for (name, val) in &snap.locals {
                        var_lines
                            .push((Color::Green, format!("  {} = {}", name, val)));
                    }
                }
                if !snap.globals_changed.is_empty() {
                    var_lines.push((Color::DarkGrey, " globals:".to_string()));
                    for (name, val) in &snap.globals_changed {
                        var_lines
                            .push((Color::White, format!("  {} = {}", name, val)));
                    }
                }
            }

            // Render: bytecode (left) | right panel
            for row in 0..panel_rows {
                // Column 1: Bytecode
                let bc_idx = bc_start + row;
                if bc_idx < bc_lines.len() {
                    let bl = &bc_lines[bc_idx];
                    let is_current = bc_idx == current_bc_line;
                    let marker = if is_current { ">" } else { " " };
                    let addr_str = format!("{:>5}", bl.addr);
                    let text = format!(
                        "{} {} {:<12} {}",
                        marker, addr_str, bl.opname, bl.operand
                    );
                    let display = pad(&text, col1_w);

                    if is_current {
                        stdout
                            .queue(SetForegroundColor(Color::Yellow))?
                            .queue(SetAttribute(Attribute::Bold))?
                            .queue(Print(&display))?
                            .queue(SetAttribute(Attribute::Reset))?;
                    } else {
                        stdout
                            .queue(SetForegroundColor(Color::DarkGrey))?
                            .queue(Print(&display))?
                            .queue(SetAttribute(Attribute::Reset))?;
                    }
                } else {
                    stdout.queue(Print(&" ".repeat(col1_w)))?;
                }

                // Separator
                stdout
                    .queue(SetForegroundColor(Color::DarkGrey))?
                    .queue(Print("│"))?
                    .queue(SetAttribute(Attribute::Reset))?;

                if heap_mode {
                    // Heap inspect: single wide panel
                    if row < heap_lines.len() {
                        let (color, ref text) = heap_lines[row];
                        let display = pad(text, right_w);
                        stdout
                            .queue(SetForegroundColor(color))?
                            .queue(Print(&display))?
                            .queue(SetAttribute(Attribute::Reset))?;
                    }
                } else {
                    // Stack + Variables: two sub-columns
                    if row < stack_lines.len() {
                        let (color, ref text) = stack_lines[row];
                        let display = pad(text, col2_w);
                        stdout
                            .queue(SetForegroundColor(color))?
                            .queue(Print(&display))?
                            .queue(SetAttribute(Attribute::Reset))?;
                    } else {
                        stdout.queue(Print(&" ".repeat(col2_w)))?;
                    }

                    // Separator
                    stdout
                        .queue(SetForegroundColor(Color::DarkGrey))?
                        .queue(Print("│"))?
                        .queue(SetAttribute(Attribute::Reset))?;

                    // Variables sub-column
                    if row < var_lines.len() {
                        let (color, ref text) = var_lines[row];
                        let display = trunc(text, col3_w);
                        stdout
                            .queue(SetForegroundColor(color))?
                            .queue(Print(&display))?
                            .queue(SetAttribute(Attribute::Reset))?;
                    }
                }

                stdout.queue(Print("\r\n"))?;
            }

            // Output section
            stdout
                .queue(SetForegroundColor(Color::DarkGrey))?
                .queue(Print(&sep))?
                .queue(Print("\r\n"))?
                .queue(SetAttribute(Attribute::Reset))?;

            let out_rows = 2;
            let out_len = std::cmp::min(snap.output_len, output_buf.len());
            let out_start = out_len.saturating_sub(out_rows);
            if out_len == 0 {
                stdout
                    .queue(SetForegroundColor(Color::DarkGrey))?
                    .queue(Print(" Output: (none)"))?
                    .queue(SetAttribute(Attribute::Reset))?
                    .queue(Print("\r\n"))?;
            } else {
                for line in output_buf.iter().take(out_len).skip(out_start) {
                    let trimmed = trunc(line, w.saturating_sub(2));
                    let display = format!(" {}", trimmed);
                    stdout
                        .queue(SetForegroundColor(Color::White))?
                        .queue(Print(&display))?
                        .queue(Print("\r\n"))?
                        .queue(SetAttribute(Attribute::Reset))?;
                }
            }

            // Heap / GC status bar
            let gc_lock_tag = if snap.gc_locked { " LOCKED" } else { "" };
            let heap_bar = format!(
                " Heap: {} live / {} slots ({} free) │ Stack: {} │ GC next@{} base={}{} │ Calls: {}{}",
                snap.heap_live,
                snap.heap_capacity,
                snap.heap_free,
                snap.stack_depth,
                snap.gc_threshold,
                snap.gc_base,
                gc_lock_tag,
                snap.callstack_depth,
                if playing {
                    format!(" │ Speed: {}ms", play_speed_ms)
                } else {
                    String::new()
                },
            );
            let heap_display = trunc(&heap_bar, w);
            stdout
                .queue(SetForegroundColor(Color::DarkGrey))?
                .queue(Print(&heap_display))?
                .queue(SetAttribute(Attribute::Reset))?
                .queue(Print("\r\n"))?;

            // Controls bar / command mode
            if command_mode {
                // Show command input line
                let prompt = format!(":{}", command_buf);
                stdout
                    .queue(SetForegroundColor(Color::Yellow))?
                    .queue(SetAttribute(Attribute::Bold))?
                    .queue(Print(&prompt))?
                    .queue(SetAttribute(Attribute::Reset))?;
                // Show cursor blinking hint
                stdout
                    .queue(SetForegroundColor(Color::DarkGrey))?
                    .queue(Print("█  (Enter=run, Esc=cancel)"))?
                    .queue(SetAttribute(Attribute::Reset))?;
            } else if let Some(ref cmd_err) = command_error {
                // Show last command error briefly
                stdout
                    .queue(SetForegroundColor(Color::Red))?
                    .queue(Print(format!(" {}", trunc(cmd_err, w.saturating_sub(2)))))?
                    .queue(SetAttribute(Attribute::Reset))?;
            } else {
                let controls = if heap_mode {
                    " [Tab] stack  [↑↓] scroll heap  [u/d] stack scroll  [:] cmd  [p] play  [?] help  [q] quit"
                } else if playing {
                    " [p] pause  [+/-] speed  [Tab] heap  [u/d] stack scroll  [:] cmd  [?] help  [q] quit"
                } else {
                    " [↑↓] step  [u/d] scroll  [Tab] heap  [:] cmd  [p] play  [r] run  [n] next  [?] help  [q] quit"
                };
                stdout
                    .queue(SetForegroundColor(Color::DarkGrey))?
                    .queue(Print(controls))?
                    .queue(SetAttribute(Attribute::Reset))?;
            }

            stdout.flush()?;
            Ok(())
        };

        // ── Raylib detection ────────────────────────────────────────
        // Scan native function names for any "raylib::" prefix.
        // If the program uses raylib, we must NOT use the TUI (crossterm
        // raw mode + alternate screen) because raylib creates its own
        // GLFW/OpenGL window and needs the main thread's event loop to
        // run freely.  Instead we branch into a frame-stepping mode.
        let uses_raylib = info
            .native_names
            .values()
            .any(|name| name.starts_with("raylib::"));

        let raylib_rendering_index: Option<u64> = info
            .native_names
            .iter()
            .find(|(_, name)| name.starts_with("raylib::rendering"))
            .map(|(idx, _)| *idx);

        if uses_raylib {
            return run_debug_raylib_mode(
                vm, &info, raylib_rendering_index,
            );
        }

        // ── Main loop ───────────────────────────────────────────────
        let mut stdout = io::stdout();
        terminal::enable_raw_mode().map_err(|e| {
            Box::new(NovaError::Runtime {
                msg: format!("raw mode: {}", e).into(),
            })
        })?;
        stdout
            .execute(terminal::EnterAlternateScreen)
            .map_err(|e| {
                Box::new(NovaError::Runtime {
                    msg: format!("alt screen: {}", e).into(),
                })
            })?;

        let _ = render(
            &mut stdout,
            &history,
            cursor,
            finished,
            &error_msg,
            &output_buf,
            &bc_lines,
            &addr_to_line,
            show_help,
            playing,
            play_speed_ms,
            heap_mode,
            heap_scroll,
            stack_scroll,
            command_mode,
            &command_buf,
            &command_error,
        );

        loop {
            // When playing, poll with a timeout so we auto-step if no key
            // is pressed.  When paused, block until a key arrives.
            let timeout = if playing {
                Duration::from_millis(play_speed_ms)
            } else {
                Duration::from_secs(3600) // effectively blocking
            };

            let got_key = event::poll(timeout).unwrap_or(false);

            if got_key {
                if let Ok(Event::Key(KeyEvent {
                    code: key,
                    modifiers,
                    ..
                })) = event::read()
                {
                    if show_help {
                        show_help = false;
                        let _ = render(
                            &mut stdout,
                            &history,
                            cursor,
                            finished,
                            &error_msg,
                            &output_buf,
                            &bc_lines,
                            &addr_to_line,
                            show_help,
                            playing,
                            play_speed_ms,
                            heap_mode,
                            heap_scroll,
                            stack_scroll,
                            command_mode,
                            &command_buf,
                            &command_error,
                        );
                        continue;
                    }

                    // ── Command mode key handling ────────────────
                    if command_mode {
                        match key {
                            KeyCode::Esc => {
                                command_mode = false;
                                command_buf.clear();
                            }
                            KeyCode::Enter => {
                                command_mode = false;
                                let cmd = command_buf.trim().to_string();
                                command_buf.clear();
                                command_error = None;

                                if cmd.is_empty() {
                                    // Nothing to do
                                } else if cmd == "quit" || cmd == "q" {
                                    // Exit debugger
                                    let _ = stdout.execute(terminal::LeaveAlternateScreen);
                                    let _ = terminal::disable_raw_mode();
                                    return Ok(());
                                } else if cmd == "help" || cmd == "h" {
                                    show_help = true;
                                    playing = false;
                                } else if cmd == "heap" {
                                    heap_mode = true;
                                    heap_scroll = 0;
                                } else if cmd == "stack" {
                                    heap_mode = false;
                                    stack_scroll = 0;
                                } else if let Some(rest) = cmd.strip_prefix("goto ") {
                                    match rest.trim().parse::<usize>() {
                                        Ok(addr) => {
                                            // Find a history snapshot with this IP, or find the bc_line
                                            if let Some(pos) = history.iter().position(|s| s.ip == addr) {
                                                cursor = pos;
                                                command_error = None;
                                            } else if addr_to_line.contains_key(&addr) {
                                                // Address exists in bytecode but no snapshot there yet.
                                                // Step forward until we reach it or run out.
                                                let limit = 1_000_000usize;
                                                let mut count = 0;
                                                while !finished && error_msg.is_none() && count < limit {
                                                    let ip_before = vm.state.current_instruction;
                                                    if ip_before == addr {
                                                        // We've reached the target
                                                        break;
                                                    }
                                                    let opcode = if ip_before < vm.state.program.len() {
                                                        vm.state.program[ip_before]
                                                    } else {
                                                        0
                                                    };
                                                    let named = vm.peek_operand_named(opcode, &info);
                                                    match vm.step_one_debug() {
                                                        StepResult::Continue { opname, output } => {
                                                            if let Some(ref o) = output {
                                                                output_buf.push(o.clone());
                                                            }
                                                            history.push(take_snapshot(
                                                                &vm.state, ip_before, &opname, &named,
                                                                &info, output_buf.len(), &fn_byte_ranges,
                                                            ));
                                                        }
                                                        StepResult::Finished { output } => {
                                                            if let Some(ref o) = output {
                                                                output_buf.push(o.clone());
                                                            }
                                                            finished = true;
                                                            history.push(take_snapshot(
                                                                &vm.state, ip_before, "END", "",
                                                                &info, output_buf.len(), &fn_byte_ranges,
                                                            ));
                                                            break;
                                                        }
                                                        StepResult::Error(msg) => {
                                                            error_msg = Some(msg);
                                                            history.push(take_snapshot(
                                                                &vm.state, ip_before, "ERROR", "",
                                                                &info, output_buf.len(), &fn_byte_ranges,
                                                            ));
                                                            break;
                                                        }
                                                    }
                                                    count += 1;
                                                }
                                                cursor = history.len().saturating_sub(1);
                                                if count >= limit {
                                                    command_error = Some(format!("goto {}: address not reached after {} steps", addr, limit));
                                                }
                                            } else {
                                                command_error = Some(format!("goto: address {} not found in bytecode", addr));
                                            }
                                        }
                                        Err(_) => {
                                            command_error = Some(format!("goto: '{}' is not a valid address (expected a number)", rest.trim()));
                                        }
                                    }
                                } else if let Some(rest) = cmd.strip_prefix("step ") {
                                    match rest.trim().parse::<usize>() {
                                        Ok(0) => {
                                            command_error = Some("step: count must be > 0".to_string());
                                        }
                                        Ok(n) => {
                                            playing = false;
                                            for _ in 0..n {
                                                if cursor < history.len() - 1 {
                                                    cursor += 1;
                                                } else if !finished && error_msg.is_none() {
                                                    let ip_before = vm.state.current_instruction;
                                                    let opcode = if ip_before < vm.state.program.len() {
                                                        vm.state.program[ip_before]
                                                    } else {
                                                        0
                                                    };
                                                    let named = vm.peek_operand_named(opcode, &info);
                                                    match vm.step_one_debug() {
                                                        StepResult::Continue { opname, output } => {
                                                            if let Some(ref o) = output {
                                                                output_buf.push(o.clone());
                                                            }
                                                            history.push(take_snapshot(
                                                                &vm.state, ip_before, &opname, &named,
                                                                &info, output_buf.len(), &fn_byte_ranges,
                                                            ));
                                                            cursor = history.len() - 1;
                                                        }
                                                        StepResult::Finished { output } => {
                                                            if let Some(ref o) = output {
                                                                output_buf.push(o.clone());
                                                            }
                                                            finished = true;
                                                            history.push(take_snapshot(
                                                                &vm.state, ip_before, "END", "",
                                                                &info, output_buf.len(), &fn_byte_ranges,
                                                            ));
                                                            cursor = history.len() - 1;
                                                            break;
                                                        }
                                                        StepResult::Error(msg) => {
                                                            error_msg = Some(msg);
                                                            history.push(take_snapshot(
                                                                &vm.state, ip_before, "ERROR", "",
                                                                &info, output_buf.len(), &fn_byte_ranges,
                                                            ));
                                                            cursor = history.len() - 1;
                                                            break;
                                                        }
                                                    }
                                                } else {
                                                    break;
                                                }
                                            }
                                        }
                                        Err(_) => {
                                            command_error = Some(format!("step: '{}' is not a valid number", rest.trim()));
                                        }
                                    }
                                } else if let Some(rest) = cmd.strip_prefix("find ") {
                                    let needle = rest.trim().to_lowercase();
                                    if needle.is_empty() {
                                        command_error = Some("find: please provide a search term".to_string());
                                    } else {
                                        // Search bc_lines for a match starting from the current bc line
                                        let current_bc = addr_to_line.get(&history[cursor].ip).copied().unwrap_or(0);
                                        let total = bc_lines.len();
                                        let mut found = false;
                                        for offset in 1..=total {
                                            let idx = (current_bc + offset) % total;
                                            let bl = &bc_lines[idx];
                                            if bl.opname.to_lowercase().contains(&needle)
                                                || bl.operand.to_lowercase().contains(&needle)
                                            {
                                                // Found a match — try to navigate to this address
                                                let target_addr = bl.addr;
                                                // Check if we already have a snapshot at this IP
                                                if let Some(pos) = history.iter().position(|s| s.ip == target_addr) {
                                                    cursor = pos;
                                                } else {
                                                    // Step forward until we hit the address or finish
                                                    let limit = 1_000_000usize;
                                                    let mut count = 0;
                                                    while !finished && error_msg.is_none() && count < limit {
                                                        let ip_before = vm.state.current_instruction;
                                                        if ip_before == target_addr {
                                                            break;
                                                        }
                                                        let opc = if ip_before < vm.state.program.len() {
                                                            vm.state.program[ip_before]
                                                        } else {
                                                            0
                                                        };
                                                        let named = vm.peek_operand_named(opc, &info);
                                                        match vm.step_one_debug() {
                                                            StepResult::Continue { opname, output } => {
                                                                if let Some(ref o) = output {
                                                                    output_buf.push(o.clone());
                                                                }
                                                                history.push(take_snapshot(
                                                                    &vm.state, ip_before, &opname, &named,
                                                                    &info, output_buf.len(), &fn_byte_ranges,
                                                                ));
                                                            }
                                                            StepResult::Finished { output } => {
                                                                if let Some(ref o) = output {
                                                                    output_buf.push(o.clone());
                                                                }
                                                                finished = true;
                                                                history.push(take_snapshot(
                                                                    &vm.state, ip_before, "END", "",
                                                                    &info, output_buf.len(), &fn_byte_ranges,
                                                                ));
                                                                break;
                                                            }
                                                            StepResult::Error(msg) => {
                                                                error_msg = Some(msg);
                                                                history.push(take_snapshot(
                                                                    &vm.state, ip_before, "ERROR", "",
                                                                    &info, output_buf.len(), &fn_byte_ranges,
                                                                ));
                                                                break;
                                                            }
                                                        }
                                                        count += 1;
                                                    }
                                                    cursor = history.len().saturating_sub(1);
                                                    if count >= limit {
                                                        command_error = Some(format!("find: '{}' address not reached after {} steps", needle, limit));
                                                    }
                                                }
                                                found = true;
                                                break;
                                            }
                                        }
                                        if !found {
                                            command_error = Some(format!("find: '{}' not found in bytecode", needle));
                                        }
                                    }
                                } else if let Some(rest) = cmd.strip_prefix("speed ") {
                                    match rest.trim().parse::<u64>() {
                                        Ok(ms) if (1..=10000).contains(&ms) => {
                                            play_speed_ms = ms;
                                        }
                                        Ok(_) => {
                                            command_error = Some("speed: value must be between 1 and 10000 ms".to_string());
                                        }
                                        Err(_) => {
                                            command_error = Some(format!("speed: '{}' is not a valid number", rest.trim()));
                                        }
                                    }
                                } else {
                                    command_error = Some(format!("unknown command: '{}' (try :help)", cmd));
                                }
                            }
                            KeyCode::Backspace => {
                                command_buf.pop();
                            }
                            KeyCode::Char(c) => {
                                command_buf.push(c);
                            }
                            _ => {}
                        }
                        let _ = render(
                            &mut stdout, &history, cursor, finished, &error_msg,
                            &output_buf, &bc_lines, &addr_to_line, show_help,
                            playing, play_speed_ms, heap_mode, heap_scroll,
                            stack_scroll, command_mode, &command_buf, &command_error,
                        );
                        continue;
                    }

                    // Clear any command error on next keypress
                    if command_error.is_some() {
                        command_error = None;
                    }

                    match key {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char('c')
                            if modifiers.contains(KeyModifiers::CONTROL) =>
                        {
                            break
                        }
                        KeyCode::Char('?') => {
                            show_help = true;
                            playing = false;
                        }

                        // Toggle heap inspect / stack view
                        KeyCode::Tab => {
                            heap_mode = !heap_mode;
                            if heap_mode {
                                heap_scroll = 0;
                            }
                        }

                        // Play / Pause toggle
                        KeyCode::Char('p') => {
                            if playing {
                                playing = false;
                            } else if cursor < history.len() - 1 {
                                // There is recorded history ahead — replay it
                                playing = true;
                            } else if !finished && error_msg.is_none() {
                                // At the end of history, but program still running
                                playing = true;
                            }
                        }

                        // Speed controls
                        KeyCode::Char('+') | KeyCode::Char('=') => {
                            if play_speed_ms > 10 {
                                play_speed_ms = play_speed_ms.saturating_sub(
                                    if play_speed_ms > 100 { 50 } else { 10 },
                                );
                            }
                        }
                        KeyCode::Char('-') | KeyCode::Char('_') => {
                            play_speed_ms = (play_speed_ms + if play_speed_ms >= 100 { 50 } else { 10 }).min(2000);
                        }

                        // Enter command mode
                        KeyCode::Char(':') => {
                            command_mode = true;
                            command_buf.clear();
                            command_error = None;
                            playing = false;
                        }

                        // Stack scroll (u = up / d = down) — only in stack view
                        KeyCode::Char('u') if !heap_mode => {
                            stack_scroll = stack_scroll.saturating_sub(1);
                        }
                        KeyCode::Char('d') if !heap_mode => {
                            stack_scroll = stack_scroll.saturating_add(1);
                        }

                        // Step forward (pauses play mode)
                        // In heap mode, ↑/↓/j/k scroll the heap; Space still steps.
                        KeyCode::Down | KeyCode::Char('j') if heap_mode => {
                            heap_scroll = heap_scroll.saturating_add(1);
                        }
                        KeyCode::Up | KeyCode::Char('k') if heap_mode => {
                            heap_scroll = heap_scroll.saturating_sub(1);
                        }
                        KeyCode::PageDown if heap_mode => {
                            heap_scroll = heap_scroll.saturating_add(20);
                        }
                        KeyCode::PageUp if heap_mode => {
                            heap_scroll = heap_scroll.saturating_sub(20);
                        }
                        KeyCode::Home if heap_mode => {
                            heap_scroll = 0;
                        }
                        KeyCode::End if heap_mode => {
                            // Scroll to bottom of heap
                            let snap = &history[cursor];
                            heap_scroll = snap.heap_cells.len().saturating_sub(1);
                        }

                        KeyCode::Down
                        | KeyCode::Char('j')
                        | KeyCode::Char(' ') => {
                            playing = false;
                            if cursor < history.len() - 1 {
                                cursor += 1;
                            } else if !finished && error_msg.is_none() {
                                let ip_before = vm.state.current_instruction;
                                let opcode = if ip_before < vm.state.program.len()
                                {
                                    vm.state.program[ip_before]
                                } else {
                                    0
                                };
                                let named =
                                    vm.peek_operand_named(opcode, &info);
                                match vm.step_one_debug() {
                                    StepResult::Continue {
                                        opname,
                                        output,
                                    } => {
                                        if let Some(ref o) = output {
                                            output_buf.push(o.clone());
                                        }
                                        history.push(take_snapshot(
                                            &vm.state,
                                            ip_before,
                                            &opname,
                                            &named,
                                            &info,
                                            output_buf.len(),
                                            &fn_byte_ranges,
                                        ));
                                        cursor = history.len() - 1;
                                    }
                                    StepResult::Finished { output } => {
                                        if let Some(ref o) = output {
                                            output_buf.push(o.clone());
                                        }
                                        finished = true;
                                        history.push(take_snapshot(
                                            &vm.state,
                                            ip_before,
                                            "END",
                                            "",
                                            &info,
                                            output_buf.len(),
                                            &fn_byte_ranges,
                                        ));
                                        cursor = history.len() - 1;
                                    }
                                    StepResult::Error(msg) => {
                                        error_msg = Some(msg);
                                        history.push(take_snapshot(
                                            &vm.state,
                                            ip_before,
                                            "ERROR",
                                            "",
                                            &info,
                                            output_buf.len(),
                                            &fn_byte_ranges,
                                        ));
                                        cursor = history.len() - 1;
                                    }
                                }
                            }
                        }

                        // Step backward (pauses play mode)
                        KeyCode::Up | KeyCode::Char('k') => {
                            playing = false;
                            cursor = cursor.saturating_sub(1);
                        }

                        // Page navigation (pauses play mode)
                        KeyCode::PageDown => {
                            playing = false;
                            for _ in 0..20 {
                                if cursor < history.len() - 1 {
                                    cursor += 1;
                                } else if !finished && error_msg.is_none() {
                                    let ip_b = vm.state.current_instruction;
                                    let opc =
                                        if ip_b < vm.state.program.len() {
                                            vm.state.program[ip_b]
                                        } else {
                                            0
                                        };
                                    let named =
                                        vm.peek_operand_named(opc, &info);
                                    match vm.step_one_debug() {
                                        StepResult::Continue {
                                            opname,
                                                output,
                                        } => {
                                            if let Some(ref o) = output {
                                                output_buf.push(o.clone());
                                            }
                                            history.push(take_snapshot(
                                            &vm.state,
                                            ip_b,
                                            &opname,
                                            &named,
                                            &info,
                                            output_buf.len(),
                                            &fn_byte_ranges,
                                            ));
                                            cursor = history.len() - 1;
                                        }
                                        StepResult::Finished { output } => {
                                            if let Some(ref o) = output {
                                                output_buf.push(o.clone());
                                            }
                                            finished = true;
                                            history.push(take_snapshot(
                                                &vm.state,
                                                ip_b,
                                                "END",
                                                "",
                                                &info,
                                                output_buf.len(),
                                            &fn_byte_ranges,
                                            ));
                                            cursor = history.len() - 1;
                                            break;
                                        }
                                        StepResult::Error(msg) => {
                                            error_msg = Some(msg);
                                            history.push(take_snapshot(
                                                &vm.state,
                                                ip_b,
                                                "ERROR",
                                                "",
                                                &info,
                                                output_buf.len(),
                                            &fn_byte_ranges,
                                            ));
                                            cursor = history.len() - 1;
                                            break;
                                        }
                                    }
                                } else {
                                    break;
                                }
                            }
                        }
                        KeyCode::PageUp => {
                            playing = false;
                            cursor = cursor.saturating_sub(20);
                        }

                        // Run to end (records every step so you can rewind)
                        KeyCode::Char('r') => {
                            playing = false;
                            let limit = 1_000_000;
                            let mut count = 0;
                            while !finished
                                && error_msg.is_none()
                                && count < limit
                            {
                                let ip_before = vm.state.current_instruction;
                                let opcode = if ip_before < vm.state.program.len() {
                                    vm.state.program[ip_before]
                                } else {
                                    0
                                };
                                let named = vm.peek_operand_named(opcode, &info);
                                match vm.step_one_debug() {
                                    StepResult::Continue {
                                        opname,
                                        output,
                                    } => {
                                        if let Some(ref o) = output {
                                            output_buf.push(o.clone());
                                        }
                                        history.push(take_snapshot(
                                            &vm.state,
                                            ip_before,
                                            &opname,
                                            &named,
                                            &info,
                                            output_buf.len(),
                                            &fn_byte_ranges,
                                        ));
                                    }
                                    StepResult::Finished { output } => {
                                        if let Some(ref o) = output {
                                            output_buf.push(o.clone());
                                        }
                                        finished = true;
                                        history.push(take_snapshot(
                                            &vm.state,
                                            ip_before,
                                            "END",
                                            "",
                                            &info,
                                            output_buf.len(),
                                            &fn_byte_ranges,
                                        ));
                                    }
                                    StepResult::Error(msg) => {
                                        error_msg = Some(msg);
                                        history.push(take_snapshot(
                                            &vm.state,
                                            ip_before,
                                            "ERROR",
                                            "",
                                            &info,
                                            output_buf.len(),
                                            &fn_byte_ranges,
                                        ));
                                    }
                                }
                                count += 1;
                            }
                            cursor = history.len() - 1;
                        }

                        // Step over (records every step so you can rewind)
                        KeyCode::Char('n') => {
                            playing = false;
                            let target_depth =
                                history[cursor].callstack_depth;
                            let limit = 100_000;
                            let mut count = 0;
                            while !finished
                                && error_msg.is_none()
                                && count < limit
                            {
                                let ip_before = vm.state.current_instruction;
                                let opcode = if ip_before < vm.state.program.len() {
                                    vm.state.program[ip_before]
                                } else {
                                    0
                                };
                                let named = vm.peek_operand_named(opcode, &info);
                                match vm.step_one_debug() {
                                    StepResult::Continue {
                                        opname,
                                        output,
                                    } => {
                                        if let Some(ref o) = output {
                                            output_buf.push(o.clone());
                                        }
                                        history.push(take_snapshot(
                                            &vm.state,
                                            ip_before,
                                            &opname,
                                            &named,
                                            &info,
                                            output_buf.len(),
                                            &fn_byte_ranges,
                                        ));
                                        if vm.state.callstack.len()
                                            <= target_depth
                                        {
                                            break;
                                        }
                                    }
                                    StepResult::Finished { output } => {
                                        if let Some(ref o) = output {
                                            output_buf.push(o.clone());
                                        }
                                        finished = true;
                                        history.push(take_snapshot(
                                            &vm.state,
                                            ip_before,
                                            "END",
                                            "",
                                            &info,
                                            output_buf.len(),
                                            &fn_byte_ranges,
                                        ));
                                        break;
                                    }
                                    StepResult::Error(msg) => {
                                        error_msg = Some(msg);
                                        history.push(take_snapshot(
                                            &vm.state,
                                            ip_before,
                                            "ERROR",
                                            "",
                                            &info,
                                            output_buf.len(),
                                            &fn_byte_ranges,
                                        ));
                                        break;
                                    }
                                }
                                count += 1;
                            }
                            cursor = history.len() - 1;
                        }

                        KeyCode::Home => {
                            playing = false;
                            cursor = 0;
                        }
                        KeyCode::End => {
                            playing = false;
                            cursor = history.len() - 1;
                        }

                        _ => {}
                    }
                }
            } else if playing {
                // No key pressed within timeout → auto-step forward
                if cursor < history.len() - 1 {
                    // Replay from history
                    cursor += 1;
                } else if !finished && error_msg.is_none() {
                    // Execute next instruction
                    let ip_before = vm.state.current_instruction;
                    let opcode = if ip_before < vm.state.program.len() {
                        vm.state.program[ip_before]
                    } else {
                        0
                    };
                    let named = vm.peek_operand_named(opcode, &info);
                    match vm.step_one_debug() {
                        StepResult::Continue { opname, output } => {
                            if let Some(ref o) = output {
                                output_buf.push(o.clone());
                            }
                            history.push(take_snapshot(
                                &vm.state,
                                ip_before,
                                &opname,
                                &named,
                                &info,
                                output_buf.len(),
                                            &fn_byte_ranges,
                            ));
                            cursor = history.len() - 1;
                        }
                        StepResult::Finished { output } => {
                            if let Some(ref o) = output {
                                output_buf.push(o.clone());
                            }
                            finished = true;
                            playing = false;
                            history.push(take_snapshot(
                                &vm.state,
                                ip_before,
                                "END",
                                "",
                                &info,
                                output_buf.len(),
                                            &fn_byte_ranges,
                            ));
                            cursor = history.len() - 1;
                        }
                        StepResult::Error(msg) => {
                            error_msg = Some(msg);
                            playing = false;
                            history.push(take_snapshot(
                                &vm.state,
                                ip_before,
                                "ERROR",
                                "",
                                &info,
                                output_buf.len(),
                                            &fn_byte_ranges,
                            ));
                            cursor = history.len() - 1;
                        }
                    }
                } else {
                    // Nothing left to do, stop playing
                    playing = false;
                }
            }

            let _ = render(
                &mut stdout,
                &history,
                cursor,
                finished,
                &error_msg,
                &output_buf,
                &bc_lines,
                &addr_to_line,
                show_help,
                playing,
                play_speed_ms,
                heap_mode,
                heap_scroll,
                stack_scroll,
                command_mode,
                &command_buf,
                &command_error,
            );
        }

        let _ = stdout.execute(terminal::LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
        Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
// Raylib-compatible debug mode
// ═══════════════════════════════════════════════════════════════════════
//
// When a program uses raylib, the normal TUI debugger cannot work because:
//   1. crossterm's alternate screen hides the raylib GLFW window.
//   2. Step-by-step pausing starves the raylib window of events, causing
//      the OS / window manager to flag it as "not responding" and kill it.
//   3. crossterm raw mode breaks \n → \r\n translation, garbling any
//      output that raylib (or the program) writes via C-level printf.
//
// This mode solves all three problems:
//   • No alternate screen, no raw mode — terminal output stays clean.
//   • The VM runs freely between frame boundaries (raylib::rendering
//     calls), keeping the window responsive.
//   • Periodic status lines show game-relevant metrics (FPS, draw calls,
//     sprites, heap, GC) instead of raw VM internals.

fn run_debug_raylib_mode(
    vm: &mut Vm,
    _info: &DebugInfo,
    rendering_index: Option<u64>,
) -> NovaResult<()> {
    use common::code::Code;
    use std::time::Instant;

    // ── Helper: peek the native index at the current IP ─────────
    fn peek_native_index(vm: &Vm) -> Option<u64> {
        let ip = vm.state.current_instruction;
        let prog = &vm.state.program;
        if ip < prog.len() && prog[ip] == Code::NATIVE {
            let idx_start = ip + 1;
            if idx_start + 8 <= prog.len() {
                Some(u64::from_le_bytes(
                    prog[idx_start..idx_start + 8]
                        .try_into()
                        .unwrap_or_default(),
                ))
            } else {
                None
            }
        } else {
            None
        }
    }

    // ── Banner ──────────────────────────────────────────────────
    eprintln!();
    eprintln!("  Nova Debugger — Raylib Mode");
    eprintln!("  ──────────────────────────────────────────");
    eprintln!("  Raylib program detected.");
    eprintln!("  TUI disabled — game runs at full speed.");
    eprintln!("  Close the window or Ctrl+C to stop.");
    eprintln!();

    let mut frame_count: u64 = 0;
    let mut step_count: u64 = 0;
    let mut last_report = Instant::now();
    let mut frames_since_report: u64 = 0;
    let mut peak_heap: usize = 0;
    let mut peak_draw_calls: usize = 0;
    let mut total_draw_calls: u64 = 0;
    // Track the draw queue size just before each rendering call.
    // After rendering it gets cleared, so we peek right before.
    let mut last_draw_count: usize = 0;
    #[allow(unused_assignments)]
    let _ = last_draw_count; // suppress initial-value-never-read warning

    // Print status every ~2 seconds
    let report_secs: f64 = 2.0;

    let start_time = Instant::now();

    // No raw mode, no alternate screen — just run the VM.
    loop {
        if vm.state.current_instruction >= vm.state.program.len() {
            let elapsed = start_time.elapsed().as_secs_f64();
            eprintln!();
            eprintln!("  ── Program finished ──────────────────────");
            eprintln!("  Frames:      {}", frame_count);
            eprintln!("  Instructions:{}", step_count);
            eprintln!("  Run time:    {:.1}s", elapsed);
            if elapsed > 0.0 {
                eprintln!("  Avg FPS:     {:.1}", frame_count as f64 / elapsed);
            }
            eprintln!("  Peak heap:   {} objects", peak_heap);
            eprintln!("  Peak draws:  {}/frame", peak_draw_calls);
            eprintln!("  Sprites:     {}", vm.state.sprites.len());
            if vm.state.audio_initialized {
                eprintln!(
                    "  Audio:       {} sounds, {} music",
                    vm.state.sounds.len(),
                    vm.state.music.len(),
                );
            }
            eprintln!();
            return Ok(());
        }

        // Check if the next instruction is raylib::rendering
        if let (Some(ri), Some(ni)) = (rendering_index, peek_native_index(vm)) {
            if ni == ri {
                // Capture draw queue size before rendering clears it
                last_draw_count = vm.state.draw_queue.len();
                total_draw_calls += last_draw_count as u64;
                if last_draw_count > peak_draw_calls {
                    peak_draw_calls = last_draw_count;
                }

                // Execute the rendering call so the frame is drawn
                match vm.step_one_debug() {
                    StepResult::Continue { output, .. } => {
                        if let Some(ref o) = output {
                            println!("{}", o);
                        }
                    }
                    StepResult::Finished { output } => {
                        if let Some(ref o) = output {
                            println!("{}", o);
                        }
                        let elapsed = start_time.elapsed().as_secs_f64();
                        eprintln!();
                        eprintln!("  ── Program finished ──────────────────────");
                        eprintln!("  Frames:      {}", frame_count);
                        eprintln!("  Run time:    {:.1}s", elapsed);
                        eprintln!();
                        return Ok(());
                    }
                    StepResult::Error(msg) => {
                        eprintln!();
                        eprintln!("  ── ERROR at frame {} ─────────────────────", frame_count);
                        eprintln!("  {}", msg);
                        eprintln!();
                        return Err(Box::new(NovaError::Runtime {
                            msg: msg.into(),
                        }));
                    }
                }
                step_count += 1;
                frame_count += 1;
                frames_since_report += 1;

                // Track peak heap
                let live = vm.state.memory.live_count();
                if live > peak_heap {
                    peak_heap = live;
                }

                // Periodic status — every ~2s of real time
                let since = last_report.elapsed().as_secs_f64();
                if since >= report_secs {
                    let fps = frames_since_report as f64 / since;
                    let avg_draws = if frames_since_report > 0 {
                        total_draw_calls as f64 / frame_count as f64
                    } else {
                        0.0
                    };
                    eprintln!(
                        "  [frame {:>6}]  {:.0} fps  |  {} draws/frame (avg {:.0})  |  heap: {} live, {} capacity  |  sprites: {}",
                        frame_count,
                        fps,
                        last_draw_count,
                        avg_draws,
                        live,
                        vm.state.memory.heap_capacity(),
                        vm.state.sprites.len(),
                    );
                    last_report = Instant::now();
                    frames_since_report = 0;
                }
                continue;
            }
        }

        // Execute the instruction normally
        match vm.step_one_debug() {
            StepResult::Continue { output, .. } => {
                if let Some(ref o) = output {
                    println!("{}", o);
                }
            }
            StepResult::Finished { output } => {
                if let Some(ref o) = output {
                    println!("{}", o);
                }
                let elapsed = start_time.elapsed().as_secs_f64();
                eprintln!();
                eprintln!("  ── Program finished ──────────────────────");
                eprintln!("  Frames:      {}", frame_count);
                eprintln!("  Instructions:{}", step_count);
                eprintln!("  Run time:    {:.1}s", elapsed);
                eprintln!();
                return Ok(());
            }
            StepResult::Error(msg) => {
                eprintln!();
                eprintln!("  ── ERROR at step {} ──────────────────────", step_count);
                eprintln!("  {}", msg);
                eprintln!();
                return Err(Box::new(NovaError::Runtime {
                    msg: msg.into(),
                }));
            }
        }
        step_count += 1;
    }
}