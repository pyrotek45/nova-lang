use std::collections::HashMap;

use common::{
    code::Asm,
    debug_info::DebugInfo,
};

pub fn new() -> Disassembler {
    Disassembler
}

pub struct Disassembler;

// ── ANSI helpers ──────────────────────────────────────────────────────
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const CYAN: &str = "\x1b[36m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const MAGENTA: &str = "\x1b[35m";
const RED: &str = "\x1b[31m";
const BLUE: &str = "\x1b[34m";
const WHITE: &str = "\x1b[97m";

/// Instruction category for color coding
#[derive(Clone, Copy)]
enum Category {
    Section,
    Memory,
    Stack,
    Arith,
    Compare,
    Control,
    IO,
    Data,
}

fn cat_color(cat: Category) -> &'static str {
    match cat {
        Category::Section => CYAN,
        Category::Memory => GREEN,
        Category::Stack => WHITE,
        Category::Arith => YELLOW,
        Category::Compare => MAGENTA,
        Category::Control => RED,
        Category::IO => BLUE,
        Category::Data => GREEN,
    }
}

// ── Flow arrow in the left margin ────────────────────────────────────
#[derive(Clone)]
struct FlowArrow {
    from: usize,
    to: usize,
    column: usize,
    is_loop: bool,      // backward jump
    is_conditional: bool,
}

impl Disassembler {
    // ══════════════════════════════════════════════════════════════════
    //  Pretty ASM-level disassembly with control flow arrows
    // ══════════════════════════════════════════════════════════════════

