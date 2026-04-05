pub mod memory_manager;
pub mod state;
pub type CallBack = fn(state: &mut state::State) -> NovaResult<()>;

use std::{
    borrow::Cow,
    collections::HashMap,
    io::{self, Write},
    process::exit,
    time::Duration,
};

use common::{
    code::{self as code, Code},
    error::{NovaError, NovaResult},
    fileposition::FilePosition,
};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    style::{Attribute, Color, Print, SetAttribute, SetForegroundColor},
    terminal::{self, ClearType},
    ExecutableCommand, QueueableCommand,
};

use memory_manager::{Object, ObjectType, VmData};
use modulo::Mod;
use state::{CallType, State};

#[derive(Debug, Clone)]
pub struct Vm {
    pub runtime_errors_table: HashMap<usize, FilePosition>,
    pub native_functions: Vec<CallBack>,
    pub state: State,
}

pub fn new() -> Vm {
    Vm {
        native_functions: vec![],
        state: State::new(),
        runtime_errors_table: HashMap::default(),
    }
}
pub struct VmTask {
    pub vm: Vm,
    pub is_done: bool,
}

/// Result of executing a single debug step.
enum StepResult {
    Continue {
        opname: String,
        output: Option<String>,
    },
    Finished {
        output: Option<String>,
    },
    Error(String),
}

impl Vm {
    /// Look up the source position for the current instruction pointer.
    /// Returns `RuntimeWithPos` if a position is available, otherwise `Runtime`.
    #[inline]
    fn runtime_error(&self, msg: impl Into<String>) -> Box<NovaError> {
        let msg: Cow<'static, str> = Cow::Owned(msg.into());
        if let Some(pos) = self
            .runtime_errors_table
            .get(&self.state.current_instruction)
        {
            Box::new(NovaError::RuntimeWithPos {
                msg,
                position: pos.clone(),
            })
        } else {
            Box::new(NovaError::Runtime { msg })
        }
    }

    #[inline(always)]
    pub fn run(&mut self) -> NovaResult<()> {
        loop {
            match self.state.next_instruction() {
                Code::RET => {
                    let with_return = self.state.next_instruction();
                    if let Some(destination) = self.state.callstack.pop() {
                        if with_return == 1 {
                            self.state.deallocate_registers_with_return()?;
                        } else {
                            self.state.deallocate_registers()?;
                        }
                        match destination {
                            CallType::Function(target) => {
                                self.state.goto(target);
                            }
                            CallType::Closure { target, closure } => {
                                self.state.goto(target);
                                self.state.memory.dec(closure);
                            }
                        }
                    } else {
                        break;
                    }
                }
                Code::ERROR => {
                    return Err(self.runtime_error("Error"));
                }
                Code::EXIT => exit(0),
                instruction => self.dispatch(instruction)?,
            }
        }
        Ok(())
    }

