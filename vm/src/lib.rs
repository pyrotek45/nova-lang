pub mod memory_manager;
pub mod state;
pub type CallBack = fn(state: &mut state::State) -> Result<(), NovaError>;

use std::{
    collections::HashMap,
    io::{self, Write},
    process::exit,
};

use common::{code::Code, error::NovaError, fileposition::FilePosition};

use memory_manager::{Object, VmData};
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

impl Vm {
    #[inline(always)]
    pub fn run(&mut self) -> Result<(), NovaError> {
        loop {
            match self.state.next_instruction() {
                Code::RET => {
                    let with_return = self.state.next_instruction();
                    if let Some(destination) = self.state.callstack.pop() {
                        if with_return == 1 {
                            self.state.deallocate_registers_with_return();
                        } else {
                            self.state.deallocate_registers();
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
                    return Err(NovaError::RuntimeWithPos {
                        msg: "Error".into(),
                        position: self.runtime_errors_table[&self.state.current_instruction]
                            .clone(),
                    });
                }
                Code::EXIT => exit(0),
                instruction => self.dispatch(instruction)?,
            }
        }
        // self.state.memory.collect();
        println!("");
        let mut counts = HashMap::new();
        println!("Stack:");
        for (i, item) in self.state.memory.stack.iter().enumerate() {
            if let VmData::Object(index) = item {
                counts.insert(*index, counts.get(index).unwrap_or(&0) + 1);

                let object = self.state.memory.ref_from_heap(*index as usize).unwrap();
                for i in object.data.iter() {
                    if let VmData::Object(index) = i {
                        counts.insert(*index, counts.get(index).unwrap_or(&0) + 1);
                    }
                }
            }
            println!("{}: {:?}", i, item);
        }
        println!("\nHeap:");
        for (i, item) in self.state.memory.heap.iter().enumerate() {
            if item.is_none() {
                continue;
            }
            println!(
                "Count: {} -> {}; {}: {:?}",
                counts.get(&i).unwrap_or(&0),
                item.as_ref().unwrap().ref_count,
                i,
                item
            );
        }
        Ok(())
    }

    fn dispatch(&mut self, instruction: u8) -> Result<(), NovaError> {
        match instruction {
            Code::ISSOME => match self.state.memory.stack.pop() {
                Some(VmData::None) => self.state.memory.stack.push(VmData::Bool(false)),
                None => (),
                _ => self.state.memory.stack.push(VmData::Bool(true)),
            },

            Code::UNWRAP => {
                if let Some(VmData::None) = self.state.memory.stack.last() {
                    // get the position of the error
                    if let Some(pos) = self
                        .runtime_errors_table
                        .get(&self.state.current_instruction)
                    {
                        return Err(NovaError::RuntimeWithPos {
                            msg: "Tried to Unwrap a None value".into(),
                            position: pos.clone(),
                        });
                    } else {
                        return Err(NovaError::Runtime {
                            msg: "Tried to Unwrap a None value".into(),
                        });
                    }
                }
            }

            Code::DUP => self
                .state
                .memory
                .stack
                .push(*self.state.memory.stack.last().unwrap()),

            Code::POP => {
                self.state.memory.pop();
            }

            Code::NATIVE => {
                let index = u64::from_le_bytes(self.state.next_arr());

                match self.native_functions[index as usize](&mut self.state) {
                    Ok(_) => {}
                    Err(error) => return Err(error),
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
                let item = self.state.memory.stack.pop().unwrap();
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
                io::stdout().flush().unwrap();

                self.state.memory.dec_value(item);
            }

            Code::FADD => {
                let (Some(VmData::Float(v1)), Some(VmData::Float(v2))) =
                    (self.state.memory.stack.pop(), self.state.memory.stack.pop())
                else {
                    return Err(NovaError::Runtime {
                        msg: format!(
                            "Error Not enough arguments Opcode : {}",
                            self.state.program[self.state.current_instruction]
                        )
                        .into(),
                    });
                };
                let result = v1 + v2;
                self.state.memory.stack.push(VmData::Float(result))
            }

            Code::FSUB => {
                let (Some(VmData::Float(v1)), Some(VmData::Float(v2))) =
                    (self.state.memory.stack.pop(), self.state.memory.stack.pop())
                else {
                    return Err(NovaError::Runtime {
                        msg: format!(
                            "Error Not enough arguments Opcode : {}",
                            self.state.program[self.state.current_instruction]
                        )
                        .into(),
                    });
                };
                let result = v2 - v1;
                self.state.memory.stack.push(VmData::Float(result))
            }

            Code::FMUL => {
                let (Some(VmData::Float(v1)), Some(VmData::Float(v2))) =
                    (self.state.memory.stack.pop(), self.state.memory.stack.pop())
                else {
                    return Err(NovaError::Runtime {
                        msg: format!(
                            "Error Not enough arguments Opcode : {}",
                            self.state.program[self.state.current_instruction]
                        )
                        .into(),
                    });
                };
                let result = v1 * v2;
                self.state.memory.stack.push(VmData::Float(result))
            }

            Code::FDIV => {
                let (Some(VmData::Float(v1)), Some(VmData::Float(v2))) =
                    (self.state.memory.stack.pop(), self.state.memory.stack.pop())
                else {
                    return Err(NovaError::Runtime {
                        msg: format!(
                            "Error Not enough arguments Opcode : {}",
                            self.state.program[self.state.current_instruction]
                        )
                        .into(),
                    });
                };
                let result = v2 / v1;
                self.state.memory.stack.push(VmData::Float(result))
            }

            Code::IADD => {
                let (Some(VmData::Int(v1)), Some(VmData::Int(v2))) =
                    (self.state.memory.stack.pop(), self.state.memory.stack.pop())
                else {
                    return Err(NovaError::Runtime {
                        msg: format!(
                            "Error Not enough arguments Opcode : {}",
                            self.state.program[self.state.current_instruction]
                        )
                        .into(),
                    });
                };
                let result = v1.checked_add(v2).ok_or_else(|| NovaError::Runtime {
                    msg: "Integer overflow".into(),
                })?;
                self.state.memory.stack.push(VmData::Int(result))
            }

            Code::ISUB => {
                let (Some(VmData::Int(v1)), Some(VmData::Int(v2))) =
                    (self.state.memory.stack.pop(), self.state.memory.stack.pop())
                else {
                    return Err(NovaError::Runtime {
                        msg: format!(
                            "Error Not enough arguments Opcode : {}",
                            self.state.program[self.state.current_instruction]
                        )
                        .into(),
                    });
                };
                let result = v2.checked_sub(v1).ok_or_else(|| NovaError::Runtime {
                    msg: "Integer overflow".into(),
                })?;
                self.state.memory.stack.push(VmData::Int(result))
            }

            Code::IMUL => {
                let (Some(VmData::Int(v1)), Some(VmData::Int(v2))) =
                    (self.state.memory.stack.pop(), self.state.memory.stack.pop())
                else {
                    return Err(NovaError::Runtime {
                        msg: format!(
                            "Error Not enough arguments Opcode : {}",
                            self.state.program[self.state.current_instruction]
                        )
                        .into(),
                    });
                };
                // safely multi

                let result = v1.checked_mul(v2).ok_or_else(|| NovaError::Runtime {
                    msg: "Integer overflow".into(),
                })?;
                self.state.memory.stack.push(VmData::Int(result))
            }

            Code::IDIV => {
                let (Some(VmData::Int(v1)), Some(VmData::Int(v2))) =
                    (self.state.memory.stack.pop(), self.state.memory.stack.pop())
                else {
                    return Err(NovaError::Runtime {
                        msg: format!(
                            "Error Not enough arguments Opcode : {}",
                            self.state.program[self.state.current_instruction]
                        )
                        .into(),
                    });
                };
                let result = v2.checked_div(v1).ok_or_else(|| NovaError::Runtime {
                    msg: "Integer division by zero".into(),
                })?;
                self.state.memory.stack.push(VmData::Int(result))
            }

            Code::STOREGLOBAL => {
                let index = u32::from_le_bytes(self.state.next_arr());
                let item = self.state.memory.stack.pop().unwrap();
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
                    panic!("Error on Closure")
                };

                let mut myarray = vec![];
                for _ in 0..size {
                    if let Some(value) = self.state.memory.stack.pop() {
                        myarray.push(value);
                    } else {
                        todo!()
                    }
                }

                myarray.reverse();

                let closure = self
                    .state
                    .memory
                    .allocate(Object::closure(self.state.current_instruction + 4, myarray));

                self.state.memory.stack.push(VmData::Object(closure));
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
                        if let Some(object) = self.state.memory.ref_from_heap(index as usize) {
                            if let Some((target, env)) = object.as_closure() {
                                for captured in env {
                                    self.state.memory.push(captured);
                                }
                                self.state.goto(target);
                            }
                        }
                    }
                    _ => {
                        dbg!(callee);
                        todo!()
                    }
                }
            }

            Code::CALL => {
                let Some(callee) = self.state.memory.stack.pop() else {
                    todo!()
                };
                match callee {
                    VmData::Object(index) => {
                        if let Some(object) = self.state.memory.ref_from_heap(index as usize) {
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
                        dbg!(a);
                        todo!()
                    }
                }
            }

            Code::ILSS => {
                let (Some(VmData::Int(v1)), Some(VmData::Int(v2))) =
                    (self.state.memory.stack.pop(), self.state.memory.stack.pop())
                else {
                    return Err(NovaError::Runtime {
                        msg: format!(
                            "Error Not enough arguments Opcode : {}",
                            self.state.program[self.state.current_instruction]
                        )
                        .into(),
                    });
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
                    dbg!(a, b);
                    return Err(NovaError::Runtime {
                        msg: format!(
                            "IGTR Error Not enough arguments Opcode : {}",
                            self.state.program[self.state.current_instruction]
                        )
                        .into(),
                    });
                }
            },

            Code::FLSS => {
                let (Some(VmData::Float(v1)), Some(VmData::Float(v2))) =
                    (self.state.memory.stack.pop(), self.state.memory.stack.pop())
                else {
                    return Err(NovaError::Runtime {
                        msg: format!(
                            "Error Not enough arguments Opcode : {}",
                            self.state.program[self.state.current_instruction]
                        )
                        .into(),
                    });
                };
                let result = v2 < v1;
                self.state.memory.stack.push(VmData::Bool(result))
            }

            Code::FGTR => {
                let (Some(VmData::Float(v1)), Some(VmData::Float(v2))) =
                    (self.state.memory.stack.pop(), self.state.memory.stack.pop())
                else {
                    return Err(NovaError::Runtime {
                        msg: format!(
                            "Error Not enough arguments Opcode : {}",
                            self.state.program[self.state.current_instruction]
                        )
                        .into(),
                    });
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
                if let VmData::Bool(test) = self.state.memory.stack.pop().unwrap() {
                    if !test {
                        self.state.current_instruction += jump as usize;
                    }
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
                    return Err(NovaError::Runtime {
                        msg: format!(
                            "Error Not enough arguments Opcode : {}",
                            self.state.program[self.state.current_instruction]
                        )
                        .into(),
                    });
                };
                match (v1, v2) {
                    (a, b) => {
                        if a == b {
                            self.state.memory.stack.push(VmData::Bool(true))
                        } else {
                            self.state.memory.stack.push(VmData::Bool(false))
                        }
                    }
                }
            }

            Code::NOT => match self.state.memory.stack.pop() {
                Some(VmData::Bool(b)) => {
                    self.state.memory.stack.push(VmData::Bool(!b));
                }
                _ => {
                    return Err(NovaError::Runtime {
                        msg: format!(
                            "Error on Opcode : {}",
                            self.state.program[self.state.current_instruction]
                        )
                        .into(),
                    });
                }
            },

            Code::AND => {
                if let (Some(VmData::Bool(v1)), Some(VmData::Bool(v2))) =
                    (self.state.memory.stack.pop(), self.state.memory.stack.pop())
                {
                    self.state.memory.stack.push(VmData::Bool(v1 && v2))
                }
            }

            Code::OR => {
                if let (Some(VmData::Bool(v1)), Some(VmData::Bool(v2))) =
                    (self.state.memory.stack.pop(), self.state.memory.stack.pop())
                {
                    self.state.memory.stack.push(VmData::Bool(v1 || v2))
                }
            }

            Code::NEG => {
                if let Some(value) = self.state.memory.stack.pop() {
                    match value {
                        VmData::Int(v) => self.state.memory.stack.push(VmData::Int(-v)),
                        VmData::Float(v) => self.state.memory.stack.push(VmData::Float(-v)),
                        _ => {
                            return Err(NovaError::Runtime {
                                msg: format!(
                                    "Error on Opcode : {}",
                                    self.state.program[self.state.current_instruction]
                                )
                                .into(),
                            });
                        }
                    }
                }
            }

            Code::IMODULO => {
                let (Some(VmData::Int(v1)), Some(VmData::Int(v2))) =
                    (self.state.memory.stack.pop(), self.state.memory.stack.pop())
                else {
                    return Err(NovaError::Runtime {
                        msg: format!(
                            "Error Not enough arguments Opcode : {}",
                            self.state.program[self.state.current_instruction]
                        )
                        .into(),
                    });
                };
                let result = v2.modulo(v1);
                self.state.memory.stack.push(VmData::Int(result))
            }

            Code::ASSIGN => {
                let (destination, value) =
                    match (self.state.memory.stack.pop(), self.state.memory.stack.pop()) {
                        (Some(dest), Some(val)) => (dest, val),
                        _ => {
                            return Err(NovaError::Runtime {
                                msg: "Not enough operands for assignment".into(),
                            })
                        }
                    };
                if let VmData::Int(index) = destination {
                    let target_index = self.state.offset + index as usize;
                    self.state.memory.store(target_index, value);
                } else {
                    return Err(NovaError::Runtime {
                        msg: format!("Invalid assignment destination: {:?}", destination).into(),
                    });
                }
            }

            Code::NEWLIST => {
                let size = u64::from_le_bytes(self.state.next_arr());
                let mut myarray = vec![];
                for _ in 0..size {
                    if let Some(value) = self.state.memory.stack.pop() {
                        myarray.push(value);
                    } else {
                        todo!()
                    }
                }
                myarray.reverse();
                self.state.memory.push_list(myarray);
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
                    Err(_) => todo!(),
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

            Code::CONCAT => match (self.state.memory.stack.pop(), self.state.memory.stack.pop()) {
                (Some(VmData::Object(index1)), Some(VmData::Object(index2))) => {
                    let object2 = self.state.memory.ref_from_heap(index1 as usize).unwrap();
                    let object1 = self.state.memory.ref_from_heap(index2 as usize).unwrap();
                    let new_data = object1
                        .data
                        .iter()
                        .cloned()
                        .chain(object2.data.iter().cloned())
                        .collect::<Vec<VmData>>();
                    let new_object = Object::new(object1.object_type.clone(), new_data);
                    let result = self.state.memory.allocate(new_object);
                    self.state.memory.dec(index1);
                    self.state.memory.dec(index2);
                    self.state.memory.stack.push(VmData::Object(result));
                }
                _ => {
                    return Err(NovaError::Runtime {
                        msg: "Error on Concat".into(),
                    })
                }
            },

            Code::LINDEX => {
                let (Some(array), Some(index)) =
                    (self.state.memory.stack.pop(), self.state.memory.stack.pop())
                else {
                    todo!()
                };
                match (array, index) {
                    (VmData::Object(object), VmData::Int(index)) => {
                        let heap_object = self.state.memory.ref_from_heap(object as usize).unwrap();
                        if let Some(item) = heap_object.data.get(index as usize) {
                            self.state.memory.push(item.clone());
                        }
                        self.state.memory.dec(object);
                    }
                    (a, b) => {
                        dbg!(a, b);
                        todo!()
                    }
                }
            }

            Code::PINDEX => {
                let (Some(array), Some(index), Some(value)) = (
                    self.state.memory.stack.pop(),
                    self.state.memory.stack.pop(),
                    self.state.memory.stack.pop(),
                ) else {
                    return Err(NovaError::Runtime {
                        msg: format!(
                            "Error Not enough arguments Opcode : {}",
                            self.state.program[self.state.current_instruction]
                        )
                        .into(),
                    });
                };

                match (array, index, value) {
                    (VmData::Object(object), VmData::Int(index), value) => {
                        let old_value = {
                            let heap_object = self.state.memory.ref_from_heap_mut(object as usize).unwrap();
                            heap_object.data.get(index as usize).cloned()
                        };
                        if let Some(old) = old_value {
                            self.state.memory.dec_value(old);
                            let heap_object = self.state.memory.ref_from_heap_mut(object as usize).unwrap();
                            if let Some(item) = heap_object.data.get_mut(index as usize) {
                                *item = value;
                            }
                        }
                        self.state.memory.dec(object);
                    }
                    (a, b, c) => {
                        dbg!(a, b, c);
                        todo!()
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
            error => {
                dbg!(error);
            }
        }

        // dbg!(&self.state.memory.stack);
        // dbg!(&self.state.program[self.state.current_instruction]);
        Ok(())
    }

    #[inline(always)]
    pub fn run_debug(&mut self) -> Result<(), NovaError> {
        Ok(())
    }
}
