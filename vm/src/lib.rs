pub mod state;
pub type CallBack = fn(state: &mut state::State) -> Result<(), NovaError>;

use std::{
    collections::HashMap,
    io::{self, Write},
    process::exit,
};

use common::{
    code::{byte_to_string, Code},
    error::NovaError,
    fileposition::FilePosition,
};

use modulo::Mod;
use state::{Heap, State};

use crate::state::VmData;

#[derive(Debug, Clone)]
pub struct Vm {
    pub runtime_errors_table: HashMap<usize, FilePosition>,
    pub native_functions: Vec<CallBack>,
    pub state: state::State,
}

pub fn new() -> Vm {
    Vm {
        native_functions: vec![],
        state: state::new(),
        runtime_errors_table: HashMap::default(),
    }
}

impl Vm {
    #[inline(always)]
    pub fn run(&mut self) -> Result<(), NovaError> {
        loop {
            // /dbg!(&self.state.stack, &self.state.program[self.state.current_instruction]);
            match self.state.next_instruction() {
                Code::ERROR => {
                    return Err(NovaError::RuntimeWithPos {
                        msg: "Error".into(),
                        position: self.runtime_errors_table[&self.state.current_instruction]
                            .clone(),
                    });
                }
                Code::EXIT => exit(0),
                Code::CONCAT => match (self.state.stack.pop(), self.state.stack.pop()) {
                    (Some(VmData::String(s1)), Some(VmData::String(s2))) => {
                        match (self.state.get_ref(s1), self.state.get_ref(s2)) {
                            (Heap::String(str2), Heap::String(str1)) => {
                                let newstring = format!("{}{}", str1, str2);
                                let index = self.state.allocate_string(newstring.into());
                                self.state.stack.push(VmData::String(index));
                            }

                            _ => {
                                return Err(NovaError::Runtime {
                                    msg: "Error on Concat".into(),
                                })
                            }
                        }
                    }
                    (Some(VmData::List(l1)), Some(VmData::List(l2))) => {
                        // make new list from both after getting lists from heap
                        match (self.state.get_ref(l1), self.state.get_ref(l2)) {
                            (Heap::List(list1), Heap::List(list2)) => {
                                let newlist = list2.iter().chain(list1).copied().collect();
                                self.state.gclock = true;
                                let index = self.state.allocate_array(newlist);
                                self.state.gclock = false;
                                self.state.stack.push(VmData::List(index));
                            }
                            _ => {
                                return Err(NovaError::Runtime {
                                    msg: "Error on Concat".into(),
                                })
                            }
                        }
                    }
                    _ => {
                        return Err(NovaError::Runtime {
                            msg: "Error on Concat".into(),
                        })
                    }
                },
                Code::ISSOME => match self.state.stack.pop() {
                    Some(VmData::None) => self.state.stack.push(VmData::Bool(false)),
                    None => (),
                    _ => self.state.stack.push(VmData::Bool(true)),
                },
                Code::UNWRAP => {
                    if let Some(VmData::None) = self.state.stack.last() {
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
                Code::DUP => self.state.stack.push(*self.state.stack.last().unwrap()),

                Code::POP => {
                    self.state.stack.pop();
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
                    self.state.stack.push(VmData::Int(int));
                }

                Code::STACKREF => {
                    let index = u32::from_le_bytes(self.state.next_arr());
                    self.state.stack.push(VmData::StackAddress(index as usize));
                }

                // takes item and stores it into stack at location
                // with offset
                Code::STORE => {
                    let index = u32::from_le_bytes(self.state.next_arr());
                    let data = self.state.stack.pop().unwrap();
                    self.state.stack[self.state.offset + index as usize] = data;
                }

                // gets the data from a local index in the stack
                // from offset
                Code::GET => {
                    let index = u32::from_le_bytes(self.state.next_arr());
                    let item = self.state.stack[self.state.offset + index as usize];
                    self.state.stack.push(item);
                }

                // jumps back to the callsite of a function
                Code::RET => {
                    let with_return = self.state.next_instruction();
                    if let Some(destination) = self.state.callstack.pop() {
                        if with_return == 1 {
                            self.state.deallocate_registers_with_return();
                        } else {
                            self.state.deallocate_registers();
                        }
                        self.state.goto(destination);
                        //dbg!(&self.state.stack);
                    } else {
                        break;
                    }
                }

                // i think you can figure this one out
                Code::PRINT => {
                    fn print_item(state: &mut State, item: VmData) {
                        let mut out = io::stdout().lock();
                        match item {
                            VmData::Function(v) => {
                                write!(out, "Function Pointer ({})", v).unwrap();
                            }
                            VmData::Int(v) => {
                                write!(out, "{}", v).unwrap();
                            }
                            VmData::Float(v) => {
                                write!(out, "{}", v).unwrap();
                            }
                            VmData::Bool(v) => {
                                write!(out, "{}", v).unwrap();
                            }
                            VmData::None => {
                                write!(out, "None").unwrap();
                            }
                            VmData::List(index) => {
                                state.print_heap(index);
                            }
                            VmData::String(index) => {
                                state.print_heap(index);
                            }
                            VmData::Closure(v) => {
                                state.print_heap(v);
                            }
                            VmData::StackAddress(v) => {
                                print_item(state, state.stack[state.offset + v]);
                            }
                            VmData::Struct(v) => {
                                state.print_heap(v);
                            }
                            VmData::Char(char) => {
                                write!(out, "{}", char).unwrap();
                            }
                        }
                        out.flush().unwrap();
                    }

                    let item = self.state.stack.pop().unwrap();
                    print_item(&mut self.state, item);
                }

                Code::FADD => {
                    let (Some(VmData::Float(v1)), Some(VmData::Float(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
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
                    self.state.stack.push(VmData::Float(result))
                }

                Code::FSUB => {
                    let (Some(VmData::Float(v1)), Some(VmData::Float(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
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
                    self.state.stack.push(VmData::Float(result))
                }

                Code::FMUL => {
                    let (Some(VmData::Float(v1)), Some(VmData::Float(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
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
                    self.state.stack.push(VmData::Float(result))
                }

                Code::FDIV => {
                    let (Some(VmData::Float(v1)), Some(VmData::Float(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
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
                    self.state.stack.push(VmData::Float(result))
                }

                Code::IADD => {
                    let (Some(VmData::Int(v1)), Some(VmData::Int(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
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
                    self.state.stack.push(VmData::Int(result))
                }

                Code::ISUB => {
                    let (Some(VmData::Int(v1)), Some(VmData::Int(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
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
                    self.state.stack.push(VmData::Int(result))
                }

                Code::IMUL => {
                    let (Some(VmData::Int(v1)), Some(VmData::Int(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
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
                    self.state.stack.push(VmData::Int(result))
                }

                Code::IDIV => {
                    let (Some(VmData::Int(v1)), Some(VmData::Int(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
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
                    self.state.stack.push(VmData::Int(result))
                }

                Code::STOREGLOBAL => {
                    let index = u32::from_le_bytes(self.state.next_arr());
                    let item = self.state.stack.pop().unwrap();
                    self.state.stack[index as usize] = item;
                }

                Code::FUNCTION => {
                    self.state
                        .stack
                        .push(VmData::Function(self.state.current_instruction + 4));

                    let jump = u32::from_le_bytes(self.state.next_arr());

                    self.state.current_instruction += jump as usize;
                }

                Code::CLOSURE => {
                    let Some(VmData::List(list)) = self.state.stack.pop() else {
                        todo!()
                    };
                    self.state.gclock = true;
                    let closure = self.state.allocate_new_heap();
                    self.state.heap[closure] =
                        Heap::Closure(self.state.current_instruction + 4, list);

                    self.state.stack.push(VmData::Closure(closure));
                    self.state.gclock = false;
                    self.state.collect_garbage();
                    let jump = u32::from_le_bytes(self.state.next_arr());
                    self.state.current_instruction += jump as usize;
                }

                Code::DIRECTCALL => {
                    self.state
                        .callstack
                        .push(self.state.current_instruction + 4);
                    let index = u32::from_le_bytes(self.state.next_arr());

                    let callee = self.state.stack[index as usize];

                    match callee {
                        VmData::Function(target) => {
                            self.state.goto(target);
                        }
                        VmData::Closure(target) => {
                            if let Heap::Closure(target, captured) = self.state.heap[target] {
                                if let Heap::List(list) = &self.state.heap[captured] {
                                    for &i in list {
                                        self.state.stack.push(self.state.to_vmdata(i))
                                    }
                                    self.state.goto(target);
                                } else {
                                    todo!()
                                }
                            } else {
                                todo!()
                            }
                        }
                        _ => {
                            dbg!(callee);
                            todo!()
                        }
                    }
                }

                Code::TAILCALL => {
                    let index = u32::from_le_bytes(self.state.next_arr());
                    if let VmData::Function(target) = self.state.stack[index as usize] {
                        self.state.goto(target);
                    }
                    todo!("Tail call");
                }

                Code::ILSS => {
                    let (Some(VmData::Int(v1)), Some(VmData::Int(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
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
                    self.state.stack.push(VmData::Bool(result))
                }

                Code::IGTR => match (self.state.stack.pop(), self.state.stack.pop()) {
                    (Some(VmData::Int(v1)), Some(VmData::Int(v2))) => {
                        let result = v2 > v1;
                        self.state.stack.push(VmData::Bool(result))
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
                        (self.state.stack.pop(), self.state.stack.pop())
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
                    self.state.stack.push(VmData::Bool(result))
                }

                Code::FGTR => {
                    let (Some(VmData::Float(v1)), Some(VmData::Float(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
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
                    self.state.stack.push(VmData::Bool(result))
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
                    if let VmData::Bool(test) = self.state.stack.pop().unwrap() {
                        if !test {
                            self.state.current_instruction += jump as usize;
                        }
                    }
                }

                Code::TRUE => {
                    self.state.stack.push(VmData::Bool(true));
                }

                Code::FALSE => {
                    self.state.stack.push(VmData::Bool(false));
                }

                Code::EQUALS => {
                    let (Some(v1), Some(v2)) = (self.state.stack.pop(), self.state.stack.pop())
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
                        (VmData::String(i1), VmData::String(i2)) => {
                            let s1 = self.state.heap[i1].get_string();
                            let s2 = self.state.heap[i2].get_string();
                            let result = s1 == s2;
                            self.state.stack.push(VmData::Bool(result))
                        }
                        _ => {
                            let result = v2 == v1;
                            self.state.stack.push(VmData::Bool(result))
                        }
                    }
                }

                Code::NOT => match self.state.stack.pop() {
                    Some(VmData::Bool(b)) => {
                        self.state.stack.push(VmData::Bool(!b));
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
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        self.state.stack.push(VmData::Bool(v1 && v2))
                    }
                }

                Code::OR => {
                    if let (Some(VmData::Bool(v1)), Some(VmData::Bool(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        self.state.stack.push(VmData::Bool(v1 || v2))
                    }
                }

                Code::NEG => {
                    if let Some(value) = self.state.stack.pop() {
                        match value {
                            VmData::Int(v) => self.state.stack.push(VmData::Int(-v)),
                            VmData::Float(v) => self.state.stack.push(VmData::Float(-v)),
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
                        (self.state.stack.pop(), self.state.stack.pop())
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
                    self.state.stack.push(VmData::Int(result))
                }

                Code::ASSIGN => {
                    let (Some(destination), Some(value)) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    else {
                        return Err(NovaError::Runtime {
                            msg: format!(
                                "Error Not enough arguments Opcode : {}",
                                self.state.program[self.state.current_instruction]
                            )
                            .into(),
                        });
                    };
                    match (value, destination) {
                        (item, VmData::StackAddress(index)) => {
                            match (self.state.stack[self.state.offset + index], value) {
                                (VmData::List(d), VmData::Closure(v)) => {
                                    self.state.heap[d] = self.state.heap[v].clone()
                                }
                                (VmData::List(d), VmData::List(v)) => {
                                    self.state.heap[d] = self.state.heap[v].clone();
                                }
                                (VmData::List(_), VmData::Struct(_)) => todo!(),
                                (VmData::List(_), VmData::String(_)) => todo!(),
                                (VmData::String(d), VmData::String(v)) => {
                                    self.state.heap[d] = self.state.heap[v].clone()
                                }
                                _ => self.state.stack[self.state.offset + index] = item,
                            }
                        }
                        (item, VmData::List(index)) => {
                            //dbg!(&item, &index);
                            match item {
                                VmData::Function(v) => self.state.heap[index] = Heap::Function(v),
                                VmData::Int(v) => self.state.heap[index] = Heap::Int(v),
                                VmData::Float(_) => todo!(),
                                VmData::Bool(_) => todo!(),
                                VmData::List(v) => {
                                    //dbg!(&self.state.heap[v]);
                                    self.state.heap[index] = Heap::ListAddress(v)
                                }
                                VmData::None => todo!(),
                                VmData::String(v) => {
                                    self.state.heap[index] = Heap::StringAddress(v)
                                }
                                VmData::Closure(_) => todo!(),
                                VmData::StackAddress(_) => todo!(),
                                VmData::Struct(_) => todo!(),
                                VmData::Char(v) => self.state.heap[index] = Heap::Char(v),
                            };
                        }
                        (a, b) => {
                            dbg!(a, b, self.state.program[self.state.current_instruction]);
                            todo!()
                        }
                    }
                }

                Code::NEWLIST => {
                    let size = u64::from_le_bytes(self.state.next_arr());

                    let mut myarray = vec![];
                    for _ in 0..size {
                        if let Some(value) = self.state.stack.pop() {
                            myarray.push(self.state.allocate_vmdata_to_heap(value))
                        } else {
                            todo!()
                        }
                    }
                    myarray.reverse();
                    let index = self.state.allocate_array(myarray);
                    self.state.stack.push(VmData::List(index));
                }

                Code::PINDEX => {
                    let (Some(array), Some(index)) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    else {
                        return Err(NovaError::Runtime {
                            msg: format!(
                                "Error Not enough arguments Opcode : {}",
                                self.state.program[self.state.current_instruction]
                            )
                            .into(),
                        });
                    };
                    match (array, index) {
                        (VmData::StackAddress(array_index), VmData::Int(index_to_get)) => {
                            if let VmData::List(newindex) =
                                &self.state.stack[self.state.offset + array_index]
                            {
                                if let Heap::List(array) = self.state.get_ref(*newindex) {
                                    if array.len() <= index_to_get as usize {
                                        if let Some(pos) = self
                                            .runtime_errors_table
                                            .get(&self.state.current_instruction)
                                        {
                                            return Err(NovaError::RuntimeWithPos { msg: format!("Invalid array access , array length: {}, index tried: {}", array.len(), index_to_get).into(), position: pos.clone() });
                                        } else {
                                            return Err(NovaError::Runtime { msg: format!("Invalid array access , array length: {}, index tried: {}", array.len(), index_to_get).into() });
                                        }
                                    }
                                    self.state
                                        .stack
                                        .push(VmData::List(array[index_to_get as usize]))
                                }
                            }
                        }
                        (VmData::List(array_index), VmData::Int(index_to_get)) => {
                            let Heap::ListAddress(newindex) = self.state.get_ref(array_index)
                            else {
                                return Err(NovaError::Runtime {
                                    msg: format!("Index out of bounds {}", array_index).into(),
                                });
                            };
                            if let Heap::List(array) = self.state.get_ref(*newindex) {
                                if array.len() <= index_to_get as usize {
                                    if let Some(pos) = self
                                        .runtime_errors_table
                                        .get(&self.state.current_instruction)
                                    {
                                        return Err(NovaError::RuntimeWithPos { msg: format!("Invalid array access , array length: {}, index tried: {}", array.len(), index_to_get).into(), position: pos.clone() });
                                    } else {
                                        return Err(NovaError::Runtime { msg: format!("Invalid array access , array length: {}, index tried: {}", array.len(), index_to_get).into() });
                                    }
                                }
                                self.state
                                    .stack
                                    .push(VmData::List(array[index_to_get as usize]))
                            }
                        }
                        (a, b) => {
                            dbg!(a, b);
                            todo!()
                        }
                    }
                }

                Code::LINDEX => {
                    let (Some(array), Some(index)) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    else {
                        todo!()
                    };
                    match (array, index) {
                        (VmData::List(array), VmData::Int(index_to)) => {
                            match self.state.get_ref(array) {
                                Heap::List(array) => {
                                    // test if array is out of bounds
                                    if array.len() <= index_to as usize {
                                        if let Some(pos) = self
                                            .runtime_errors_table
                                            .get(&self.state.current_instruction)
                                        {
                                            return Err(NovaError::RuntimeWithPos { msg: format!("Invalid array access , array length: {}, index tried: {}", array.len(), index_to).into(), position: pos.clone() });
                                        } else {
                                            return Err(NovaError::Runtime { msg: format!("Invalid array access , array length: {}, index tried: {}", array.len(), index_to).into() });
                                        }
                                    }
                                    let item = self.state.get_ref(array[index_to as usize]).clone();
                                    match item {
                                        Heap::Function(v) => {
                                            self.state.stack.push(VmData::Function(v))
                                        }
                                        Heap::Int(v) => self.state.stack.push(VmData::Int(v)),
                                        Heap::Float(v) => self.state.stack.push(VmData::Float(v)),
                                        Heap::Bool(v) => self.state.stack.push(VmData::Bool(v)),
                                        Heap::ListAddress(v) => {
                                            self.state.stack.push(VmData::List(v))
                                        }
                                        Heap::List(_) => panic!(),
                                        Heap::String(_) => panic!(),
                                        Heap::None => self.state.stack.push(VmData::None),
                                        Heap::StringAddress(v) => {
                                            self.state.stack.push(VmData::String(v))
                                        }
                                        Heap::Closure(_, _) => todo!(),
                                        Heap::ClosureAddress(v) => {
                                            self.state.stack.push(VmData::Closure(v))
                                        }
                                        Heap::Struct(_, _) => todo!(),
                                        Heap::StructAddress(_) => todo!(),
                                        Heap::Char(v) => self.state.stack.push(VmData::Char(v)),
                                    }
                                }
                                _ => {
                                    todo!()
                                }
                            }
                        }
                        (a, b) => {
                            dbg!(a, b);
                            todo!()
                        }
                    }
                }

                Code::FLOAT => {
                    let fl = f64::from_le_bytes(self.state.next_arr());
                    self.state.stack.push(VmData::Float(fl));
                }

                Code::GETGLOBAL => {
                    let index = u32::from_le_bytes(self.state.next_arr());
                    let item = self.state.stack[index as usize];
                    self.state.stack.push(item);
                }

                Code::CALL => {
                    let Some(callee) = self.state.stack.pop() else {
                        todo!()
                    };
                    match callee {
                        VmData::Closure(index) => {
                            if let Some(Heap::Closure(target, captured)) =
                                self.state.heap.get(index)
                            {
                                //dbg!(&self.state.heap[*captured]);
                                if let Heap::List(list) = &self.state.heap[*captured] {
                                    for i in list {
                                        self.state.stack.push(self.state.to_vmdata(*i))
                                    }
                                    self.state.callstack.push(self.state.current_instruction);
                                    self.state.goto(*target);
                                } else {
                                    dbg!(target, callee, captured);
                                    todo!()
                                }
                            } else {
                                todo!()
                            }
                        }
                        VmData::Function(target) => {
                            self.state.callstack.push(self.state.current_instruction);
                            self.state.goto(target);
                        }
                        a => {
                            dbg!(a);
                            todo!()
                        }
                    }
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
                    let index = self.state.allocate_string(string.into());
                    self.state.stack.push(VmData::String(index));
                    //self.state.collect_garbage();
                }

                Code::CHAR => {
                    let char = self.state.next_instruction() as char;
                    self.state.stack.push(VmData::Char(char));
                }

                Code::FREE => {
                    if let Some(item) = self.state.stack.pop() {
                        match item {
                            VmData::String(index) => {
                                self.state.free_heap(index);
                            }
                            VmData::List(index) => {
                                self.state.free_heap(index);
                            }
                            _ => {
                                todo!()
                            }
                        }
                    }
                }

                Code::CLONE => {
                    if let Some(item) = self.state.stack.pop() {
                        match item {
                            VmData::String(index) => {
                                let clone = self.state.allocate_new_heap();
                                self.state.copy_heap(index, clone);
                                self.state.stack.push(VmData::String(clone))
                            }
                            VmData::List(index) => {
                                let mut newarray = vec![];
                                match self.state.get_ref(index).clone() {
                                    Heap::List(vec) => {
                                        for item in vec {
                                            let item_clone_index = self.state.allocate_new_heap();
                                            self.state.copy_heap(item, item_clone_index);
                                            newarray.push(item_clone_index);
                                        }
                                    }
                                    _ => {
                                        todo!()
                                    }
                                }
                                let clone = self.state.allocate_array(newarray);
                                self.state.stack.push(VmData::List(clone))
                            }
                            a => {
                                // put it back
                                self.state.stack.push(a);
                            }
                        }
                    }
                }
                Code::NONE => {
                    self.state.stack.push(VmData::None);
                }
                error => {
                    dbg!(error);
                }
            }

            // dbg!(&self.state.stack);
            // dbg!(&self.state.program[self.state.current_instruction]);
        }
        //dbg!(&self.state.heap, self.state.heap.len(), self.state.threshold, self.state.gc_count);
        //self.state.collect_garbage();
        // dbg!(&self.state.heap.len());
        // dbg!(&self.state.gc_count);
        // dbg!(&self.state.garbage_collected);
        // dbg!(&self.state.threshold);
        // dbg!(&self.state.used_data);
        Ok(())
    }

    #[inline(always)]
    pub fn run_debug(&mut self) -> Result<(), NovaError> {
        let mut tick = 0;
        let mut input = String::new();
        loop {
            println!(
                "Current Instruction: {} Tick: {}",
                byte_to_string(self.state.program[self.state.current_instruction]),
                tick
            );
            println!("Stack: {:?}", &self.state.stack);
            // Read a line from the standard input and discard it.
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");

            // Attempt to clear the terminal screen
            // if let Err(e) = execute!(std::io::stdout(), Clear(ClearType::All)) {
            //     eprintln!("Failed to clear the terminal screen: {}", e);
            // }
            match self.state.next_instruction() {
                Code::ISSOME => {
                    if let Some(value) = self.state.stack.pop() {
                        match value {
                            VmData::None => self.state.stack.push(VmData::Bool(false)),
                            _ => self.state.stack.push(VmData::Bool(true)),
                        }
                    }
                }
                Code::UNWRAP => {
                    if let Some(value) = self.state.stack.last() {
                        if value == &VmData::None {
                            panic!()
                        }
                    }
                }
                Code::DUP => self.state.stack.push(*self.state.stack.last().unwrap()),
                Code::POP => {
                    self.state.stack.pop();
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
                    self.state.stack.push(VmData::Int(int));
                }

                Code::STACKREF => {
                    let index = u32::from_le_bytes(self.state.next_arr());
                    self.state.stack.push(VmData::StackAddress(index as usize));
                }

                // takes item and stores it into stack at location
                // with offset
                Code::STORE => {
                    let index = u32::from_le_bytes(self.state.next_arr());

                    let data = self.state.stack.pop().unwrap();
                    //dbg!(&data,index);
                    self.state.stack[self.state.offset + index as usize] = data;
                }

                // gets the data from a local index in the stack
                // from offset
                Code::GET => {
                    let index = u32::from_le_bytes(self.state.next_arr());
                    let item = &self.state.stack[self.state.offset + index as usize];
                    //dbg!(&item);
                    self.state.stack.push(*item);
                }

                // jumps back to the callsite of a function
                Code::RET => {
                    let with_return = self.state.next_instruction();
                    if let Some(destination) = self.state.callstack.pop() {
                        if with_return == 1 {
                            self.state.deallocate_registers_with_return();
                        } else {
                            self.state.deallocate_registers();
                        }
                        self.state.goto(destination);
                        //dbg!(&self.state.stack);
                    } else {
                        break;
                    }
                }

                // i think you can figure this one out
                Code::PRINT => {
                    let item = self.state.stack.pop().unwrap();
                    match item {
                        VmData::Function(v) => {
                            println!("function pointer: {v}")
                        }
                        VmData::Int(v) => {
                            println!("{v}")
                        }
                        VmData::Float(v) => {
                            println!("{v}")
                        }
                        VmData::Bool(v) => {
                            println!("{v}")
                        }
                        VmData::None => {
                            println!("None")
                        }
                        VmData::List(index) => {
                            if let Heap::List(array) = self.state.get_ref(index) {
                                print!("[");
                                for (index, item) in array.iter().enumerate() {
                                    if index > 0 {
                                        print!(", ");
                                    }
                                    print!("{:?}", self.state.get_ref(*item));
                                }
                                print!("]");
                                io::stdout().flush().expect("");
                            }
                        }
                        VmData::String(index) => {
                            if let Heap::String(str) = self.state.get_ref(index) {
                                println!("{str}")
                            }
                        }
                        VmData::Closure(_) => todo!(),
                        VmData::StackAddress(_) => todo!(),
                        VmData::Struct(_) => todo!(),
                        VmData::Char(_) => todo!(),
                    }
                }

                Code::FADD => {
                    if let (Some(VmData::Float(v1)), Some(VmData::Float(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        let result = v1 + v2;
                        self.state.stack.push(VmData::Float(result))
                    }
                }

                Code::FSUB => {
                    if let (Some(VmData::Float(v1)), Some(VmData::Float(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        let result = v2 - v1;
                        self.state.stack.push(VmData::Float(result))
                    }
                }

                Code::FMUL => {
                    if let (Some(VmData::Float(v1)), Some(VmData::Float(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        let result = v1 * v2;
                        self.state.stack.push(VmData::Float(result))
                    }
                }

                Code::FDIV => {
                    if let (Some(VmData::Float(v1)), Some(VmData::Float(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        let result = v2 / v1;
                        self.state.stack.push(VmData::Float(result))
                    }
                }

                Code::IADD => {
                    if let (Some(VmData::Int(v1)), Some(VmData::Int(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        let result = v1 + v2;
                        self.state.stack.push(VmData::Int(result))
                    }
                }

                Code::ISUB => {
                    if let (Some(VmData::Int(v1)), Some(VmData::Int(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        let result = v2 - v1;
                        self.state.stack.push(VmData::Int(result))
                    }
                }

                Code::IMUL => {
                    if let (Some(VmData::Int(v1)), Some(VmData::Int(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        let result = v1 * v2;
                        self.state.stack.push(VmData::Int(result))
                    }
                }

                Code::IDIV => {
                    if let (Some(VmData::Int(v1)), Some(VmData::Int(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        let result = v2 / v1;
                        self.state.stack.push(VmData::Int(result))
                    }
                }

                Code::STOREGLOBAL => {
                    let index = u32::from_le_bytes(self.state.next_arr());
                    let item = self.state.stack.pop().unwrap();
                    self.state.stack[index as usize] = item;
                }

                Code::FUNCTION => {
                    self.state
                        .stack
                        .push(VmData::Function(self.state.current_instruction + 4));

                    let jump = u32::from_le_bytes(self.state.next_arr());

                    self.state.current_instruction += jump as usize;
                }

                Code::CLOSURE => {
                    if let Some(VmData::List(list)) = self.state.stack.pop() {
                        self.state.gclock = true;
                        let closure = self.state.allocate_new_heap();
                        self.state.heap[closure] =
                            Heap::Closure(self.state.current_instruction + 4, list);
                        self.state.stack.push(VmData::Closure(closure));
                        self.state.gclock = false;
                        self.state.collect_garbage();
                        let jump = u32::from_le_bytes(self.state.next_arr());
                        self.state.current_instruction += jump as usize;
                    } else {
                        todo!()
                    }
                }

                Code::DIRECTCALL => {
                    self.state
                        .callstack
                        .push(self.state.current_instruction + 4);
                    let index = u32::from_le_bytes(self.state.next_arr());
                    if let VmData::Function(target) = self.state.stack[index as usize] {
                        self.state.goto(target);
                    }
                }

                Code::TAILCALL => {
                    let index = u32::from_le_bytes(self.state.next_arr());
                    if let VmData::Function(target) = self.state.stack[index as usize] {
                        self.state.goto(target);
                    }
                }

                Code::ILSS => {
                    if let (Some(VmData::Int(v1)), Some(VmData::Int(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        let result = v2 < v1;
                        self.state.stack.push(VmData::Bool(result))
                    }
                }

                Code::IGTR => {
                    if let (Some(VmData::Int(v1)), Some(VmData::Int(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        let result = v2 > v1;
                        self.state.stack.push(VmData::Bool(result))
                    }
                }

                Code::FLSS => {
                    if let (Some(VmData::Float(v1)), Some(VmData::Float(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        let result = v2 < v1;
                        self.state.stack.push(VmData::Bool(result))
                    }
                }

                Code::FGTR => {
                    if let (Some(VmData::Float(v1)), Some(VmData::Float(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        let result = v2 > v1;
                        self.state.stack.push(VmData::Bool(result))
                    }
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
                    if let VmData::Bool(test) = self.state.stack.pop().unwrap() {
                        if !test {
                            self.state.current_instruction += jump as usize;
                        }
                    }
                }

                Code::TRUE => {
                    self.state.stack.push(VmData::Bool(true));
                }

                Code::FALSE => {
                    self.state.stack.push(VmData::Bool(false));
                }

                Code::EQUALS => {
                    if let (Some(v1), Some(v2)) = (self.state.stack.pop(), self.state.stack.pop()) {
                        match (v1, v2) {
                            (VmData::String(i1), VmData::String(i2)) => {
                                let s1 = self.state.heap[i1].get_string();
                                let s2 = self.state.heap[i2].get_string();
                                let result = s1 == s2;
                                self.state.stack.push(VmData::Bool(result))
                            }
                            _ => {
                                let result = v2 == v1;
                                self.state.stack.push(VmData::Bool(result))
                            }
                        }
                    }
                }

                Code::AND => {
                    if let (Some(VmData::Bool(v1)), Some(VmData::Bool(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        self.state.stack.push(VmData::Bool(v1 && v2))
                    }
                }

                Code::OR => {
                    if let (Some(VmData::Bool(v1)), Some(VmData::Bool(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        self.state.stack.push(VmData::Bool(v1 || v2))
                    }
                }

                Code::NOT => {
                    if let Some(VmData::Bool(b)) = self.state.stack.pop() {
                        self.state.stack.push(VmData::Bool(!b))
                    }
                }

                Code::NEG => {
                    if let Some(value) = self.state.stack.pop() {
                        match value {
                            VmData::Int(v) => self.state.stack.push(VmData::Int(-v)),
                            VmData::Float(v) => self.state.stack.push(VmData::Float(-v)),
                            _ => {}
                        }
                    }
                }

                Code::IMODULO => {
                    if let (Some(VmData::Int(v1)), Some(VmData::Int(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        let result = v2.modulo(v1);
                        self.state.stack.push(VmData::Int(result))
                    }
                }

                Code::ASSIGN => {
                    if let (Some(destination), Some(value)) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        match (value, destination) {
                            (item, VmData::StackAddress(index)) => {
                                dbg!(self.state.stack[self.state.offset + index]);
                                match (self.state.stack[self.state.offset + index], value) {
                                    (VmData::List(_), VmData::StackAddress(_)) => todo!(),
                                    (VmData::List(_), VmData::Function(_)) => todo!(),
                                    (VmData::List(_), VmData::Closure(_)) => todo!(),
                                    (VmData::List(d), VmData::List(v)) => {
                                        self.state.heap[d] = self.state.heap[v].clone()
                                    }
                                    (VmData::List(_), VmData::Struct(_)) => todo!(),
                                    _ => self.state.stack[self.state.offset + index] = item,
                                }
                                //self.state.stack[self.state.offset + index as usize] = item
                            }
                            (item, VmData::List(index)) => {
                                match item {
                                    VmData::Function(_) => todo!(),
                                    VmData::Int(v) => self.state.heap[index] = Heap::Int(v),
                                    VmData::Float(_) => todo!(),
                                    VmData::Bool(_) => todo!(),
                                    VmData::List(v) => {
                                        self.state.heap[index] = Heap::ListAddress(v)
                                    }
                                    VmData::None => todo!(),
                                    VmData::String(v) => {
                                        self.state.heap[index] = Heap::StringAddress(v)
                                    }
                                    VmData::Closure(_) => todo!(),
                                    VmData::StackAddress(_) => todo!(),
                                    VmData::Struct(_) => todo!(),
                                    VmData::Char(_) => todo!(),
                                };
                            }
                            (a, b) => {
                                dbg!(a, b, self.state.current_instruction);
                                todo!()
                            }
                        }
                    }
                }

                Code::NEWLIST => {
                    let size = u64::from_le_bytes(self.state.next_arr());
                    let mut myarray = vec![];
                    self.state.gclock = true;
                    for _ in 0..size {
                        if let Some(value) = self.state.stack.pop() {
                            myarray.push(self.state.allocate_vmdata_to_heap(value))
                        } else {
                            todo!()
                        }
                    }
                    myarray.reverse();
                    let index = self.state.allocate_array(myarray);
                    self.state.stack.push(VmData::List(index));
                    self.state.gclock = false;
                }

                Code::PINDEX => {
                    if let (Some(array), Some(index)) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        match (array, index) {
                            (
                                VmData::StackAddress(array_index),
                                VmData::StackAddress(index_to_get),
                            ) => {
                                if let VmData::Int(index_to_get) =
                                    self.state.stack[self.state.offset + index_to_get]
                                {
                                    if let VmData::List(newindex) =
                                        &self.state.stack[self.state.offset + array_index]
                                    {
                                        if let Heap::List(array) = self.state.get_ref(*newindex) {
                                            self.state
                                                .stack
                                                .push(VmData::List(array[index_to_get as usize]))
                                        }
                                    }
                                } else {
                                    todo!()
                                }
                            }
                            (VmData::StackAddress(array_index), VmData::Int(index_to_get)) => {
                                if let VmData::List(newindex) =
                                    &self.state.stack[self.state.offset + array_index]
                                {
                                    if let Heap::List(array) = self.state.get_ref(*newindex) {
                                        self.state
                                            .stack
                                            .push(VmData::List(array[index_to_get as usize]))
                                    }
                                }
                            }
                            (VmData::List(array_index), VmData::Int(index_to_get)) => {
                                if let Heap::ListAddress(newindex) = self.state.get_ref(array_index)
                                {
                                    if let Heap::List(array) = self.state.get_ref(*newindex) {
                                        self.state
                                            .stack
                                            .push(VmData::List(array[index_to_get as usize]))
                                    }
                                } else {
                                    todo!()
                                }
                            }
                            (a, b) => {
                                dbg!(a, b);
                                todo!()
                            }
                        }
                    }
                }

                Code::LINDEX => {
                    if let (Some(array), Some(index)) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        match (array, index) {
                            (VmData::List(array), VmData::Int(index_to)) => {
                                match self.state.get_ref(array).clone() {
                                    Heap::Function(_) => todo!(),
                                    Heap::Int(v) => self.state.stack.push(VmData::Int(v)),
                                    Heap::Float(_) => todo!(),
                                    Heap::Bool(_) => todo!(),
                                    Heap::ListAddress(_) => todo!(),
                                    Heap::StringAddress(_) => todo!(),
                                    Heap::List(array) => {
                                        let item =
                                            self.state.get_ref(array[index_to as usize]).clone();
                                        match item {
                                            Heap::Function(v) => {
                                                self.state.stack.push(VmData::Function(v))
                                            }
                                            Heap::Int(v) => self.state.stack.push(VmData::Int(v)),
                                            Heap::Float(v) => {
                                                self.state.stack.push(VmData::Float(v))
                                            }
                                            Heap::Bool(v) => self.state.stack.push(VmData::Bool(v)),
                                            Heap::ListAddress(v) => {
                                                self.state.stack.push(VmData::List(v))
                                            }
                                            Heap::List(_) => todo!(),
                                            Heap::String(_) => todo!(),
                                            Heap::None => self.state.stack.push(VmData::None),
                                            Heap::StringAddress(v) => {
                                                self.state.stack.push(VmData::String(v))
                                            }
                                            Heap::Closure(_, _) => todo!(),
                                            Heap::ClosureAddress(v) => {
                                                self.state.stack.push(VmData::Closure(v))
                                            }
                                            Heap::Struct(_, _) => todo!(),
                                            Heap::StructAddress(_) => todo!(),
                                            Heap::Char(_) => todo!(),
                                        }
                                    }
                                    Heap::String(_) => todo!(),
                                    Heap::None => todo!(),
                                    Heap::Closure(_, _) => todo!(),
                                    Heap::ClosureAddress(_) => todo!(),
                                    Heap::Struct(_, _) => todo!(),
                                    Heap::StructAddress(_) => todo!(),
                                    Heap::Char(_) => todo!(),
                                }
                            }
                            (a, b) => {
                                dbg!(a, b);
                                todo!()
                            }
                        }
                    } else {
                        todo!()
                    }
                }

                Code::FLOAT => {
                    let fl = f64::from_le_bytes(self.state.next_arr());
                    self.state.stack.push(VmData::Float(fl));
                }

                Code::GETGLOBAL => {
                    let index = u32::from_le_bytes(self.state.next_arr());
                    self.state.stack.push(self.state.stack[index as usize]);
                }

                Code::CALL => {
                    if let Some(callee) = self.state.stack.pop() {
                        match callee {
                            VmData::Closure(index) => {
                                if let Some(Heap::Closure(target, captured)) =
                                    self.state.heap.get(index)
                                {
                                    dbg!(&self.state.heap[*captured]);
                                    if let Heap::List(list) = &self.state.heap[*captured] {
                                        for i in list {
                                            self.state.stack.push(self.state.to_vmdata(*i))
                                        }
                                        self.state.callstack.push(self.state.current_instruction);
                                        self.state.goto(*target);
                                    } else {
                                        dbg!(target, callee, captured);
                                        todo!()
                                    }
                                } else {
                                    todo!()
                                }
                            }
                            VmData::Function(target) => {
                                self.state.callstack.push(self.state.current_instruction);
                                self.state.goto(target);
                            }
                            a => {
                                dbg!(a);
                                todo!()
                            }
                        }
                    } else {
                        todo!()
                    }
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
                    let index = self.state.allocate_string(string.into());
                    self.state.stack.push(VmData::String(index));
                    //self.state.collect_garbage();
                }

                Code::FREE => {
                    if let Some(item) = self.state.stack.pop() {
                        match item {
                            VmData::String(index) => {
                                self.state.free_heap(index);
                            }
                            VmData::List(index) => {
                                self.state.free_heap(index);
                            }
                            _ => {
                                todo!()
                            }
                        }
                    }
                }

                Code::CLONE => {
                    if let Some(item) = self.state.stack.pop() {
                        match item {
                            VmData::String(index) => {
                                let clone = self.state.allocate_new_heap();
                                self.state.copy_heap(index, clone);
                                self.state.stack.push(VmData::String(clone))
                            }
                            VmData::List(index) => {
                                let clone = self.state.allocate_new_heap();
                                self.state.copy_heap(index, clone);
                                self.state.stack.push(VmData::List(clone))
                            }
                            _ => {
                                todo!()
                            }
                        }
                    }
                }
                Code::NONE => {
                    self.state.stack.push(VmData::None);
                }
                Code::NATIVE => {
                    let index = u64::from_le_bytes(self.state.next_arr());

                    match self.native_functions[index as usize](&mut self.state) {
                        Ok(_) => {}
                        Err(error) => return Err(error),
                    }
                }
                error => {
                    dbg!(error);
                }
            }

            // dbg!(&self.state.stack);
            // dbg!(&self.state.program[self.state.current_instruction]);

            tick += 1;
        }
        //dbg!(&self.state.heap, self.state.heap.len(), self.state.threshold, self.state.gc_count);
        //self.state.collect_garbage();
        // dbg!(&self.state.heap.len());
        // dbg!(&self.state.gc_count);
        // dbg!(&self.state.garbage_collected);
        //dbg!(&self.state.used_data);

        // Create a mutable variable to hold the user's input.

        Ok(())
    }
}