    pub fn dis_asm(&self, asm: Vec<Asm>, info: &DebugInfo) {
        // ── Phase 1: Build label → line-index map ────────────────
        let mut label_line: HashMap<u64, usize> = HashMap::new();
        for (i, inst) in asm.iter().enumerate() {
            if let Asm::LABEL(id) = inst {
                label_line.insert(*id, i);
            }
        }

        // ── Phase 2: Collect jump arrows ─────────────────────────
        let mut arrows: Vec<FlowArrow> = Vec::new();
        for (i, inst) in asm.iter().enumerate() {
            let (target_label, conditional) = match inst {
                Asm::JMP(lbl) => (Some(*lbl), false),
                Asm::BJMP(lbl) => (Some(*lbl), false),
                Asm::JUMPIFFALSE(lbl) => (Some(*lbl), true),
                _ => (None, false),
            };
            if let Some(lbl) = target_label {
                if let Some(&target_line) = label_line.get(&lbl) {
                    arrows.push(FlowArrow {
                        from: i,
                        to: target_line,
                        column: 0,
                        is_loop: target_line < i,
                        is_conditional: conditional,
                    });
                }
            }
        }

        // ── Phase 3: Assign non-overlapping columns ──────────────
        arrows.sort_by_key(|a| {
            let lo = a.from.min(a.to);
            let hi = a.from.max(a.to);
            hi - lo
        });
        let max_col = assign_columns(&mut arrows);

        // ── Phase 4: Build margin lookup per line ────────────────
        let n = asm.len();
        let mut margin: Vec<Vec<(usize, char, &str)>> = vec![Vec::new(); n];

        for arrow in &arrows {
            let lo = arrow.from.min(arrow.to);
            let hi = arrow.from.max(arrow.to);
            let col = arrow.column;
            let color = if arrow.is_loop {
                MAGENTA
            } else if arrow.is_conditional {
                YELLOW
            } else {
                CYAN
            };

            // vertical bars in between
            for idx in (lo + 1)..hi {
                margin[idx].push((col, '│', color));
            }

            // corners at endpoints
            if arrow.from < arrow.to {
                // forward (downward) jump
                margin[arrow.from].push((col, '┌', color));
                margin[arrow.to].push((col, '└', color));
            } else {
                // backward (upward / loop) jump
                margin[arrow.from].push((col, '└', color));
                margin[arrow.to].push((col, '┌', color));
            }
        }

        // ── Phase 5: Identify function body ranges ───────────────
        let mut fn_ranges: Vec<(usize, usize, u64, bool)> = Vec::new();
        for (i, inst) in asm.iter().enumerate() {
            match inst {
                Asm::FUNCTION(lbl) => {
                    if let Some(&end) = label_line.get(lbl) {
                        fn_ranges.push((i, end, *lbl, false));
                    }
                }
                Asm::CLOSURE(lbl) => {
                    if let Some(&end) = label_line.get(lbl) {
                        fn_ranges.push((i, end, *lbl, true));
                    }
                }
                _ => {}
            }
        }

        // ── Phase 6: Print ───────────────────────────────────────
        let margin_width = if max_col == 0 { 0 } else { max_col + 1 };
        let line_num_width = format!("{}", n).len().max(4);

        // Header
        println!(
            "\n{}{}══════════════════════════════════════════════════════════════{}",
            BOLD, CYAN, RESET
        );
        println!(
            "{}{}  Nova Disassembly  ({} instructions){}",
            BOLD, CYAN, asm.len(), RESET
        );
        println!(
            "{}{}══════════════════════════════════════════════════════════════{}",
            BOLD, CYAN, RESET
        );
        println!(
            "{}  Legend: {}│{} loop   {}│{} branch   {}│{} jump{}",
            DIM, MAGENTA, RESET, YELLOW, RESET, CYAN, RESET, RESET
        );

        // Globals table (printed at the top for quick reference)
        if !info.global_names.is_empty() {
            println!("\n{}{}  Globals:{}", BOLD, CYAN, RESET);
            let mut globals: Vec<_> = info.global_names.iter().collect();
            globals.sort_by_key(|(k, _)| *k);
            for (idx, name) in &globals {
                println!("    {}[{:>3}]{} {}", DIM, idx, RESET, name);
            }
        }

        println!();

        for (i, inst) in asm.iter().enumerate() {
            let m = render_margin(&margin[i], margin_width);
            let depth = fn_ranges.iter().filter(|(s, e, _, _)| i > *s && i < *e).count();

            // Build the nesting prefix.  Every instruction gets the same
            // column budget so opcodes / content always line up.
            //
            //  depth 0  → ""           (0 chars)
            //  depth 1  → "│  "        (3 chars)
            //  depth 2  → "│  │  "     (6 chars)
            //
            // FUNCTION / CLOSURE use depth+1 with Open cap so the ┌─
            // sits at the same column as body instructions (│  ).
            // end-LABEL uses depth+1 with Close cap for the matching └─.

            match inst {
                Asm::LABEL(id) => {
                    let name = info.label_name(*id);
                    let is_fn_end = fn_ranges.iter().any(|(_, end, lbl, _)| *lbl == *id && *end == i);
                    if is_fn_end {
                        // end-label: close the nesting bracket one level
                        // deeper than this line's depth (to match the
                        // body instructions and the ┌─ opener).
                        let nest = build_nest(depth + 1, NestCap::Close);
                        println!(
                            "{} {}{:>w$}  {}{}end {}{}",
                            m, DIM, i, nest, BOLD, name, RESET,
                            w = line_num_width
                        );
                    } else {
                        let nest = build_nest(depth, NestCap::None);
                        println!(
                            "{} {}{:>w$}  {}{}{}{}:{} {}; label {}{}",
                            m, DIM, i, RESET, nest, BOLD, name, RESET, DIM, id, RESET,
                            w = line_num_width
                        );
                    }
                }
                Asm::FUNCTION(lbl) => {
                    let name = info.label_name(*lbl);
                    let nest = build_nest(depth + 1, NestCap::Open);
                    println!(
                        "{} {}{:>w$}  {}{}fn {}{}",
                        m, DIM, i, nest, BOLD, name, RESET,
                        w = line_num_width
                    );
                }
                Asm::CLOSURE(lbl) => {
                    let name = info.label_name(*lbl);
                    let nest = build_nest(depth + 1, NestCap::Open);
                    println!(
                        "{} {}{:>w$}  {}{}closure {}{}",
                        m, DIM, i, nest, BOLD, name, RESET,
                        w = line_num_width
                    );
                }
                _ => {
                    let (mnemonic, operand, cat) = decode_asm(inst, info);
                    let color = cat_color(cat);
                    let nest = build_nest(depth, NestCap::None);
                    if operand.is_empty() {
                        println!(
                            "{} {}{:>w$}  {}{}{}{:<16}{}",
                            m, DIM, i, RESET, nest, color, mnemonic, RESET,
                            w = line_num_width
                        );
                    } else {
                        println!(
                            "{} {}{:>w$}  {}{}{}{:<16} {}{}{}",
                            m, DIM, i, RESET, nest, color, mnemonic, WHITE, operand, RESET,
                            w = line_num_width
                        );
                    }
                }
            }
        }

        // ── Summary ──────────────────────────────────────────────
        println!(
            "\n{}{}──────────────────────────────────────────────────────────────{}",
            DIM, CYAN, RESET
        );
        let (mut n_fn, mut n_cls, mut n_nat, mut n_lbl) = (0, 0, 0, 0);
        for inst in &asm {
            match inst {
                Asm::FUNCTION(_) => n_fn += 1,
                Asm::CLOSURE(_) => n_cls += 1,
                Asm::NATIVE(_, _) => n_nat += 1,
                Asm::LABEL(_) => n_lbl += 1,
                _ => {}
            }
        }
        println!(
            "  {}functions: {}  {}closures: {}  {}natives: {}  {}labels: {}{}",
            WHITE, n_fn, DIM, n_cls, DIM, n_nat, DIM, n_lbl, RESET
        );
        println!("  {}total instructions: {}{}", WHITE, asm.len(), RESET);

        // Reading guide
        println!("\n{}{}  How to read:{}", BOLD, CYAN, RESET);
        println!("{}  Left margin: control flow arrows show where jumps go.", DIM);
        println!("    {}│{} = loop (backward)   {}│{} = conditional branch   {}│{} = forward jump",
            MAGENTA, RESET, YELLOW, RESET, CYAN, RESET);
        println!("{}  Nesting: {}│  {}= function/closure body boundary", DIM, CYAN, RESET);
        println!("{}  Operands show variable/function names when known.", DIM);
        println!("  {}store/get{}  show local variable names", GREEN, RESET);
        println!("  {}storeg/getg/dcall{}  show global names (functions, variables)", GREEN, RESET);
        println!("  {}jmp/bjmp/jif{}  show target label names with arrows{}", RED, RESET, RESET);
        println!();
    }
}