    fn dispatch(&mut self, instruction: u8) -> NovaResult<()> {
        match instruction {
            Code::ISSOME => match self.state.memory.stack.pop() {
                Some(VmData::None) => self.state.memory.stack.push(VmData::Bool(false)),
                None => (),
                _ => self.state.memory.stack.push(VmData::Bool(true)),
            },

            Code::UNWRAP => {
                if let Some(VmData::None) = self.state.memory.stack.last() {
                    return Err(self.runtime_error("Tried to unwrap a None value"));
                }
            }

            Code::DUP => {
                let top = self
                    .state
                    .memory
                    .stack
                    .last()
                    .ok_or_else(|| self.runtime_error("DUP: stack is empty"))?;
                self.state.memory.stack.push(*top);
            }

            Code::POP => {
                self.state.memory.pop();
            }

            Code::NATIVE => {
                // Save current_instruction AFTER the opcode byte (+1 from next_instruction)
                // but BEFORE reading the 8-byte index — this matches the offset stored in
                // runtime_errors_table by the assembler (same convention as UNWRAP/GETF/ERROR).
                let native_ip = self.state.current_instruction;
                let index = u64::from_le_bytes(self.state.next_arr());

                // Inhibit GC while the native function runs.  Native helpers
                // use raw stack.pop() (no dec), so their arguments are temporarily
                // off the stack and invisible to collect_cycles.  Without this
                // guard an allocation inside the native could trigger a sweep
                // that frees the very objects the native is working with.
                self.state.memory.gc_inhibit();
                let result = self.native_functions[index as usize](&mut self.state);
                self.state.memory.gc_release();

                if let Err(error) = result {
                    // If the native error is already positioned, forward it as-is.
                    // Otherwise try to attach a source position from the table.
                    if matches!(*error, NovaError::RuntimeWithPos { .. }) {
                        return Err(error);
                    }
                    if let Some(pos) = self.runtime_errors_table.get(&native_ip) {
                        let msg = match *error {
                            NovaError::Runtime { ref msg } => msg.clone(),
                            _ => std::borrow::Cow::Borrowed("native function error"),
                        };
                        return Err(Box::new(NovaError::RuntimeWithPos {
                            msg,
                            position: pos.clone(),
                        }));
                    }
                    return Err(error);
                }
            }

            // sets up the stack with empty values for use later with local variables
            Code::ALLOCATEGLOBAL => {
                let allocations = u32::from_le_bytes(self.state.next_arr());
                self.state.alloc_locals(allocations as usize);
            }
            // sets up the stack with empty values for use later with local variables
            Code::ALLOCLOCALS => {
                let allocations = u32::from_le_bytes(self.state.next_arr());
                self.state.alloc_locals(allocations as usize);
            }
            // sets up the stack with empty values for use later with local variables
            Code::OFFSET => {
                let offset = u32::from_le_bytes(self.state.next_arr());
                let locals = u32::from_le_bytes(self.state.next_arr());
                self.state.offset_locals(offset as usize, locals as usize);
            }

            // pushes a constant integer to the stack
            Code::INTEGER => {
                let int = i64::from_le_bytes(self.state.next_arr());
                self.state.memory.stack.push(VmData::Int(int));
            }

            // takes item and stores it into stack at location
            // with offset
            Code::STORE => {
                let index = u32::from_le_bytes(self.state.next_arr());
                self.state
                    .memory
                    .pop_store_index(self.state.offset + index as usize);
            }

            // gets the data from a local index in the stack
            // from offset
            Code::GET => {
                let index = u32::from_le_bytes(self.state.next_arr());
                self.state
                    .memory
                    .stack_index_to_stack(self.state.offset + index as usize);
            }

            // i think you can figure this one out
            Code::PRINT => {
                let item = self
                    .state
                    .memory
                    .stack
                    .pop()
                    .ok_or_else(|| self.runtime_error("PRINT: stack is empty"))?;
                match item {
                    VmData::Int(i) => print!("{}", i),
                    VmData::Float(f) => print!("{}", f),
                    VmData::Bool(b) => print!("{}", b),
                    VmData::Char(c) => print!("{}", c),
                    VmData::Object(l) => {
                        let l = self.state.memory.print_heap_object(l, 0);
                        print!("{}", l)
                    }
                    VmData::None => print!("None"),
                    VmData::Function(f) => {
                        print!("<function: {}>", f)
                    }
                    VmData::StackAddress(s) => {
                        let s = self.state.memory.print_heap_object(s, 0);
                        print!("{}", s)
                    }
                }
                let _ = io::stdout().flush();

                self.state.memory.dec_value(item);
            }

            Code::FADD => {
                let (Some(VmData::Float(v1)), Some(VmData::Float(v2))) =
                    (self.state.memory.stack.pop(), self.state.memory.stack.pop())
                else {
                    return Err(
                        self.runtime_error("Float addition (+): expected two Float values on the stack")
                    );
                };
                let result = v1 + v2;
                self.state.memory.stack.push(VmData::Float(result))
            }

            Code::FSUB => {
                let (Some(VmData::Float(v1)), Some(VmData::Float(v2))) =
                    (self.state.memory.stack.pop(), self.state.memory.stack.pop())
                else {
                    return Err(
                        self.runtime_error("Float subtraction (-): expected two Float values on the stack")
                    );
                };
                let result = v2 - v1;
                self.state.memory.stack.push(VmData::Float(result))
            }

            Code::FMUL => {
                let (Some(VmData::Float(v1)), Some(VmData::Float(v2))) =
                    (self.state.memory.stack.pop(), self.state.memory.stack.pop())
                else {
                    return Err(
                        self.runtime_error("Float multiplication (*): expected two Float values on the stack")
                    );
                };
                let result = v1 * v2;
                self.state.memory.stack.push(VmData::Float(result))
            }

            Code::FDIV => {
                let (Some(VmData::Float(v1)), Some(VmData::Float(v2))) =
                    (self.state.memory.stack.pop(), self.state.memory.stack.pop())
                else {
                    return Err(
                        self.runtime_error("Float division (/): expected two Float values on the stack")
                    );
                };
                let result = v2 / v1;
                self.state.memory.stack.push(VmData::Float(result))
            }

            Code::IADD => {
                let (Some(VmData::Int(v1)), Some(VmData::Int(v2))) =
                    (self.state.memory.stack.pop(), self.state.memory.stack.pop())
                else {
                    return Err(
                        self.runtime_error("Integer addition (+): expected two Int values on the stack")
                    );
                };
                let result = v1.checked_add(v2).ok_or_else(|| {
                    self.runtime_error("Integer addition (+): overflow")
                })?;
                self.state.memory.stack.push(VmData::Int(result))
            }

            Code::ISUB => {
                let (Some(VmData::Int(v1)), Some(VmData::Int(v2))) =
                    (self.state.memory.stack.pop(), self.state.memory.stack.pop())
                else {
                    return Err(
                        self.runtime_error("Integer subtraction (-): expected two Int values on the stack")
                    );
                };
                let result = v2.checked_sub(v1).ok_or_else(|| {
                    self.runtime_error("Integer subtraction (-): overflow")
                })?;
                self.state.memory.stack.push(VmData::Int(result))
            }

            Code::IMUL => {
                let (Some(VmData::Int(v1)), Some(VmData::Int(v2))) =
                    (self.state.memory.stack.pop(), self.state.memory.stack.pop())
                else {
                    return Err(
                        self.runtime_error("Integer multiplication (*): expected two Int values on the stack")
                    );
                };
                let result = v1.checked_mul(v2).ok_or_else(|| {
                    self.runtime_error("Integer multiplication (*): overflow")
                })?;
                self.state.memory.stack.push(VmData::Int(result))
            }

            Code::IDIV => {
                let (Some(VmData::Int(v1)), Some(VmData::Int(v2))) =
                    (self.state.memory.stack.pop(), self.state.memory.stack.pop())
                else {
                    return Err(
                        self.runtime_error("Integer division (/): expected two Int values on the stack")
                    );
                };
                let result = v2.checked_div(v1).ok_or_else(|| {
                    self.runtime_error("Integer division (/): division by zero")
                })?;
                self.state.memory.stack.push(VmData::Int(result))
            }

            Code::STOREGLOBAL => {
                let index = u32::from_le_bytes(self.state.next_arr());
                let item = self
                    .state
                    .memory
                    .stack
                    .pop()
                    .ok_or_else(|| self.runtime_error("STOREGLOBAL: stack is empty"))?;
                self.state.memory.stack[index as usize] = item;
            }

            Code::FUNCTION => {
                self.state
                    .memory
                    .stack
                    .push(VmData::Function(self.state.current_instruction + 4));

                let jump = u32::from_le_bytes(self.state.next_arr());

                self.state.current_instruction += jump as usize;
            }

            Code::CLOSURE => {
                let Some(VmData::Int(size)) = self.state.memory.stack.pop() else {
                    return Err(self.runtime_error("CLOSURE: expected integer size on stack"));
                };

                let mut myarray = vec![];
                for _ in 0..size {
                    let value = self.state.memory.stack.pop().ok_or_else(|| {
                        self.runtime_error("CLOSURE: not enough values on stack for capture")
                    })?;
                    myarray.push(value);
                }

                myarray.reverse();

                // Inhibit GC: captured values are off the stack but still
                // referenced by myarray — protect them during allocate.
                self.state.memory.gc_inhibit();
                let closure = self
                    .state
                    .memory
                    .allocate(Object::closure(self.state.current_instruction + 4, myarray));

                self.state.memory.stack.push(VmData::Object(closure));
                self.state.memory.gc_release();
                let jump = u32::from_le_bytes(self.state.next_arr());
                self.state.current_instruction += jump as usize;
            }

            Code::DIRECTCALL => {
                self.state
                    .callstack
                    .push(CallType::Function(self.state.current_instruction + 4));

                let index = u32::from_le_bytes(self.state.next_arr());

                let callee = self.state.memory.stack[index as usize];

                match callee {
                    VmData::Function(target) => {
                        self.state.goto(target);
                    }
                    VmData::Object(index) => {
                        if let Some(object) = self.state.memory.ref_from_heap(index) {
                            if let Some((target, env)) = object.as_closure() {
                                for captured in env {
                                    self.state.memory.push(captured);
                                }
                                self.state.goto(target);
                            }
                        }
                    }
                    _ => {
                        return Err(
                            self.runtime_error(format!("DIRECTCALL: cannot call value {}", callee))
                        );
                    }
                }
            }

            Code::CALL => {
                let Some(callee) = self.state.memory.stack.pop() else {
                    return Err(self.runtime_error("CALL: stack is empty"));
                };
                match callee {
                    VmData::Object(index) => {
                        if let Some(object) = self.state.memory.ref_from_heap(index) {
                            if let Some((target, env)) = object.as_closure() {
                                for captured in env {
                                    self.state.memory.push(captured);
                                }
                                self.state.callstack.push(CallType::Closure {
                                    target: self.state.current_instruction,
                                    closure: index,
                                });
                                self.state.goto(target);
                            }
                        }
                    }
                    VmData::Function(target) => {
                        self.state
                            .callstack
                            .push(CallType::Function(self.state.current_instruction));
                        self.state.goto(target);
                    }
                    a => {
                        return Err(self.runtime_error(format!("CALL: cannot call value {}", a)));
                    }
                }
            }

            Code::ILSS => {
                let (Some(VmData::Int(v1)), Some(VmData::Int(v2))) =
                    (self.state.memory.stack.pop(), self.state.memory.stack.pop())
                else {
                    return Err(
                        self.runtime_error("Integer less-than (<): expected two Int values on the stack")
                    );
                };
                let result = v2 < v1;
                self.state.memory.stack.push(VmData::Bool(result))
            }

            Code::IGTR => match (self.state.memory.stack.pop(), self.state.memory.stack.pop()) {
                (Some(VmData::Int(v1)), Some(VmData::Int(v2))) => {
                    let result = v2 > v1;
                    self.state.memory.stack.push(VmData::Bool(result))
                }
                (a, b) => {
                    return Err(self.runtime_error(format!(
                        "IGTR: expected two integers, got {} and {}",
                        a.unwrap_or(VmData::None),
                        b.unwrap_or(VmData::None)
                    )));
                }
            },

            Code::FLSS => {
                let (Some(VmData::Float(v1)), Some(VmData::Float(v2))) =
                    (self.state.memory.stack.pop(), self.state.memory.stack.pop())
                else {
                    return Err(
                        self.runtime_error("Float less-than (<): expected two Float values on the stack")
                    );
                };
                let result = v2 < v1;
                self.state.memory.stack.push(VmData::Bool(result))
            }

            Code::FGTR => {
                let (Some(VmData::Float(v1)), Some(VmData::Float(v2))) =
                    (self.state.memory.stack.pop(), self.state.memory.stack.pop())
                else {
                    return Err(
                        self.runtime_error("Float greater-than (>): expected two Float values on the stack")
                    );
                };
                let result = v2 > v1;
                self.state.memory.stack.push(VmData::Bool(result))
            }

            Code::JMP => {
                let jump = u32::from_le_bytes(self.state.next_arr());
                self.state.current_instruction += jump as usize;
            }
            Code::BJMP => {
                let jump = u32::from_le_bytes(self.state.next_arr());
                self.state.current_instruction -= jump as usize;
            }
            Code::JUMPIFFALSE => {
                let jump = u32::from_le_bytes(self.state.next_arr());
                let value = self
                    .state
                    .memory
                    .stack
                    .pop()
                    .ok_or_else(|| self.runtime_error("JUMPIFFALSE: stack is empty"))?;
                if let VmData::Bool(test) = value {
                    if !test {
                        self.state.current_instruction += jump as usize;
                    }
                } else {
                    return Err(
                        self.runtime_error(format!("JUMPIFFALSE: expected Bool, got {}", value))
                    );
                }
            }

            Code::TRUE => {
                self.state.memory.stack.push(VmData::Bool(true));
            }

            Code::FALSE => {
                self.state.memory.stack.push(VmData::Bool(false));
            }

            Code::EQUALS => {
                let (Some(v1), Some(v2)) =
                    (self.state.memory.stack.pop(), self.state.memory.stack.pop())
                else {
                    return Err(
                        self.runtime_error("Equality check (==): expected two values on the stack")
                    );
                };
                let equal = self.deep_equal(&v1, &v2);
                self.state.memory.stack.push(VmData::Bool(equal));
            }

            Code::NOT => match self.state.memory.stack.pop() {
                Some(VmData::Bool(b)) => {
                    self.state.memory.stack.push(VmData::Bool(!b));
                }
                Some(other) => {
                    return Err(self.runtime_error(format!(
                        "Logical not (!): expected a Bool, got {}",
                        other
                    )));
                }
                None => {
                    return Err(self.runtime_error("Logical not (!): stack is empty"));
                }
            },

            Code::AND => match (self.state.memory.stack.pop(), self.state.memory.stack.pop()) {
                (Some(VmData::Bool(v1)), Some(VmData::Bool(v2))) => {
                    self.state.memory.stack.push(VmData::Bool(v1 && v2))
                }
                (Some(a), Some(b)) => {
                    return Err(self.runtime_error(format!(
                        "Logical and (&&): expected two Bool values, got {} and {}",
                        b, a
                    )));
                }
                _ => {
                    return Err(self.runtime_error("Logical and (&&): not enough values on the stack"));
                }
            },

            Code::OR => match (self.state.memory.stack.pop(), self.state.memory.stack.pop()) {
                (Some(VmData::Bool(v1)), Some(VmData::Bool(v2))) => {
                    self.state.memory.stack.push(VmData::Bool(v1 || v2))
                }
                (Some(a), Some(b)) => {
                    return Err(self.runtime_error(format!(
                        "Logical or (||): expected two Bool values, got {} and {}",
                        b, a
                    )));
                }
                _ => {
                    return Err(self.runtime_error("Logical or (||): not enough values on the stack"));
                }
            },

            Code::NEG => {
                let value = self
                    .state
                    .memory
                    .stack
                    .pop()
                    .ok_or_else(|| self.runtime_error("Negation (-): stack is empty"))?;
                match value {
                    VmData::Int(v) => self.state.memory.stack.push(VmData::Int(-v)),
                    VmData::Float(v) => self.state.memory.stack.push(VmData::Float(-v)),
                    other => {
                        return Err(self.runtime_error(format!(
                            "Negation (-): expected Int or Float, got {}",
                            other
                        )));
                    }
                }
            }

            Code::IMODULO => {
                let (Some(VmData::Int(v1)), Some(VmData::Int(v2))) =
                    (self.state.memory.stack.pop(), self.state.memory.stack.pop())
                else {
                    return Err(
                        self.runtime_error("Modulo (%): expected two Int values on the stack")
                    );
                };
                let result = v2.modulo(v1);
                self.state.memory.stack.push(VmData::Int(result))
            }

            Code::ASSIGN => {
                let (destination, value) =
                    match (self.state.memory.stack.pop(), self.state.memory.stack.pop()) {
                        (Some(dest), Some(val)) => (dest, val),
                        _ => {
                            return Err(
                                self.runtime_error("Assignment (=): not enough values on the stack")
                            )
                        }
                    };
                if let VmData::Int(index) = destination {
                    let target_index = self.state.offset + index as usize;
                    self.state.memory.store(target_index, value);
                } else {
                    return Err(self.runtime_error(format!(
                        "Assignment (=): expected an Int destination, got {}",
                        destination
                    )));
                }
            }

            Code::NEWLIST => {
                let size = u64::from_le_bytes(self.state.next_arr());
                let mut myarray = vec![];
                for _ in 0..size {
                    let value =
                        self.state.memory.stack.pop().ok_or_else(|| {
                            self.runtime_error("NEWLIST: not enough values on stack")
                        })?;
                    myarray.push(value);
                }
                myarray.reverse();
                // Inhibit GC: the popped items are off the stack but still
                // referenced by myarray — protect them during allocate.
                self.state.memory.gc_inhibit();
                self.state.memory.push_list(myarray);
                self.state.memory.gc_release();
            }

            Code::FLOAT => {
                let fl = f64::from_le_bytes(self.state.next_arr());
                self.state.memory.stack.push(VmData::Float(fl));
            }

            Code::GETGLOBAL => {
                let index = u32::from_le_bytes(self.state.next_arr());
                self.state.memory.stack_index_to_stack(index as usize);
            }

            Code::STRING => {
                let mut string = vec![];
                let size = u64::from_le_bytes(self.state.next_arr());
                for _ in 0..size {
                    string.push(self.state.next_instruction());
                }
                let string = match String::from_utf8(string) {
                    Ok(ok) => ok,
                    Err(e) => {
                        return Err(
                            self.runtime_error(format!("STRING: invalid UTF-8 bytes: {}", e))
                        );
                    }
                };

                self.state.memory.push_string(string);
            }

            Code::CHAR => {
                let char = self.state.next_instruction() as char;
                self.state.memory.stack.push(VmData::Char(char));
            }

            Code::CLONE => {
                self.state.memory.clone_top();
            }

            Code::NONE => {
                self.state.memory.stack.push(VmData::None);
            }

            Code::LEN => {
                let value = self
                    .state
                    .memory
                    .stack
                    .pop()
                    .ok_or_else(|| self.runtime_error("len: stack is empty"))?;
                match value {
                    VmData::Object(index) => {
                        let len = {
                            let obj =
                                self.state.memory.ref_from_heap(index).ok_or_else(|| {
                                    self.runtime_error(
                                        "len: invalid heap reference (object was freed)",
                                    )
                                })?;
                            obj.data.len() as i64
                        };
                        self.state.memory.dec(index);
                        self.state.memory.stack.push(VmData::Int(len));
                    }
                    other => {
                        return Err(self.runtime_error(format!(
                            "len: expected a List, String, or Tuple, got {}",
                            other
                        )));
                    }
                }
            }

            Code::CONCAT => match (self.state.memory.stack.pop(), self.state.memory.stack.pop()) {
                (Some(VmData::Object(index1)), Some(VmData::Object(index2))) => {
                    // Inhibit GC: both objects were raw-popped and are invisible
                    // to collect_cycles until we finish allocating the result.
                    self.state.memory.gc_inhibit();
                    let (new_object_type, new_data) = {
                        let object1 = self.state.memory.ref_from_heap(index2).ok_or_else(|| {
                            self.runtime_error("Concatenation (++): left operand is an invalid heap reference")
                        })?;
                        let object2 = self.state.memory.ref_from_heap(index1).ok_or_else(|| {
                            self.runtime_error("Concatenation (++): right operand is an invalid heap reference")
                        })?;
                        let total_len = object1.data.len() + object2.data.len();
                        let mut combined = Vec::with_capacity(total_len);
                        combined.extend_from_slice(&object1.data);
                        combined.extend_from_slice(&object2.data);
                        (object1.object_type.clone(), combined)
                    };
                    let new_object = Object::new(new_object_type, new_data);
                    let result = self.state.memory.allocate(new_object);
                    self.state.memory.dec(index1);
                    self.state.memory.dec(index2);
                    self.state.memory.stack.push(VmData::Object(result));
                    self.state.memory.gc_release();
                }
                _ => {
                    return Err(
                        self.runtime_error("Concatenation (++): expected two List, String, or Tuple objects")
                    );
                }
            },

            Code::LINDEX => {
                let (Some(array), Some(index)) =
                    (self.state.memory.stack.pop(), self.state.memory.stack.pop())
                else {
                    return Err(self.runtime_error("Index: not enough values on stack"));
                };
                match (array, index) {
                    (VmData::Object(object), VmData::Int(index)) => {
                        let item = {
                            let heap_object =
                                self.state.memory.ref_from_heap(object).ok_or_else(|| {
                                    self.runtime_error(
                                        "Index: invalid heap reference (object was freed)",
                                    )
                                })?;
                            let type_name = match &heap_object.object_type {
                                ObjectType::List => "List",
                                ObjectType::Tuple => "Tuple",
                                ObjectType::String => "String",
                                _ => "Object",
                            };
                            let len = heap_object.data.len() as i64;
                            // Python-style negative indexing: -1 → last, -2 → second-to-last, etc.
                            let resolved = if index < 0 { index + len } else { index };
                            if resolved < 0 || resolved >= len {
                                return Err(self.runtime_error(format!(
                                    "Index out of bounds: index is {} but {} length is {}",
                                    index, type_name, len
                                )));
                            }
                            heap_object.data[resolved as usize]
                        };
                        self.state.memory.push(item);
                        self.state.memory.dec(object);
                    }
                    (a, b) => {
                        return Err(self.runtime_error(format!(
                            "Index: expected (Object, Int), got ({}, {})",
                            a, b
                        )));
                    }
                }
            }

            Code::PINDEX => {
                let (Some(array), Some(index), Some(value)) = (
                    self.state.memory.stack.pop(),
                    self.state.memory.stack.pop(),
                    self.state.memory.stack.pop(),
                ) else {
                    return Err(self.runtime_error("PINDEX: not enough values on stack"));
                };

                match (array, index, value) {
                    (VmData::Object(object), VmData::Int(index), value) => {
                        let (resolved, old_value) = {
                            let heap_object =
                                self.state.memory.ref_from_heap(object).ok_or_else(|| {
                                    self.runtime_error(
                                        "PINDEX: invalid heap reference (object was freed)",
                                    )
                                })?;
                            let len = heap_object.data.len() as i64;
                            // Python-style negative indexing
                            let resolved = if index < 0 { index + len } else { index };
                            if resolved < 0 || resolved >= len {
                                return Err(self.runtime_error(format!(
                                    "Index out of bounds: index is {} but length is {}",
                                    index, len
                                )));
                            }
                            let old = heap_object.data[resolved as usize];
                            (resolved as usize, old)
                        };
                        self.state.memory.dec_value(old_value);
                        if let Some(heap_object) = self.state.memory.ref_from_heap_mut(object) {
                            heap_object.data[resolved] = value;
                        }
                        self.state.memory.dec(object);
                    }
                    (a, b, c) => {
                        return Err(self.runtime_error(format!(
                            "PINDEX: expected (Object, Int, value), got ({}, {}, {})",
                            a, b, c
                        )));
                    }
                }
            }

            // unsupported for now

            // Code::STACKREF => {
            //     let index = u32::from_le_bytes(self.state.next_arr());
            //     self.state
            //         .memory
            //         .stack
            //         .push(VmData::StackAddress(index as usize));
            // }

            // Code::TAILCALL => {
            //     let index = u32::from_le_bytes(self.state.next_arr());
            //     if let VmData::Function(target) = self.state.memory.stack[index as usize] {
            //         self.state.goto(target);
            //     }
            //     todo!("Tail call");
            // }

            // Code::FREE => {
            //     if let Some(item) = self.state.memory.stack.pop() {
            //         match item {
            //             VmData::String(index) => {
            //                 self.state.free_heap(index);
            //             }
            //             VmData::List(index) => {
            //                 self.state.free_heap(index);
            //             }
            //             _ => {
            //                 todo!()
            //             }
            //         }
            //     }
            // }
            // NEWSTRUCT: pop N field values + N field name strings + struct name string
            // Stack layout (top first): field_name_N, ..., field_name_1, field_val_N, ..., field_val_1
            // The struct name is encoded as a STRING pushed right before the field names
            Code::NEWSTRUCT => {
                let num_fields = u64::from_le_bytes(self.state.next_arr()) as usize;

                // Pop the struct name string
                let struct_name = match self.state.memory.stack.pop() {
                    Some(VmData::Object(idx)) => {
                        let name = self
                            .state
                            .memory
                            .ref_from_heap(idx)
                            .and_then(|o| o.as_string())
                            .unwrap_or_default();
                        self.state.memory.dec(idx);
                        name
                    }
                    _ => String::new(),
                };

                // Pop field names (they are strings on the heap)
                let mut field_names = Vec::with_capacity(num_fields);
                for _ in 0..num_fields {
                    match self.state.memory.stack.pop() {
                        Some(VmData::Object(idx)) => {
                            let name = self
                                .state
                                .memory
                                .ref_from_heap(idx)
                                .and_then(|o| o.as_string())
                                .unwrap_or_default();
                            self.state.memory.dec(idx);
                            field_names.push(name);
                        }
                        _ => field_names.push(String::new()),
                    }
                }
                field_names.reverse();

                // Pop field values
                let mut field_values = Vec::with_capacity(num_fields);
                for _ in 0..num_fields {
                    if let Some(value) = self.state.memory.stack.pop() {
                        field_values.push(value);
                    }
                }
                field_values.reverse();

                // Build the table (field_name -> index in data)
                let mut table = std::collections::HashMap::new();
                for (i, name) in field_names.iter().enumerate() {
                    table.insert(name.clone(), i);
                }

                // Inhibit GC: field_values may contain heap Objects that were
                // raw-popped and are invisible to collect_cycles.
                self.state.memory.gc_inhibit();

                // Add the "type" field: allocate a string object with the struct name
                let type_str_obj = Object::string(struct_name.clone());
                let type_str_idx = self.state.memory.allocate(type_str_obj);
                let type_field_index = field_values.len();
                field_values.push(VmData::Object(type_str_idx));
                table.insert("type".to_string(), type_field_index);

                // Create the struct object
                let obj = Object {
                    object_type: memory_manager::ObjectType::Struct(struct_name),
                    table,
                    data: field_values,
                };
                let idx = self.state.memory.allocate(obj);
                self.state.memory.stack.push(VmData::Object(idx));
                self.state.memory.gc_release();
            }

            // GETF: pop field_name (string), pop object -> push object.data[table[field_name]]
            Code::GETF => {
                let (Some(field_name_val), Some(object_val)) =
                    (self.state.memory.stack.pop(), self.state.memory.stack.pop())
                else {
                    return Err(self.runtime_error("GETF: not enough arguments"));
                };

                // Get field name string
                let field_name = match field_name_val {
                    VmData::Object(idx) => {
                        let name = self
                            .state
                            .memory
                            .ref_from_heap(idx)
                            .and_then(|o| o.as_string())
                            .unwrap_or_default();
                        self.state.memory.dec(idx);
                        name
                    }
                    _ => {
                        return Err(self.runtime_error("GETF: field name must be a string"));
                    }
                };

                match object_val {
                    VmData::Object(obj_idx) => {
                        let heap_obj =
                            self.state.memory.ref_from_heap(obj_idx).ok_or_else(|| {
                                self.runtime_error("GETF: invalid heap reference")
                            })?;
                        if let Some(value) = heap_obj.get(&field_name) {
                            self.state.memory.push(value);
                        } else {
                            return Err(self.runtime_error(format!(
                                "Field '{}' not found on object",
                                field_name
                            )));
                        }
                        self.state.memory.dec(obj_idx);
                    }
                    _ => {
                        return Err(self.runtime_error("GETF: expected an object"));
                    }
                }
            }

            // PINF: pop object, pop field_name (string), pop value -> object.data[table[field_name]] = value
            Code::PINF => {
                let (Some(object_val), Some(field_name_val), Some(value)) = (
                    self.state.memory.stack.pop(),
                    self.state.memory.stack.pop(),
                    self.state.memory.stack.pop(),
                ) else {
                    return Err(self.runtime_error("PINF: not enough arguments"));
                };

                // Get field name string
                let field_name = match field_name_val {
                    VmData::Object(idx) => {
                        let name = self
                            .state
                            .memory
                            .ref_from_heap(idx)
                            .and_then(|o| o.as_string())
                            .unwrap_or_default();
                        self.state.memory.dec(idx);
                        name
                    }
                    _ => {
                        return Err(self.runtime_error("PINF: field name must be a string"));
                    }
                };

                match object_val {
                    VmData::Object(obj_idx) => {
                        // Look up the field index
                        let field_idx = {
                            let heap_obj = self.state.memory.ref_from_heap(obj_idx).unwrap();
                            heap_obj.table.get(&field_name).copied()
                        };
                        if let Some(idx) = field_idx {
                            // Dec old value, set new value
                            let old_value = {
                                let heap_obj =
                                    self.state.memory.ref_from_heap_mut(obj_idx).unwrap();
                                let old = heap_obj.data[idx];
                                heap_obj.data[idx] = value;
                                old
                            };
                            self.state.memory.dec_value(old_value);
                        } else {
                            return Err(self.runtime_error(format!(
                                "Field '{}' not found for assignment",
                                field_name
                            )));
                        }
                        self.state.memory.dec(obj_idx);
                    }
                    _ => {
                        return Err(self.runtime_error("PINF: expected an object"));
                    }
                }
            }

            error => {
                return Err(self.runtime_error(format!("Unknown VM opcode: {}", error)));
            }
        }

        // dbg!(&self.state.memory.stack);
        // dbg!(&self.state.program[self.state.current_instruction]);
        Ok(())
    }

