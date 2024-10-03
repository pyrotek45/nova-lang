use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Heap {
    // pointer and instance
    ClosureAddress(usize),
    Closure(usize, usize),
    // function target
    Function(usize),

    // basic types
    Int(i64),
    Float(f64),
    Bool(bool),
    Char(char),

    // pointer and instance
    ListAddress(usize),
    List(Vec<usize>),

    // pointer and instance
    StringAddress(usize),
    String(String),

    // pointer and instance
    StructAddress(usize),
    Struct(String, Vec<usize>),

    None,
}

impl Heap {
    pub fn get_string(&self) -> &str {
        match self {
            Heap::String(s) => s,
            _ => {
                panic!()
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum VmData {
    // pointer to stack
    StackAddress(usize),

    // jump target
    Function(usize),
    Closure(usize),

    // basic types
    Int(i64),
    Float(f64),
    Bool(bool),
    Char(char),

    // pointer to heap
    List(usize),
    Struct(usize),
    String(usize),

    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub program: Vec<u8>,
    pub heap: Vec<Heap>,
    pub free_space: Vec<usize>,
    pub callstack: Vec<usize>,
    pub stack: Vec<VmData>,
    pub current_instruction: usize,
    pub offset: usize,
    pub window: Vec<usize>,
    pub used_data: common::table::Table<usize>,
    pub threshold: usize,
    pub gc_count: usize,
    pub garbage_collected: usize,
    pub gclock: bool,
}

pub fn new() -> State {
    State {
        program: vec![],
        current_instruction: 0,
        stack: vec![],
        callstack: vec![],
        offset: 0,
        window: vec![],
        heap: vec![],
        free_space: vec![],
        used_data: common::table::new(),
        threshold: 999999999,
        gc_count: 0,
        garbage_collected: 0,
        gclock: false,
    }
}

impl State {
    #[inline(always)]
    pub fn to_vmdata(&self, index: usize) -> VmData {
        match self.heap[index] {
            Heap::ClosureAddress(v) => VmData::Closure(v),
            Heap::Function(v) => VmData::Function(v),
            Heap::Int(v) => VmData::Int(v),
            Heap::Float(v) => VmData::Float(v),
            Heap::Bool(v) => VmData::Bool(v),
            Heap::ListAddress(v) => VmData::List(v),
            Heap::StringAddress(v) => VmData::String(v),
            Heap::None => VmData::None,
            Heap::Closure(_, _) => todo!(),
            Heap::List(_) => todo!(),
            Heap::String(_) => todo!(),
            Heap::Struct(_, _) => todo!(),
            Heap::StructAddress(v) => VmData::Struct(v),
            Heap::Char(v) => VmData::Char(v),
        }
    }

    #[inline(always)]
    pub fn program(&mut self, program: Vec<u8>) {
        self.program = program
    }

    #[inline(always)]
    pub fn next(&mut self) -> u8 {
        let result = &self.program[self.current_instruction];
        self.current_instruction += 1;
        *result
    }

    #[inline(always)]
    pub fn goto(&mut self, addr: usize) {
        self.current_instruction = addr;
    }

    #[inline(always)]
    pub fn check_useage(&mut self, index: usize) {
        if !self.used_data.has(&index) {
            self.used_data.insert(index);
            let href = self.heap[index].clone();
            match href {
                Heap::List(list) => {
                    for i in list.iter() {
                        self.check_useage(*i)
                    }
                }
                Heap::ListAddress(index) => self.check_useage(index),
                Heap::StringAddress(index) => self.check_useage(index),
                Heap::ClosureAddress(index) => self.check_useage(index),
                Heap::Closure(_, indextwo) => self.check_useage(indextwo),
                _ => {}
            }
        }
    }

    #[inline(always)]
    pub fn collect_garbage(&mut self) {
        if self.gclock {
            return;
        }
        // only run when out of free space and over threshold
        if self.threshold <= self.heap.len() {
            self.threshold = ((self.heap.len() as f64) * 1.1) as usize;
            //dbg!(&self.threshold);
        } else {
            return;
        }

        self.gc_count += 1;
        self.used_data.clear();
        for item in self.stack.clone().iter() {
            match item {
                VmData::List(index) => {
                    self.check_useage(*index);
                }
                VmData::String(index) => {
                    self.check_useage(*index);
                }
                VmData::Closure(index) => {
                    self.check_useage(*index);
                }
                _ => {}
            }
        }

        for i in 0..self.heap.len() {
            if !self.used_data.has(&i) {
                self.free_heap(i);
                self.garbage_collected += 1;
                if i == self.heap.len() {
                    self.heap.pop();
                }
            }
        }
        //dbg!(&self.garbage_collected);
    }

    #[inline(always)]
    pub fn free_heap(&mut self, index: usize) {
        self.free_space.push(index)
    }

    #[inline(always)]
    pub fn deref(&self, index: usize) -> Heap {
        self.heap[index].clone()
    }

    #[inline(always)]
    pub fn allocate_vmdata_to_heap(&mut self, item: VmData) -> usize {
        match item {
            VmData::Function(v) => {
                if let Some(space) = self.free_space.pop() {
                    self.heap[space] = Heap::Function(v);
                    return space;
                } else {
                    self.heap.push(Heap::Function(v));
                    return self.heap.len() - 1;
                }
            }
            VmData::Int(v) => {
                if let Some(space) = self.free_space.pop() {
                    self.heap[space] = Heap::Int(v);
                    return space;
                } else {
                    self.heap.push(Heap::Int(v));
                    return self.heap.len() - 1;
                }
            }
            VmData::Float(v) => {
                if let Some(space) = self.free_space.pop() {
                    self.heap[space] = Heap::Float(v);
                    return space;
                } else {
                    self.heap.push(Heap::Float(v));
                    return self.heap.len() - 1;
                }
            }
            VmData::Bool(v) => {
                if let Some(space) = self.free_space.pop() {
                    self.heap[space] = Heap::Bool(v);
                    return space;
                } else {
                    self.heap.push(Heap::Bool(v));
                    return self.heap.len() - 1;
                }
            }
            VmData::List(v) => {
                if let Some(space) = self.free_space.pop() {
                    self.heap[space] = Heap::ListAddress(v);
                    return space;
                } else {
                    self.heap.push(Heap::ListAddress(v));
                    return self.heap.len() - 1;
                }
            }
            VmData::None => {
                if let Some(space) = self.free_space.pop() {
                    self.heap[space] = Heap::None;
                    return space;
                } else {
                    self.heap.push(Heap::None);
                    return self.heap.len() - 1;
                }
            }
            VmData::String(v) => {
                if let Some(space) = self.free_space.pop() {
                    self.heap[space] = Heap::StringAddress(v);
                    return space;
                } else {
                    self.heap.push(Heap::StringAddress(v));
                    return self.heap.len() - 1;
                }
            }
            VmData::Closure(v) => {
                if let Some(space) = self.free_space.pop() {
                    self.heap[space] = Heap::ClosureAddress(v);
                    return space;
                } else {
                    self.heap.push(Heap::ClosureAddress(v));
                    return self.heap.len() - 1;
                }
            }
            VmData::StackAddress(_) => todo!(),
            VmData::Struct(v) => {
                if let Some(space) = self.free_space.pop() {
                    self.heap[space] = Heap::StructAddress(v);
                    return space;
                } else {
                    self.heap.push(Heap::StructAddress(v));
                    return self.heap.len() - 1;
                }
            }
            VmData::Char(v) => {
                if let Some(space) = self.free_space.pop() {
                    self.heap[space] = Heap::Char(v);
                    return space;
                } else {
                    self.heap.push(Heap::Char(v));
                    return self.heap.len() - 1;
                }
            }
        }
    }

    #[inline(always)]
    pub fn copy_heap(&mut self, copy: usize, target: usize) {
        self.heap[target] = self.heap[copy].clone();
    }

    #[inline(always)]
    pub fn delete_heap(&mut self, index: usize) {
        self.heap[index] = Heap::None
    }

    #[inline(always)]
    pub fn allocate_new_heap(&mut self) -> usize {
        if let Some(space) = self.free_space.pop() {
            space
        } else {
            self.collect_garbage();
            if let Some(space) = self.free_space.pop() {
                self.heap[space] = Heap::None;
                space
            } else {
                self.heap.push(Heap::None);
                self.heap.len() - 1
            }
        }
    }

    #[inline(always)]
    pub fn allocate_array(&mut self, array: Vec<usize>) -> usize {
        if let Some(space) = self.free_space.pop() {
            self.heap[space] = Heap::List(array);
            space
        } else {
            self.heap.push(Heap::List(array));
            self.heap.len() - 1
        }
    }

    #[inline(always)]
    pub fn allocate_string(&mut self, str: String) -> usize {
        if let Some(space) = self.free_space.pop() {
            self.heap[space] = Heap::String(str);
            space
        } else {
            self.collect_garbage();
            if let Some(space) = self.free_space.pop() {
                self.heap[space] = Heap::String(str);
                space
            } else {
                self.heap.push(Heap::String(str));
                self.heap.len() - 1
            }
        }
    }

    #[inline(always)]
    pub fn offset_locals(&mut self, size: usize, locals: usize) {
        self.offset = self.stack.len() - size;
        self.window.push(self.offset);
        for _ in 0..locals {
            self.stack.push(VmData::None)
        }
    }

    #[inline(always)]
    pub fn alloc_locals(&mut self, size: usize) {
        self.offset = self.stack.len();
        self.window.push(self.offset);
        for _ in 0..size {
            self.stack.push(VmData::None)
        }
    }

    #[inline(always)]
    pub fn deallocate_registers(&mut self) {
        if let Some(window) = self.window.pop() {
            let remove = self.stack.len() - window;
            for _ in 0..remove {
                self.stack.pop();
            }
        }
        self.offset = *self.window.last().unwrap();
    }

    #[inline(always)]
    pub fn deallocate_registers_with_return(&mut self) {
        let returnvalue = self.stack.last().unwrap().clone();
        if let Some(window) = self.window.pop() {
            let remove = self.stack.len() - window;
            for _ in 0..remove {
                self.stack.pop();
            }
        }
        self.offset = *self.window.last().unwrap();
        self.stack.push(returnvalue);
    }
}