// ── Nesting prefix builder ──────────────────────────────────────────
// Produces a fixed-width string of `depth` × 3 visible characters so
// the opcode column always starts at the same position.
//
//   NestCap::None  → "│  │  │  "   (all bars)
//   NestCap::Open  → "│  │  ┌─ "   (last slot = open corner)
//   NestCap::Close → "│  │  └─ "   (last slot = close corner)
//
// When depth is 0 the result is "" regardless of cap.

enum NestCap {
    None,
    Open,
    Close,
}

fn build_nest(depth: usize, cap: NestCap) -> String {
    if depth == 0 {
        return String::new();
    }
    let mut out = String::new();
    // All levels except the last get a normal bar
    for _ in 0..depth.saturating_sub(1) {
        out.push_str(CYAN);
        out.push_str("│  ");
    }
    // Last level depends on cap
    out.push_str(CYAN);
    match cap {
        NestCap::None  => out.push_str("│  "),
        NestCap::Open  => out.push_str("┌─ "),
        NestCap::Close => out.push_str("└─ "),
    }
    out.push_str(RESET);
    out
}

// ── Column assignment ────────────────────────────────────────────────
fn assign_columns(arrows: &mut [FlowArrow]) -> usize {
    let mut max_col = 0usize;
    let mut used: Vec<(usize, usize, usize)> = Vec::new();

    for arrow in arrows.iter_mut() {
        let lo = arrow.from.min(arrow.to);
        let hi = arrow.from.max(arrow.to);
        let mut col = 0;
        loop {
            let conflicts = used.iter().any(|&(ulo, uhi, ucol)| {
                ucol == col && !(hi < ulo || lo > uhi)
            });
            if !conflicts { break; }
            col += 1;
        }
        arrow.column = col;
        if col > max_col { max_col = col; }
        used.push((lo, hi, col));
    }
    max_col
}