    /// Execute a single VM instruction for the debugger.
    fn step_one_debug(&mut self) -> StepResult {
        if self.state.current_instruction >= self.state.program.len() {
            return StepResult::Finished { output: None };
        }

        let ip_before = self.state.current_instruction;
        let opcode = self.state.program[ip_before];
        let opname = code::byte_to_string(opcode);

        let instruction = self.state.next_instruction();
        match instruction {
            Code::RET => {
                let with_return = self.state.next_instruction();
                if let Some(destination) = self.state.callstack.pop() {
                    if with_return == 1 {
                        if let Err(e) = self.state.deallocate_registers_with_return() {
                            return StepResult::Error(format!("{}", e));
                        }
                    } else {
                        if let Err(e) = self.state.deallocate_registers() {
                            return StepResult::Error(format!("{}", e));
                        }
                    }
                    match destination {
                        CallType::Function(target) => self.state.goto(target),
                        CallType::Closure { target, closure } => {
                            self.state.goto(target);
                            self.state.memory.dec(closure);
                        }
                    }
                    StepResult::Continue {
                        opname: "RET".to_string(),

                        output: None,
                    }
                } else {
                    StepResult::Finished { output: None }
                }
            }
            Code::EXIT => StepResult::Finished { output: None },
            Code::ERROR => StepResult::Error("Error instruction reached".into()),
            Code::PRINT => {
                let output_str = match self.state.memory.stack.pop() {
                    Some(VmData::Int(i)) => format!("{}", i),
                    Some(VmData::Float(f)) => format!("{}", f),
                    Some(VmData::Bool(b)) => format!("{}", b),
                    Some(VmData::Char(c)) => format!("{}", c),
                    Some(VmData::Object(l)) => {
                        let s = self.state.memory.print_heap_object(l, 0);
                        self.state.memory.dec_value(VmData::Object(l));
                        s
                    }
                    Some(VmData::None) => "None".to_string(),
                    Some(VmData::Function(f)) => format!("<fn:{}>", f),
                    Some(VmData::StackAddress(s)) => {
                        let r = self.state.memory.print_heap_object(s, 0);
                        self.state.memory.dec_value(VmData::StackAddress(s));
                        r
                    }
                    None => "(empty)".to_string(),
                };
                StepResult::Continue {
                    opname,
                    output: Some(output_str),
                }
            }
            other => match self.dispatch(other) {
                Ok(()) => StepResult::Continue {
                    opname,
                    output: None,
                },
                Err(e) => StepResult::Error(format!("{}", e)),
            },
        }
    }

