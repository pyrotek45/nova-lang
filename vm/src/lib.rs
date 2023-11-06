pub mod state;
pub type CallBack = fn(state: &mut state::State) -> Result<(), NovaError>;

use std::{io, os};

use common::{
    code::{byte_to_string, Code},
    error::NovaError,
};

use modulo::Mod;
use state::Heap;

use crate::state::VmData;

#[derive(Debug, Clone)]
pub struct Vm {
    pub native_functions: Vec<CallBack>,
    pub state: state::State,
}

pub fn new() -> Vm {
    Vm {
        native_functions: vec![],
        state: state::new(),
    }
}

impl Vm {
    #[inline(always)]
    pub fn run(&mut self) -> Result<(), NovaError> {
        loop {
            // /dbg!(&self.state.stack, &self.state.program[self.state.current_instruction]);
            match self.state.next() {
                Code::CONCAT => {
                    if let (Some(VmData::String(s1)), Some(VmData::String(s2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        if let (Heap::String(str2), Heap::String(mut str1)) =
                            (self.state.deref(s1), self.state.deref(s2))
                        {
                            str1.push_str(&str2);
                            let index = self.state.allocate_string(str1);
                            self.state.stack.push(VmData::String(index));
                        } else {
                            panic!()
                        }
                    } else {
                        panic!()
                    }
                }
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
                        match value {
                            VmData::None => {
                                println!("ERROR: Tried to unwrap a none value");
                                println!("ERROR: exiting program");
                                std::process::exit(1)
                            }
                            _ => {}
                        }
                    }
                }
                Code::DUP => self
                    .state
                    .stack
                    .push(self.state.stack.last().unwrap().clone()),
                Code::POP => {
                    self.state.stack.pop();
                }
                Code::NATIVE => {
                    let index = usize::from_le_bytes([
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                    ]);

                    match self.native_functions[index](&mut self.state) {
                        Ok(_) => {}
                        Err(error) => return Err(error),
                    }
                }

                // sets up the stack with empty values for use later with local variables
                Code::ALLOCATEGLOBAL => {
                    let allocations = u32::from_le_bytes([
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                    ]);
                    self.state.alloc_locals(allocations as usize);
                }
                // sets up the stack with empty values for use later with local variables
                Code::ALLOCLOCALS => {
                    let allocations = u32::from_le_bytes([
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                    ]);
                    self.state.alloc_locals(allocations as usize);
                }
                // sets up the stack with empty values for use later with local variables
                Code::OFFSET => {
                    let offset = u32::from_le_bytes([
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                    ]);
                    let locals = u32::from_le_bytes([
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                    ]);
                    self.state.offset_locals(offset as usize, locals as usize);
                }
                // pushes a constant integer to the stack
                Code::INTEGER => {
                    let int = i64::from_le_bytes([
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                    ]);
                    self.state.stack.push(VmData::Int(int));
                }

                Code::STACKREF => {
                    let index = u32::from_le_bytes([
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                    ]);
                    self.state.stack.push(VmData::StackRef(index as usize));
                }

                // takes item and stores it into stack at location
                // with offset
                Code::STORE => {
                    let index = u32::from_le_bytes([
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                    ]);

                    let data = self.state.stack.pop().unwrap();
                    //dbg!(&data,index);
                    self.state.stack[self.state.offset + index as usize] = data;
                }

                // gets the data from a local index in the stack
                // from offset
                Code::GET => {
                    let index = u32::from_le_bytes([
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                    ]);
                    let item = &self.state.stack[self.state.offset + index as usize];
                    //dbg!(&item);
                    self.state.stack.push(item.clone());
                }

                // jumps back to the callsite of a function
                Code::RET => {
                    let with_return = self.state.next();
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
                            if let Heap::List(array) = self.state.deref(index) {
                                print!("[");
                                for (index, item) in array.iter().enumerate() {
                                    if index > 0 {
                                        print!(", ");
                                    }
                                    print!("{:?}", self.state.deref(*item));
                                }
                                print!("]\n");
                            }
                        }
                        VmData::String(index) => {
                            if let Heap::String(str) = self.state.deref(index) {
                                println!("{str}")
                            }
                        }
                        VmData::Closure(_) => todo!(),
                        VmData::StackRef(_) => todo!(),
                    }
                }

                Code::FADD => {
                    if let (Some(VmData::Float(v1)), Some(VmData::Float(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        let result = v1 + v2;
                        self.state.stack.push(VmData::Float(result))
                    } else {
                        return Err(common::error::runtime_error(
                            "Not enough arguments for integer addition".to_string(),
                        ));
                    }
                }

                Code::FSUB => {
                    if let (Some(VmData::Float(v1)), Some(VmData::Float(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        let result = v2 - v1;
                        self.state.stack.push(VmData::Float(result))
                    } else {
                        return Err(common::error::runtime_error(
                            "Not enough arguments for integer sub".to_string(),
                        ));
                    }
                }

                Code::FMUL => {
                    if let (Some(VmData::Float(v1)), Some(VmData::Float(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        let result = v1 * v2;
                        self.state.stack.push(VmData::Float(result))
                    } else {
                        return Err(common::error::runtime_error(
                            "Not enough arguments for integer mul".to_string(),
                        ));
                    }
                }

                Code::FDIV => {
                    if let (Some(VmData::Float(v1)), Some(VmData::Float(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        let result = v2 / v1;
                        self.state.stack.push(VmData::Float(result))
                    } else {
                        return Err(common::error::runtime_error(
                            "Not enough arguments for integer div".to_string(),
                        ));
                    }
                }

                Code::IADD => {
                    if let (Some(VmData::Int(v1)), Some(VmData::Int(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        let result = v1 + v2;
                        self.state.stack.push(VmData::Int(result))
                    } else {
                        return Err(common::error::runtime_error(
                            "Not enough arguments for integer addition".to_string(),
                        ));
                    }
                }

                Code::ISUB => {
                    if let (Some(VmData::Int(v1)), Some(VmData::Int(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        let result = v2 - v1;
                        self.state.stack.push(VmData::Int(result))
                    } else {
                        return Err(common::error::runtime_error(
                            "Not enough arguments for integer sub".to_string(),
                        ));
                    }
                }

                Code::IMUL => {
                    if let (Some(VmData::Int(v1)), Some(VmData::Int(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        let result = v1 * v2;
                        self.state.stack.push(VmData::Int(result))
                    } else {
                        return Err(common::error::runtime_error(
                            "Not enough arguments for integer mul".to_string(),
                        ));
                    }
                }

                Code::IDIV => {
                    if let (Some(VmData::Int(v1)), Some(VmData::Int(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        let result = v2 / v1;
                        self.state.stack.push(VmData::Int(result))
                    } else {
                        return Err(common::error::runtime_error(
                            "Not enough arguments for integer div".to_string(),
                        ));
                    }
                }

                Code::STOREGLOBAL => {
                    let index = u32::from_le_bytes([
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                    ]);
                    let item = self.state.stack.pop().unwrap();
                    self.state.stack[index as usize] = item;
                }

                Code::FUNCTION => {
                    self.state
                        .stack
                        .push(VmData::Function(self.state.current_instruction + 4));

                    let jump = u32::from_le_bytes([
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                    ]);

                    self.state.current_instruction += jump as usize;
                }

                Code::CLOSURE => {
                    if let Some(VmData::List(list)) = self.state.stack.pop() {
                        let closure = self.state.allocate_new_heap();
                        self.state.heap[closure] =
                            Heap::Closure(self.state.current_instruction + 4, list);
                        self.state.stack.push(VmData::Closure(closure));

                        let jump = u32::from_le_bytes([
                            self.state.next(),
                            self.state.next(),
                            self.state.next(),
                            self.state.next(),
                        ]);
                        self.state.current_instruction += jump as usize;
                    } else {
                        todo!()
                    }
                }

                Code::DIRECTCALL => {
                    self.state
                        .callstack
                        .push(self.state.current_instruction + 4);
                    let index = u32::from_le_bytes([
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                    ]);
                    if let VmData::Function(target) = self.state.stack[index as usize] {
                        self.state.goto(target);
                    }
                }

                Code::TAILCALL => {
                    let index = u32::from_le_bytes([
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                    ]);
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
                    } else {
                        return Err(common::error::runtime_error(
                            "Not enough arguments for ILSS".to_string(),
                        ));
                    }
                }

                Code::IGTR => {
                    if let (Some(VmData::Int(v1)), Some(VmData::Int(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        let result = v2 > v1;
                        self.state.stack.push(VmData::Bool(result))
                    } else {
                        return Err(common::error::runtime_error(
                            "Not enough arguments for ILSS".to_string(),
                        ));
                    }
                }

                Code::FLSS => {
                    if let (Some(VmData::Float(v1)), Some(VmData::Float(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        let result = v2 < v1;
                        self.state.stack.push(VmData::Bool(result))
                    } else {
                        return Err(common::error::runtime_error(
                            "Not enough arguments for FLSS".to_string(),
                        ));
                    }
                }

                Code::FGTR => {
                    if let (Some(VmData::Float(v1)), Some(VmData::Float(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        let result = v2 > v1;
                        self.state.stack.push(VmData::Bool(result))
                    } else {
                        return Err(common::error::runtime_error(
                            "Not enough arguments for FLSS".to_string(),
                        ));
                    }
                }

                Code::JMP => {
                    let jump = u32::from_le_bytes([
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                    ]);
                    self.state.current_instruction += jump as usize;
                }
                Code::BJMP => {
                    let jump = u32::from_le_bytes([
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                    ]);
                    self.state.current_instruction -= jump as usize;
                }
                Code::JUMPIFFALSE => {
                    let jump = u32::from_le_bytes([
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                    ]);
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
                    } else {
                        return Err(common::error::runtime_error(
                            "Not enough arguments for EQ".to_string(),
                        ));
                    }
                }

                Code::NOT => {
                    if let Some(bool) = self.state.stack.pop() {
                        match bool {
                            VmData::Bool(b) => {
                                if b {
                                    self.state.stack.push(VmData::Bool(false))
                                } else {
                                    self.state.stack.push(VmData::Bool(true))
                                }
                            }
                            _ => {
                                return Err(common::error::runtime_error(
                                    "Cannot 'NOT' a non bool value".to_string(),
                                ));
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

                Code::NEG => {
                    if let Some(value) = self.state.stack.pop() {
                        match value {
                            VmData::Int(v) => self.state.stack.push(VmData::Int(-v)),
                            VmData::Float(v) => self.state.stack.push(VmData::Float(-v)),
                            _ => {
                                return Err(common::error::runtime_error(
                                    "Cannot 'NEG' a non bool value".to_string(),
                                ));
                            }
                        }
                    }
                }

                Code::IMODULO => {
                    if let (Some(VmData::Int(v1)), Some(VmData::Int(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        let result = v2.modulo(v1);
                        self.state.stack.push(VmData::Int(result))
                    } else {
                        return Err(common::error::runtime_error(
                            "Not enough arguments for integer div".to_string(),
                        ));
                    }
                }

                Code::ASSIGN => {
                    if let (Some(destination), Some(value)) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        match (value, destination) {
                            (item, VmData::StackRef(index)) => {
                                self.state.stack[self.state.offset + index as usize] = item
                            }
                            (item, VmData::List(index)) => {
                                match item {
                                    VmData::Function(_) => todo!(),
                                    VmData::Int(v) => {
                                        self.state.heap[index as usize] = Heap::Int(v)
                                    }
                                    VmData::Float(_) => todo!(),
                                    VmData::Bool(_) => todo!(),
                                    VmData::List(v) => {
                                        self.state.heap[index as usize] = Heap::ListRef(v)
                                    }
                                    VmData::None => todo!(),
                                    VmData::String(_) => todo!(),
                                    VmData::Closure(_) => todo!(),
                                    VmData::StackRef(_) => todo!(),
                                };
                            }
                            (a, b) => {
                                dbg!(a, b, self.state.current_instruction);
                                todo!()
                            }
                        }
                    } else {
                        return Err(common::error::runtime_error(
                            "Not enough arguments for assignment".to_string(),
                        ));
                    }
                }

                Code::NEWLIST => {
                    let size = usize::from_le_bytes([
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                    ]);
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
                    if let (Some(array), Some(index)) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        match (array, index) {
                            (VmData::StackRef(array_index), VmData::StackRef(index_to_get)) => {
                                if let VmData::Int(index_to_get) =
                                    self.state.stack[self.state.offset + index_to_get as usize]
                                {
                                    if let VmData::List(newindex) =
                                        &self.state.stack[self.state.offset + array_index as usize]
                                    {
                                        if let Heap::List(array) = self.state.deref(*newindex) {
                                            self.state
                                                .stack
                                                .push(VmData::List(array[index_to_get as usize]))
                                        }
                                    }
                                } else {
                                    todo!()
                                }
                            }
                            (VmData::StackRef(array_index), VmData::Int(index_to_get)) => {
                                if let VmData::List(newindex) =
                                    &self.state.stack[self.state.offset + array_index as usize]
                                {
                                    if let Heap::List(array) = self.state.deref(*newindex) {
                                        self.state
                                            .stack
                                            .push(VmData::List(array[index_to_get as usize]))
                                    }
                                }
                            }
                            (VmData::List(array_index), VmData::Int(index_to_get)) => {
                                if let Heap::ListRef(newindex) =
                                    self.state.deref(array_index as usize)
                                {
                                    if let Heap::List(array) = self.state.deref(newindex) {
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
                    } else {
                        return Err(common::error::runtime_error(
                            "Not enough arguments for assignment".to_string(),
                        ));
                    }
                }

                Code::LINDEX => {
                    if let (Some(array), Some(index)) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        match (array, index) {
                            (VmData::List(array), VmData::Int(index_to)) => {
                                match self.state.deref(array as usize) {
                                    Heap::Function(_) => todo!(),
                                    Heap::Int(v) => self.state.stack.push(VmData::Int(v)),
                                    Heap::Float(_) => todo!(),
                                    Heap::Bool(_) => todo!(),
                                    Heap::ListRef(_) => todo!(),
                                    Heap::StringRef(_) => todo!(),
                                    Heap::List(array) => {
                                        let item = self.state.deref(array[index_to as usize]);
                                        match item {
                                            Heap::Function(v) => {
                                                self.state.stack.push(VmData::Function(v))
                                            }
                                            Heap::Int(v) => self.state.stack.push(VmData::Int(v)),
                                            Heap::Float(v) => {
                                                self.state.stack.push(VmData::Float(v))
                                            }
                                            Heap::Bool(v) => self.state.stack.push(VmData::Bool(v)),
                                            Heap::ListRef(v) => {
                                                self.state.stack.push(VmData::List(v))
                                            }
                                            Heap::List(_) => todo!(),
                                            Heap::String(_) => todo!(),
                                            Heap::None => todo!(),
                                            Heap::StringRef(v) => {
                                                self.state.stack.push(VmData::String(v))
                                            }
                                            Heap::Closure(_, _) => todo!(),
                                            Heap::ClosureRef(_) => todo!(),
                                        }
                                    }
                                    Heap::String(_) => todo!(),
                                    Heap::None => todo!(),
                                    Heap::Closure(_, _) => todo!(),
                                    Heap::ClosureRef(_) => todo!(),
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
                    let fl = f64::from_le_bytes([
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                    ]);
                    self.state.stack.push(VmData::Float(fl));
                }

                Code::GETGLOBAL => {
                    let index = u32::from_le_bytes([
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                    ]);
                    self.state
                        .stack
                        .push(self.state.stack[index as usize].clone());
                }

                Code::CALL => {
                    if let Some(callee) = self.state.stack.pop() {
                        match callee {
                            VmData::Closure(index) => {
                                if let Some(Heap::Closure(target, captured)) =
                                    self.state.heap.get(index)
                                {
                                    if let Heap::List(list) = &self.state.heap[*captured] {
                                        for i in list {
                                            self.state.stack.push(self.state.to_vmdata(*i))
                                        }
                                        self.state.callstack.push(self.state.current_instruction);
                                        self.state.goto(*target);
                                    } else {
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
                    let size = usize::from_le_bytes([
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                    ]);
                    for _ in 0..size {
                        string.push(self.state.next());
                    }
                    let string = match String::from_utf8(string) {
                        Ok(ok) => ok,
                        Err(_) => todo!(),
                    };
                    let index = self.state.allocate_string(string);
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
        //dbg!(&self.state.used_data);
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
            match self.state.next() {
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
                        match value {
                            VmData::None => panic!(),
                            _ => {}
                        }
                    }
                }
                Code::DUP => self
                    .state
                    .stack
                    .push(self.state.stack.last().unwrap().clone()),
                Code::POP => {
                    self.state.stack.pop();
                }
                // sets up the stack with empty values for use later with local variables
                Code::ALLOCATEGLOBAL => {
                    let allocations = u32::from_le_bytes([
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                    ]);
                    self.state.alloc_locals(allocations as usize);
                }
                // sets up the stack with empty values for use later with local variables
                Code::ALLOCLOCALS => {
                    let allocations = u32::from_le_bytes([
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                    ]);
                    self.state.alloc_locals(allocations as usize);
                }
                // sets up the stack with empty values for use later with local variables
                Code::OFFSET => {
                    let offset = u32::from_le_bytes([
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                    ]);
                    let locals = u32::from_le_bytes([
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                    ]);
                    self.state.offset_locals(offset as usize, locals as usize);
                }
                // pushes a constant integer to the stack
                Code::INTEGER => {
                    let int = i64::from_le_bytes([
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                    ]);
                    self.state.stack.push(VmData::Int(int));
                }

                Code::STACKREF => {
                    let index = u32::from_le_bytes([
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                    ]);
                    self.state.stack.push(VmData::StackRef(index as usize));
                }

                // takes item and stores it into stack at location
                // with offset
                Code::STORE => {
                    let index = u32::from_le_bytes([
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                    ]);

                    let data = self.state.stack.pop().unwrap();
                    //dbg!(&data,index);
                    self.state.stack[self.state.offset + index as usize] = data;
                }

                // gets the data from a local index in the stack
                // from offset
                Code::GET => {
                    let index = u32::from_le_bytes([
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                    ]);
                    let item = &self.state.stack[self.state.offset + index as usize];
                    //dbg!(&item);
                    self.state.stack.push(item.clone());
                }

                // jumps back to the callsite of a function
                Code::RET => {
                    let with_return = self.state.next();
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
                            if let Heap::List(array) = self.state.deref(index) {
                                print!("[");
                                for (index, item) in array.iter().enumerate() {
                                    if index > 0 {
                                        print!(", ");
                                    }
                                    print!("{:?}", self.state.deref(*item));
                                }
                                print!("]\n");
                            }
                        }
                        VmData::String(index) => {
                            if let Heap::String(str) = self.state.deref(index) {
                                println!("{str}")
                            }
                        }
                        VmData::Closure(_) => todo!(),
                        VmData::StackRef(_) => todo!(),
                    }
                }

                Code::FADD => {
                    if let (Some(VmData::Float(v1)), Some(VmData::Float(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        let result = v1 + v2;
                        self.state.stack.push(VmData::Float(result))
                    } else {
                        return Err(common::error::runtime_error(
                            "Not enough arguments for integer addition".to_string(),
                        ));
                    }
                }

                Code::FSUB => {
                    if let (Some(VmData::Float(v1)), Some(VmData::Float(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        let result = v2 - v1;
                        self.state.stack.push(VmData::Float(result))
                    } else {
                        return Err(common::error::runtime_error(
                            "Not enough arguments for integer sub".to_string(),
                        ));
                    }
                }

                Code::FMUL => {
                    if let (Some(VmData::Float(v1)), Some(VmData::Float(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        let result = v1 * v2;
                        self.state.stack.push(VmData::Float(result))
                    } else {
                        return Err(common::error::runtime_error(
                            "Not enough arguments for integer mul".to_string(),
                        ));
                    }
                }

                Code::FDIV => {
                    if let (Some(VmData::Float(v1)), Some(VmData::Float(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        let result = v2 / v1;
                        self.state.stack.push(VmData::Float(result))
                    } else {
                        return Err(common::error::runtime_error(
                            "Not enough arguments for integer div".to_string(),
                        ));
                    }
                }

                Code::IADD => {
                    if let (Some(VmData::Int(v1)), Some(VmData::Int(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        let result = v1 + v2;
                        self.state.stack.push(VmData::Int(result))
                    } else {
                        return Err(common::error::runtime_error(
                            "Not enough arguments for integer addition".to_string(),
                        ));
                    }
                }

                Code::ISUB => {
                    if let (Some(VmData::Int(v1)), Some(VmData::Int(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        let result = v2 - v1;
                        self.state.stack.push(VmData::Int(result))
                    } else {
                        return Err(common::error::runtime_error(
                            "Not enough arguments for integer sub".to_string(),
                        ));
                    }
                }

                Code::IMUL => {
                    if let (Some(VmData::Int(v1)), Some(VmData::Int(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        let result = v1 * v2;
                        self.state.stack.push(VmData::Int(result))
                    } else {
                        return Err(common::error::runtime_error(
                            "Not enough arguments for integer mul".to_string(),
                        ));
                    }
                }

                Code::IDIV => {
                    if let (Some(VmData::Int(v1)), Some(VmData::Int(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        let result = v2 / v1;
                        self.state.stack.push(VmData::Int(result))
                    } else {
                        return Err(common::error::runtime_error(
                            "Not enough arguments for integer div".to_string(),
                        ));
                    }
                }

                Code::STOREGLOBAL => {
                    let index = u32::from_le_bytes([
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                    ]);
                    let item = self.state.stack.pop().unwrap();
                    self.state.stack[index as usize] = item;
                }

                Code::FUNCTION => {
                    self.state
                        .stack
                        .push(VmData::Function(self.state.current_instruction + 4));

                    let jump = u32::from_le_bytes([
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                    ]);

                    self.state.current_instruction += jump as usize;
                }

                Code::CLOSURE => {
                    if let Some(VmData::List(list)) = self.state.stack.pop() {
                        let closure = self.state.allocate_new_heap();
                        self.state.heap[closure] =
                            Heap::Closure(self.state.current_instruction + 4, list);
                        self.state.stack.push(VmData::Closure(closure));

                        let jump = u32::from_le_bytes([
                            self.state.next(),
                            self.state.next(),
                            self.state.next(),
                            self.state.next(),
                        ]);
                        self.state.current_instruction += jump as usize;
                    } else {
                        todo!()
                    }
                }

                Code::DIRECTCALL => {
                    self.state
                        .callstack
                        .push(self.state.current_instruction + 4);
                    let index = u32::from_le_bytes([
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                    ]);
                    if let VmData::Function(target) = self.state.stack[index as usize] {
                        self.state.goto(target);
                    }
                }

                Code::TAILCALL => {
                    let index = u32::from_le_bytes([
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                    ]);
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
                    } else {
                        return Err(common::error::runtime_error(
                            "Not enough arguments for ILSS".to_string(),
                        ));
                    }
                }

                Code::IGTR => {
                    if let (Some(VmData::Int(v1)), Some(VmData::Int(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        let result = v2 > v1;
                        self.state.stack.push(VmData::Bool(result))
                    } else {
                        return Err(common::error::runtime_error(
                            "Not enough arguments for ILSS".to_string(),
                        ));
                    }
                }

                Code::FLSS => {
                    if let (Some(VmData::Float(v1)), Some(VmData::Float(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        let result = v2 < v1;
                        self.state.stack.push(VmData::Bool(result))
                    } else {
                        return Err(common::error::runtime_error(
                            "Not enough arguments for FLSS".to_string(),
                        ));
                    }
                }

                Code::FGTR => {
                    if let (Some(VmData::Float(v1)), Some(VmData::Float(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        let result = v2 > v1;
                        self.state.stack.push(VmData::Bool(result))
                    } else {
                        return Err(common::error::runtime_error(
                            "Not enough arguments for FLSS".to_string(),
                        ));
                    }
                }

                Code::JMP => {
                    let jump = u32::from_le_bytes([
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                    ]);
                    self.state.current_instruction += jump as usize;
                }
                Code::BJMP => {
                    let jump = u32::from_le_bytes([
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                    ]);
                    self.state.current_instruction -= jump as usize;
                }
                Code::JUMPIFFALSE => {
                    let jump = u32::from_le_bytes([
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                    ]);
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
                    } else {
                        return Err(common::error::runtime_error(
                            "Not enough arguments for EQ".to_string(),
                        ));
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
                    if let Some(bool) = self.state.stack.pop() {
                        match bool {
                            VmData::Bool(b) => {
                                if b {
                                    self.state.stack.push(VmData::Bool(false))
                                } else {
                                    self.state.stack.push(VmData::Bool(true))
                                }
                            }
                            _ => {
                                return Err(common::error::runtime_error(
                                    "Cannot 'NOT' a non bool value".to_string(),
                                ));
                            }
                        }
                    }
                }

                Code::NEG => {
                    if let Some(value) = self.state.stack.pop() {
                        match value {
                            VmData::Int(v) => self.state.stack.push(VmData::Int(-v)),
                            VmData::Float(v) => self.state.stack.push(VmData::Float(-v)),
                            _ => {
                                return Err(common::error::runtime_error(
                                    "Cannot 'NEG' a non bool value".to_string(),
                                ));
                            }
                        }
                    }
                }

                Code::IMODULO => {
                    if let (Some(VmData::Int(v1)), Some(VmData::Int(v2))) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        let result = v2.modulo(v1);
                        self.state.stack.push(VmData::Int(result))
                    } else {
                        return Err(common::error::runtime_error(
                            "Not enough arguments for integer div".to_string(),
                        ));
                    }
                }

                Code::ASSIGN => {
                    if let (Some(destination), Some(value)) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        match (value, destination) {
                            (item, VmData::StackRef(index)) => {
                                self.state.stack[self.state.offset + index as usize] = item
                            }
                            (item, VmData::List(index)) => {
                                match item {
                                    VmData::Function(_) => todo!(),
                                    VmData::Int(v) => {
                                        self.state.heap[index as usize] = Heap::Int(v)
                                    }
                                    VmData::Float(_) => todo!(),
                                    VmData::Bool(_) => todo!(),
                                    VmData::List(v) => {
                                        self.state.heap[index as usize] = Heap::ListRef(v)
                                    }
                                    VmData::None => todo!(),
                                    VmData::String(_) => todo!(),
                                    VmData::Closure(_) => todo!(),
                                    VmData::StackRef(_) => todo!(),
                                };
                            }
                            (a, b) => {
                                dbg!(a, b, self.state.current_instruction);
                                todo!()
                            }
                        }
                    } else {
                        return Err(common::error::runtime_error(
                            "Not enough arguments for assignment".to_string(),
                        ));
                    }
                }

                Code::NEWLIST => {
                    let size = usize::from_le_bytes([
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                    ]);
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
                    if let (Some(array), Some(index)) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        match (array, index) {
                            (VmData::StackRef(array_index), VmData::StackRef(index_to_get)) => {
                                if let VmData::Int(index_to_get) =
                                    self.state.stack[self.state.offset + index_to_get as usize]
                                {
                                    if let VmData::List(newindex) =
                                        &self.state.stack[self.state.offset + array_index as usize]
                                    {
                                        if let Heap::List(array) = self.state.deref(*newindex) {
                                            self.state
                                                .stack
                                                .push(VmData::List(array[index_to_get as usize]))
                                        }
                                    }
                                } else {
                                    todo!()
                                }
                            }
                            (VmData::StackRef(array_index), VmData::Int(index_to_get)) => {
                                if let VmData::List(newindex) =
                                    &self.state.stack[self.state.offset + array_index as usize]
                                {
                                    if let Heap::List(array) = self.state.deref(*newindex) {
                                        self.state
                                            .stack
                                            .push(VmData::List(array[index_to_get as usize]))
                                    }
                                }
                            }
                            (VmData::List(array_index), VmData::Int(index_to_get)) => {
                                if let Heap::ListRef(newindex) =
                                    self.state.deref(array_index as usize)
                                {
                                    if let Heap::List(array) = self.state.deref(newindex) {
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
                    } else {
                        return Err(common::error::runtime_error(
                            "Not enough arguments for assignment".to_string(),
                        ));
                    }
                }

                Code::LINDEX => {
                    if let (Some(array), Some(index)) =
                        (self.state.stack.pop(), self.state.stack.pop())
                    {
                        match (array, index) {
                            (VmData::List(array), VmData::Int(index_to)) => {
                                match self.state.deref(array as usize) {
                                    Heap::Function(_) => todo!(),
                                    Heap::Int(v) => self.state.stack.push(VmData::Int(v)),
                                    Heap::Float(_) => todo!(),
                                    Heap::Bool(_) => todo!(),
                                    Heap::ListRef(_) => todo!(),
                                    Heap::StringRef(_) => todo!(),
                                    Heap::List(array) => {
                                        let item = self.state.deref(array[index_to as usize]);
                                        match item {
                                            Heap::Function(v) => {
                                                self.state.stack.push(VmData::Function(v))
                                            }
                                            Heap::Int(v) => self.state.stack.push(VmData::Int(v)),
                                            Heap::Float(v) => {
                                                self.state.stack.push(VmData::Float(v))
                                            }
                                            Heap::Bool(v) => self.state.stack.push(VmData::Bool(v)),
                                            Heap::ListRef(v) => {
                                                self.state.stack.push(VmData::List(v))
                                            }
                                            Heap::List(_) => todo!(),
                                            Heap::String(_) => todo!(),
                                            Heap::None => todo!(),
                                            Heap::StringRef(v) => {
                                                self.state.stack.push(VmData::String(v))
                                            }
                                            Heap::Closure(_, _) => todo!(),
                                            Heap::ClosureRef(_) => todo!(),
                                        }
                                    }
                                    Heap::String(_) => todo!(),
                                    Heap::None => todo!(),
                                    Heap::Closure(_, _) => todo!(),
                                    Heap::ClosureRef(_) => todo!(),
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
                    let fl = f64::from_le_bytes([
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                    ]);
                    self.state.stack.push(VmData::Float(fl));
                }

                Code::GETGLOBAL => {
                    let index = u32::from_le_bytes([
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                    ]);
                    self.state
                        .stack
                        .push(self.state.stack[index as usize].clone());
                }

                Code::CALL => {
                    if let Some(callee) = self.state.stack.pop() {
                        match callee {
                            VmData::Closure(index) => {
                                if let Some(Heap::Closure(target, captured)) =
                                    self.state.heap.get(index)
                                {
                                    if let Heap::List(list) = &self.state.heap[*captured] {
                                        for i in list {
                                            self.state.stack.push(self.state.to_vmdata(*i))
                                        }
                                        self.state.callstack.push(self.state.current_instruction);
                                        self.state.goto(*target);
                                    } else {
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
                    let size = usize::from_le_bytes([
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                    ]);
                    for _ in 0..size {
                        string.push(self.state.next());
                    }
                    let string = match String::from_utf8(string) {
                        Ok(ok) => ok,
                        Err(_) => todo!(),
                    };
                    let index = self.state.allocate_string(string);
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
                    let index = usize::from_le_bytes([
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                        self.state.next(),
                    ]);

                    match self.native_functions[index](&mut self.state) {
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