// ── Render the margin for one line ──────────────────────────────────
fn render_margin(
    chars_at_line: &[(usize, char, &str)],
    total_slots: usize,
) -> String {
    if total_slots == 0 {
        return String::new();
    }
    let mut slots: Vec<Option<(char, &str)>> = vec![None; total_slots];
    for &(col, ch, color) in chars_at_line {
        if col < total_slots {
            slots[col] = Some((ch, color));
        }
    }
    let mut out = String::new();
    for i in (0..total_slots).rev() {
        if let Some((ch, color)) = slots[i] {
            out.push_str(color);
            out.push(ch);
            out.push_str(RESET);
        } else {
            out.push(' ');
        }
    }
    out
}

// ── Decode ASM instruction to (mnemonic, operand, category) ─────────
fn decode_asm(inst: &Asm, info: &DebugInfo) -> (&'static str, String, Category) {
    match inst {
        Asm::EXIT => ("exit", String::new(), Category::Control),
        Asm::ALLOCGLOBBALS(v) => ("alloc_global", format!("{}", v), Category::Memory),
        Asm::ALLOCLOCALS(v) => ("alloc_locals", format!("{}", v), Category::Memory),
        Asm::OFFSET(args, locals) => (
            "offset",
            format!("args={}, locals={}", args, locals),
            Category::Memory,
        ),
        Asm::STORE(v) => {
            let name = info.local_name(0, *v).unwrap_or_default();
            if name.is_empty() {
                ("store", format!("local[{}]", v), Category::Memory)
            } else {
                ("store", format!("{} {}(local[{}]){}", name, DIM, v, RESET), Category::Memory)
            }
        }
        Asm::STOREGLOBAL(v) => {
            let name = info.global_name(*v);
            ("storeg", format!("{} {}(global[{}]){}", name, DIM, v, RESET), Category::Memory)
        }
        Asm::GET(v) => {
            let name = info.local_name(0, *v).unwrap_or_default();
            if name.is_empty() {
                ("get", format!("local[{}]", v), Category::Memory)
            } else {
                ("get", format!("{} {}(local[{}]){}", name, DIM, v, RESET), Category::Memory)
            }
        }
        Asm::GETGLOBAL(v) => {
            let name = info.global_name(*v);
            ("getg", format!("{} {}(global[{}]){}", name, DIM, v, RESET), Category::Memory)
        }
        Asm::STACKREF(v) => ("stackref", format!("{}", v), Category::Memory),
        Asm::ASSIGN => ("assign", String::new(), Category::Memory),
        Asm::FUNCTION(_) => ("function", String::new(), Category::Section),
        Asm::CLOSURE(_) => ("closure", String::new(), Category::Section),
        Asm::RET(true) => ("ret", "(value)".into(), Category::Control),
        Asm::RET(false) => ("ret", String::new(), Category::Control),
        Asm::LABEL(v) => ("label", format!("{}", v), Category::Section),
        Asm::JUMPIFFALSE(v) => {
            let name = info.label_name(*v);
            ("jif", format!("→ {}", name), Category::Control)
        }
        Asm::JMP(v) => {
            let name = info.label_name(*v);
            ("jmp", format!("→ {}", name), Category::Control)
        }
        Asm::BJMP(v) => {
            let name = info.label_name(*v);
            ("bjmp", format!("↺ {}", name), Category::Control)
        }
        Asm::INTEGER(v) => ("push_i", format!("{}", v), Category::Stack),
        Asm::BOOL(true) => ("push_true", String::new(), Category::Stack),
        Asm::BOOL(false) => ("push_false", String::new(), Category::Stack),
        Asm::STRING(v) => {
            let display = if v.len() > 40 {
                format!("\"{}...\"", &v[..37])
            } else {
                format!("\"{}\"", v)
            };
            ("push_s", display, Category::Stack)
        }
        Asm::LIST(v) => ("newlist", format!("size={}", v), Category::Data),
        Asm::FLOAT(v) => ("push_f", format!("{}", v), Category::Stack),
        Asm::Char(v) => ("push_c", format!("'{}'", v), Category::Stack),
        Asm::IADD => ("iadd", String::new(), Category::Arith),
        Asm::ISUB => ("isub", String::new(), Category::Arith),
        Asm::IDIV => ("idiv", String::new(), Category::Arith),
        Asm::IMUL => ("imul", String::new(), Category::Arith),
        Asm::IMODULO => ("imod", String::new(), Category::Arith),
        Asm::FADD => ("fadd", String::new(), Category::Arith),
        Asm::FSUB => ("fsub", String::new(), Category::Arith),
        Asm::FDIV => ("fdiv", String::new(), Category::Arith),
        Asm::FMUL => ("fmul", String::new(), Category::Arith),
        Asm::ILSS => ("ilss", String::new(), Category::Compare),
        Asm::IGTR => ("igtr", String::new(), Category::Compare),
        Asm::FLSS => ("flss", String::new(), Category::Compare),
        Asm::FGTR => ("fgtr", String::new(), Category::Compare),
        Asm::EQUALS => ("equals", String::new(), Category::Compare),
        Asm::NOT => ("not", String::new(), Category::Compare),
        Asm::NEG => ("neg", String::new(), Category::Arith),
        Asm::AND => ("and", String::new(), Category::Compare),
        Asm::OR => ("or", String::new(), Category::Compare),
        Asm::CONCAT => ("concat", String::new(), Category::IO),
        Asm::DUP => ("dup", String::new(), Category::Stack),
        Asm::POP => ("pop", String::new(), Category::Stack),
        Asm::PRINT => ("print", String::new(), Category::IO),
        Asm::DCALL(v) => {
            let name = info.global_name(*v);
            ("dcall", format!("{}", name), Category::Control)
        }
        Asm::TCALL(v) => {
            let name = info.global_name(*v);
            ("tcall", format!("{}", name), Category::Control)
        }
        Asm::CALL => ("call", String::new(), Category::Control),
        Asm::PIN(_) => ("pindex", String::new(), Category::Data),
        Asm::LIN(_) => ("lindex", String::new(), Category::Data),
        Asm::NATIVE(v, _) => {
            let name = info.native_name(*v);
            ("native", format!("{}()", name), Category::IO)
        }
        Asm::NONE => ("none", String::new(), Category::Stack),
        Asm::ISSOME => ("issome", String::new(), Category::Compare),
        Asm::UNWRAP(_) => ("unwrap", String::new(), Category::Stack),
        Asm::FREE => ("free", String::new(), Category::Memory),
        Asm::CLONE => ("clone", String::new(), Category::Memory),
        Asm::NEWSTRUCT(v) => ("newstruct", format!("fields={}", v), Category::Data),
        Asm::GETF(_) => ("getf", String::new(), Category::Data),
        Asm::PINF(_) => ("setf", String::new(), Category::Data),
        Asm::LEN => ("len", String::new(), Category::IO),
        Asm::ERROR(_) => ("error", String::new(), Category::Control),
    }
}