    /// Peek at the operand for display (does NOT advance IP).
    fn peek_operand(&self, opcode: u8) -> String {
        let ip = self.state.current_instruction + 1;
        let prog = &self.state.program;
        match opcode {
            Code::INTEGER => {
                if ip + 8 <= prog.len() {
                    format!(
                        "{}",
                        i64::from_le_bytes(prog[ip..ip + 8].try_into().unwrap_or_default())
                    )
                } else {
                    String::new()
                }
            }
            Code::FLOAT => {
                if ip + 8 <= prog.len() {
                    format!(
                        "{}",
                        f64::from_le_bytes(prog[ip..ip + 8].try_into().unwrap_or_default())
                    )
                } else {
                    String::new()
                }
            }
            Code::STORE | Code::GET | Code::STOREFAST | Code::DIRECTCALL | Code::STOREGLOBAL
            | Code::GETGLOBAL | Code::ALLOCLOCALS | Code::ALLOCATEGLOBAL | Code::JUMPIFFALSE
            | Code::JMP | Code::FUNCTION | Code::CLOSURE => {
                if ip + 4 <= prog.len() {
                    format!(
                        "{}",
                        u32::from_le_bytes(prog[ip..ip + 4].try_into().unwrap_or_default())
                    )
                } else {
                    String::new()
                }
            }
            Code::OFFSET => {
                if ip + 8 <= prog.len() {
                    let a =
                        i32::from_le_bytes(prog[ip..ip + 4].try_into().unwrap_or_default());
                    let b = i32::from_le_bytes(
                        prog[ip + 4..ip + 8].try_into().unwrap_or_default(),
                    );
                    format!("args={}, locals={}", a, b)
                } else {
                    String::new()
                }
            }
            Code::BJMP => {
                if ip + 4 <= prog.len() {
                    format!(
                        "-{}",
                        u32::from_le_bytes(prog[ip..ip + 4].try_into().unwrap_or_default())
                    )
                } else {
                    String::new()
                }
            }
            Code::RET => {
                if ip < prog.len() && prog[ip] != 0 {
                    "(value)".into()
                } else {
                    String::new()
                }
            }
            Code::NATIVE => {
                if ip + 8 <= prog.len() {
                    format!(
                        "#{}",
                        u64::from_le_bytes(prog[ip..ip + 8].try_into().unwrap_or_default())
                    )
                } else {
                    String::new()
                }
            }
            Code::CHAR => {
                if ip < prog.len() {
                    format!("'{}'", prog[ip] as char)
                } else {
                    String::new()
                }
            }
            Code::BYTE => {
                if ip < prog.len() {
                    format!("{}", prog[ip] as i64)
                } else {
                    String::new()
                }
            }
            Code::NEWLIST | Code::NEWSTRUCT | Code::GETBIND | Code::CID | Code::STACKREF => {
                if ip + 8 <= prog.len() {
                    format!(
                        "{}",
                        u64::from_le_bytes(prog[ip..ip + 8].try_into().unwrap_or_default())
                    )
                } else {
                    String::new()
                }
            }
            Code::STRING => {
                if ip + 8 <= prog.len() {
                    let sz = u64::from_le_bytes(
                        prog[ip..ip + 8].try_into().unwrap_or_default(),
                    ) as usize;
                    if ip + 8 + sz <= prog.len() {
                        let s = String::from_utf8_lossy(&prog[ip + 8..ip + 8 + sz]);
                        if s.len() > 30 {
                            format!("\"{}...\"", &s[..27])
                        } else {
                            format!("\"{}\"", s)
                        }
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                }
            }
            _ => String::new(),
        }
    }

    /// Peek at the operand with debug info names resolved.
    fn peek_operand_named(
        &self,
        opcode: u8,
        info: &common::debug_info::DebugInfo,
    ) -> String {
        let ip = self.state.current_instruction + 1;
        let prog = &self.state.program;
        match opcode {
            Code::STORE | Code::GET | Code::STOREFAST => {
                if ip + 4 <= prog.len() {
                    let v =
                        u32::from_le_bytes(prog[ip..ip + 4].try_into().unwrap_or_default());
                    let name = info.local_name(0, v).unwrap_or_default();
                    if name.is_empty() {
                        format!("local[{}]", v)
                    } else {
                        format!("{} (local[{}])", name, v)
                    }
                } else {
                    String::new()
                }
            }
            Code::STOREGLOBAL | Code::GETGLOBAL => {
                if ip + 4 <= prog.len() {
                    let v =
                        u32::from_le_bytes(prog[ip..ip + 4].try_into().unwrap_or_default());
                    let name = info.global_name(v);
                    format!("{} (global[{}])", name, v)
                } else {
                    String::new()
                }
            }
            Code::DIRECTCALL => {
                if ip + 4 <= prog.len() {
                    let v =
                        u32::from_le_bytes(prog[ip..ip + 4].try_into().unwrap_or_default());
                    info.global_name(v)
                } else {
                    String::new()
                }
            }
            Code::NATIVE => {
                if ip + 8 <= prog.len() {
                    let v =
                        u64::from_le_bytes(prog[ip..ip + 8].try_into().unwrap_or_default());
                    info.native_name(v)
                } else {
                    String::new()
                }
            }
            _ => self.peek_operand(opcode),
        }
    }

    // ═══════════════════════════════════════════════════════════════════
    //  Interactive Debugger TUI
    // ═══════════════════════════════════════════════════════════════════
    pub fn run_debug(
        &mut self,
        info: common::debug_info::DebugInfo,
    ) -> NovaResult<()> {
        use common::debug_info::DebugInfo;

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
                 output_len: usize|
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

            // Build local_name lookup for current scope (scope 0 = top-level)
            let local_name_map: HashMap<usize, String> = info
                .local_names
                .get(&0)
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
            if let Some(local_names) = info.local_names.get(&0) {
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

            Snapshot {
                ip: executed_ip,
                opname: opname.to_string(),
                named_op: named_op.to_string(),
                stack,
                callstack_depth: state.callstack.len(),
                offset: state.offset,
                locals,
                globals_changed,
                output_len,
                heap_live: state.memory.live_count(),
                heap_capacity: state.memory.heap_capacity(),
                heap_free: state.memory.free_count(),
                gc_threshold: state.memory.gc_threshold(),
                gc_base: state.memory.gc_base_threshold(),
                gc_locked: state.memory.gc_lock_depth() > 0,
                stack_depth: state.memory.stack.len(),
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

        // Pre-decode bytecode listing for the left panel
        struct BytecodeLine {
            addr: usize,
            opname: String,
            operand: String,
        }

        let mut bc_lines: Vec<BytecodeLine> = Vec::new();
        {
            let prog = &self.state.program;
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

        // Initial snapshot
        let ip0 = self.state.current_instruction;
        let op0 = if ip0 < self.state.program.len() {
            code::byte_to_string(self.state.program[ip0])
        } else {
            "END".into()
        };
        let named0 = if ip0 < self.state.program.len() {
            self.peek_operand_named(self.state.program[ip0], &info)
        } else {
            String::new()
        };
    history.push(take_snapshot(&self.state, ip0, &op0, &named0, &info, 0));

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
                      play_speed_ms: u64|
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
                    ("PgUp", "Jump back 20 steps"),
                    ("PgDn", "Jump forward 20 steps"),
                    ("Home", "Go to beginning"),
                    ("End", "Go to latest step"),
                    ("p", "Play / pause (auto-step visually)"),
                    ("+ / =", "Speed up playback"),
                    ("- / _", "Slow down playback"),
                    ("r", "Run to end (execute all remaining)"),
                    ("n", "Step over (run until callstack returns)"),
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
            let header = format!(
                " Nova Debugger │ Step {}/{} │ IP:{} │ Depth:{} │ Offset:{}{}",
                cursor + 1,
                history.len(),
                snap.ip,
                snap.callstack_depth,
                snap.offset,
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
            let bc_start = if current_bc_line > half {
                current_bc_line - half
            } else {
                0
            };

            // Column 2: Stack (top-of-stack first)
            let mut stack_lines: Vec<(Color, String)> = Vec::new();
            stack_lines.push((Color::Cyan, "─ Stack ─".to_string()));
            let stack_label = format!(
                " ({} entries, offset={})",
                snap.stack.len(),
                snap.offset
            );
            stack_lines.push((Color::DarkGrey, stack_label));

            let max_stack = panel_rows.saturating_sub(3).max(3);
            let stack_show: Vec<_> = snap
                .stack
                .iter()
                .enumerate()
                .rev()
                .take(max_stack)
                .collect();
            for (i, entry) in stack_show.iter().rev() {
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
            if snap.stack.len() > max_stack {
                stack_lines.push((
                    Color::DarkGrey,
                    format!(" ... {} more", snap.stack.len() - max_stack),
                ));
            }
            if snap.stack.is_empty() {
                stack_lines.push((Color::DarkGrey, " (empty)".to_string()));
            }

            // Column 3: Variables (locals + globals)
            let mut var_lines: Vec<(Color, String)> = Vec::new();
            var_lines.push((Color::Cyan, "─ Variables ─".to_string()));
            if snap.locals.is_empty() && snap.globals_changed.is_empty() {
                var_lines.push((Color::DarkGrey, " (none)".to_string()));
            }
            if !snap.locals.is_empty() {
                var_lines.push((Color::DarkGrey, " locals:".to_string()));
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

            // Render 3 columns side by side
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

                // Column 2: Stack
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

                // Column 3: Variables
                if row < var_lines.len() {
                    let (color, ref text) = var_lines[row];
                    let display = trunc(text, col3_w);
                    stdout
                        .queue(SetForegroundColor(color))?
                        .queue(Print(&display))?
                        .queue(SetAttribute(Attribute::Reset))?;
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

            // Controls bar
            let controls = if playing {
                " [p] pause  [+/-] speed  [↑/↓] step  [?] help  [q] quit"
            } else {
                " [↑/↓] step  [p] play  [r] run  [n] next  [Home/End] bounds  [?] help  [q] quit"
            };
            stdout
                .queue(SetForegroundColor(Color::DarkGrey))?
                .queue(Print(controls))?
                .queue(SetAttribute(Attribute::Reset))?;

            stdout.flush()?;
            Ok(())
        };

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
                        );
                        continue;
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

                        // Step forward (pauses play mode)
                        KeyCode::Down
                        | KeyCode::Char('j')
                        | KeyCode::Char(' ') => {
                            playing = false;
                            if cursor < history.len() - 1 {
                                cursor += 1;
                            } else if !finished && error_msg.is_none() {
                                let ip_before = self.state.current_instruction;
                                let opcode = if ip_before < self.state.program.len()
                                {
                                    self.state.program[ip_before]
                                } else {
                                    0
                                };
                                let named =
                                    self.peek_operand_named(opcode, &info);
                                match self.step_one_debug() {
                                    StepResult::Continue {
                                        opname,
                                        output,
                                    } => {
                                        if let Some(ref o) = output {
                                            output_buf.push(o.clone());
                                        }
                                        history.push(take_snapshot(
                                            &self.state,
                                            ip_before,
                                            &opname,
                                            &named,
                                            &info,
                                            output_buf.len(),
                                        ));
                                        cursor = history.len() - 1;
                                    }
                                    StepResult::Finished { output } => {
                                        if let Some(ref o) = output {
                                            output_buf.push(o.clone());
                                        }
                                        finished = true;
                                        history.push(take_snapshot(
                                            &self.state,
                                            ip_before,
                                            "END",
                                            "",
                                            &info,
                                            output_buf.len(),
                                        ));
                                        cursor = history.len() - 1;
                                    }
                                    StepResult::Error(msg) => {
                                        error_msg = Some(msg);
                                        history.push(take_snapshot(
                                            &self.state,
                                            ip_before,
                                            "ERROR",
                                            "",
                                            &info,
                                            output_buf.len(),
                                        ));
                                        cursor = history.len() - 1;
                                    }
                                }
                            }
                        }

                        // Step backward (pauses play mode)
                        KeyCode::Up | KeyCode::Char('k') => {
                            playing = false;
                            if cursor > 0 {
                                cursor -= 1;
                            }
                        }

                        // Page navigation (pauses play mode)
                        KeyCode::PageDown => {
                            playing = false;
                            for _ in 0..20 {
                                if cursor < history.len() - 1 {
                                    cursor += 1;
                                } else if !finished && error_msg.is_none() {
                                    let ip_b = self.state.current_instruction;
                                    let opc =
                                        if ip_b < self.state.program.len() {
                                            self.state.program[ip_b]
                                        } else {
                                            0
                                        };
                                    let named =
                                        self.peek_operand_named(opc, &info);
                                    match self.step_one_debug() {
                                        StepResult::Continue {
                                            opname,
                                                output,
                                        } => {
                                            if let Some(ref o) = output {
                                                output_buf.push(o.clone());
                                            }
                                            history.push(take_snapshot(
                                            &self.state,
                                            ip_b,
                                            &opname,
                                            &named,
                                            &info,
                                            output_buf.len(),
                                            ));
                                            cursor = history.len() - 1;
                                        }
                                        StepResult::Finished { output } => {
                                            if let Some(ref o) = output {
                                                output_buf.push(o.clone());
                                            }
                                            finished = true;
                                            history.push(take_snapshot(
                                                &self.state,
                                                ip_b,
                                                "END",
                                                "",
                                                &info,
                                                output_buf.len(),
                                            ));
                                            cursor = history.len() - 1;
                                            break;
                                        }
                                        StepResult::Error(msg) => {
                                            error_msg = Some(msg);
                                            history.push(take_snapshot(
                                                &self.state,
                                                ip_b,
                                                "ERROR",
                                                "",
                                                &info,
                                                output_buf.len(),
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
                                let ip_before = self.state.current_instruction;
                                let opcode = if ip_before < self.state.program.len() {
                                    self.state.program[ip_before]
                                } else {
                                    0
                                };
                                let named = self.peek_operand_named(opcode, &info);
                                match self.step_one_debug() {
                                    StepResult::Continue {
                                        opname,
                                        output,
                                    } => {
                                        if let Some(ref o) = output {
                                            output_buf.push(o.clone());
                                        }
                                        history.push(take_snapshot(
                                            &self.state,
                                            ip_before,
                                            &opname,
                                            &named,
                                            &info,
                                            output_buf.len(),
                                        ));
                                    }
                                    StepResult::Finished { output } => {
                                        if let Some(ref o) = output {
                                            output_buf.push(o.clone());
                                        }
                                        finished = true;
                                        history.push(take_snapshot(
                                            &self.state,
                                            ip_before,
                                            "END",
                                            "",
                                            &info,
                                            output_buf.len(),
                                        ));
                                    }
                                    StepResult::Error(msg) => {
                                        error_msg = Some(msg);
                                        history.push(take_snapshot(
                                            &self.state,
                                            ip_before,
                                            "ERROR",
                                            "",
                                            &info,
                                            output_buf.len(),
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
                                let ip_before = self.state.current_instruction;
                                let opcode = if ip_before < self.state.program.len() {
                                    self.state.program[ip_before]
                                } else {
                                    0
                                };
                                let named = self.peek_operand_named(opcode, &info);
                                match self.step_one_debug() {
                                    StepResult::Continue {
                                        opname,
                                        output,
                                    } => {
                                        if let Some(ref o) = output {
                                            output_buf.push(o.clone());
                                        }
                                        history.push(take_snapshot(
                                            &self.state,
                                            ip_before,
                                            &opname,
                                            &named,
                                            &info,
                                            output_buf.len(),
                                        ));
                                        if self.state.callstack.len()
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
                                            &self.state,
                                            ip_before,
                                            "END",
                                            "",
                                            &info,
                                            output_buf.len(),
                                        ));
                                        break;
                                    }
                                    StepResult::Error(msg) => {
                                        error_msg = Some(msg);
                                        history.push(take_snapshot(
                                            &self.state,
                                            ip_before,
                                            "ERROR",
                                            "",
                                            &info,
                                            output_buf.len(),
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
                    let ip_before = self.state.current_instruction;
                    let opcode = if ip_before < self.state.program.len() {
                        self.state.program[ip_before]
                    } else {
                        0
                    };
                    let named = self.peek_operand_named(opcode, &info);
                    match self.step_one_debug() {
                        StepResult::Continue { opname, output } => {
                            if let Some(ref o) = output {
                                output_buf.push(o.clone());
                            }
                            history.push(take_snapshot(
                                &self.state,
                                ip_before,
                                &opname,
                                &named,
                                &info,
                                output_buf.len(),
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
                                &self.state,
                                ip_before,
                                "END",
                                "",
                                &info,
                                output_buf.len(),
                            ));
                            cursor = history.len() - 1;
                        }
                        StepResult::Error(msg) => {
                            error_msg = Some(msg);
                            playing = false;
                            history.push(take_snapshot(
                                &self.state,
                                ip_before,
                                "ERROR",
                                "",
                                &info,
                                output_buf.len(),
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
            );
        }

        let _ = stdout.execute(terminal::LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
        Ok(())
    }

    /// Deep value equality for VmData, following heap references for objects.
    fn deep_equal(&self, a: &VmData, b: &VmData) -> bool {
        match (a, b) {
            (VmData::Int(x), VmData::Int(y)) => x == y,
            (VmData::Float(x), VmData::Float(y)) => x == y,
            (VmData::Bool(x), VmData::Bool(y)) => x == y,
            (VmData::Char(x), VmData::Char(y)) => x == y,
            (VmData::None, VmData::None) => true,
            (VmData::Function(x), VmData::Function(y)) => x == y,
            (VmData::StackAddress(x), VmData::StackAddress(y)) => x == y,
            (VmData::Object(idx_a), VmData::Object(idx_b)) => {
                if idx_a == idx_b {
                    return true; // same heap slot
                }
                let (obj_a, obj_b) = match (
                    self.state.memory.ref_from_heap(*idx_a),
                    self.state.memory.ref_from_heap(*idx_b),
                ) {
                    (Some(a), Some(b)) => (a, b),
                    _ => return false,
                };
                if obj_a.get_type() != obj_b.get_type() {
                    return false;
                }
                let data_a = obj_a.get_data();
                let data_b = obj_b.get_data();
                if data_a.len() != data_b.len() {
                    return false;
                }
                for (va, vb) in data_a.iter().zip(data_b.iter()) {
                    if !self.deep_equal(va, vb) {
                        return false;
                    }
                }
                true
            }
            _ => false,
        }
    }
}
