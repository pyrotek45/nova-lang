pub mod memory_manager;
pub mod state;
pub type CallBack = fn(state: &mut state::State) -> NovaResult<()>;

use std::{
    borrow::Cow,
    collections::HashMap,
    io::{self, Write},
    process::exit,
};

use common::{
    code::{self as code, Code},
    error::{NovaError, NovaResult},
    fileposition::FilePosition,
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
pub enum StepResult {
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
    pub fn step_one_debug(&mut self) -> StepResult {
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
                    } else if let Err(e) = self.state.deallocate_registers() {
                        return StepResult::Error(format!("{}", e));
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
    pub fn peek_operand(&self, opcode: u8) -> String {
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
    pub fn peek_operand_named(
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
