use ::std::fmt;
use common::table::Table;
use raylib::prelude::*;
use std::{
    cell::RefCell,
    fmt::{Display, Formatter},
    io::{self, Write},
    rc::Rc,
};

#[derive(Debug, Clone)]
pub enum Draw {
    Text {
        text: Rc<str>,
        x: i32,
        y: i32,
        size: i32,
        color: Color,
    },
    FPS {
        x: i32,
        y: i32,
    },
    Rectangle {
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        color: Color,
    },
    Circle {
        x: i32,
        y: i32,
        radius: i32,
        color: Color,
    },
    Line {
        start_x: i32,
        start_y: i32,
        end_x: i32,
        end_y: i32,
        color: Color,
    },
    ClearBackground {
        color: Color,
    },
    Sprite {
        x: i32,
        y: i32,
        sprite_index: usize,
    },
}

#[derive(Debug, Clone, PartialEq)]
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
    String(Rc<str>),

    // pointer and instance
    StructAddress(usize),
    Struct(String, Vec<usize>),

    None,
}

#[derive(Debug, Copy, Clone, PartialEq)]
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

impl Display for Heap {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Heap::ClosureAddress(v) => write!(f, "Closure Address ({})", v),
            Heap::Function(v) => write!(f, "Function Pointer ({})", v),
            Heap::Int(v) => write!(f, "{}", v),
            Heap::Float(v) => write!(f, "{}", v),
            Heap::Bool(v) => write!(f, "{}", v),
            Heap::ListAddress(v) => write!(f, "List Address ({})", v),
            Heap::List(v) => {
                write!(f, "[")?;
                for i in 0..v.len() {
                    write!(f, "{}", v[i])?;
                    if i != v.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "]")
            }
            Heap::StringAddress(v) => write!(f, "String Address ({})", v),
            Heap::String(v) => write!(f, "{}", v),
            Heap::None => write!(f, "None"),
            Heap::Closure(_, _) => write!(f, "Closure"),
            Heap::Struct(_, _) => write!(f, "Struct"),
            Heap::StructAddress(v) => write!(f, "Struct Address ({})", v),
            Heap::Char(v) => write!(f, "{}", v),
        }
    }
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

    // get integer value
    pub fn get_int(&self) -> i64 {
        match self {
            Heap::Int(i) => *i,
            _ => {
                panic!()
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct State {
    pub program: Vec<u8>,
    pub heap: fxhash::FxHashMap<usize, Heap>,
    pub free_space: Vec<usize>,
    pub callstack: Vec<usize>,
    pub stack: Vec<VmData>,
    pub current_instruction: usize,
    pub offset: usize,
    pub window: Vec<usize>,
    pub used_data: Table<usize>,
    pub threshold: usize,
    pub gc_count: usize,
    pub garbage_collected: usize,
    pub gclock: bool,
    pub raylib: Option<Rc<RefCell<RaylibHandle>>>,
    pub raylib_thread: Option<RaylibThread>,
    pub draw_queue: Vec<Draw>,
    pub sprites: Vec<Rc<Texture2D>>,
    pub current_dir: std::path::PathBuf,
    pub latency: u128,
}

impl Drop for State {
    fn drop(&mut self) {
        if let Some(raylib) = &self.raylib {
            let raylib = raylib.borrow_mut();
            // clear all sprites in the texture
            self.sprites.clear();
            drop(raylib);
        }
    }
}

pub fn new() -> State {
    State {
        program: vec![],
        current_instruction: 0,
        current_dir: std::path::PathBuf::new(),
        stack: vec![],
        callstack: vec![],
        offset: 0,
        window: vec![],
        heap: fxhash::FxHashMap::default(),
        free_space: vec![],
        used_data: Table::new(),
        threshold: 50,
        gc_count: 0,
        garbage_collected: 0,
        gclock: false,
        raylib: None,
        raylib_thread: None,
        draw_queue: vec![],
        sprites: vec![],
        latency: 16,
    }
}

impl State {
    // recursiely print out data for the Heap type, and ouly print out the value

    pub fn print_heap(&self, index: usize) {
        let mut out = io::stdout().lock();
        // check if the index is out of bounds
        if index >= self.heap.len() {
            return;
        }
        match &self.heap[&index] {
            Heap::ClosureAddress(v) => {
                self.print_heap(*v);
            }
            Heap::Function(v) => {
                write!(out, "Function Pointer ({v})").unwrap();
            }
            Heap::Int(v) => {
                write!(out, "{v}").unwrap();
            }
            Heap::Float(v) => {
                write!(out, "{v}").unwrap();
            }
            Heap::Bool(v) => {
                write!(out, "{v}").unwrap();
            }
            Heap::ListAddress(v) => {
                self.print_heap(*v);
            }
            Heap::StringAddress(v) => {
                self.print_heap(*v);
            }
            Heap::None => {
                write!(out, "None").unwrap();
            }
            Heap::Closure(function_poiner, capture_index) => {
                print!("Closure (");
                print!("Function Pointer: {}", function_poiner);
                print!(", ");
                print!("Captures: ");
                self.print_heap(*capture_index);
                print!(")");
            }
            Heap::List(v) => {
                write!(out, "[").unwrap();
                // print out the list with commas in between without the last comma
                for i in 0..v.len() {
                    self.print_heap(v[i]);
                    if i != v.len() - 1 {
                        write!(out, ",").unwrap();
                    }
                }
                write!(out, "]").unwrap();
            }
            Heap::String(v) => {
                write!(out, "{v}").unwrap();
            }
            Heap::Struct(_, _) => {
                todo!()
            }
            Heap::StructAddress(v) => {
                self.print_heap(*v);
            }
            Heap::Char(v) => {
                write!(out, "{}", v).unwrap();
            }
        }
        out.flush().unwrap();
    }

    #[inline(always)]
    pub fn to_vmdata(&self, index: usize) -> VmData {
        match self.heap[&index] {
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
    pub fn next_instruction(&mut self) -> u8 {
        let result = &self.program[self.current_instruction];
        self.current_instruction += 1;
        *result
    }
    #[inline(always)]
    pub fn next_arr<const LEN: usize>(&mut self) -> [u8; LEN] {
        let arr = self.program[self.current_instruction..][..LEN]
            .try_into()
            .unwrap();
        self.current_instruction += LEN;
        arr
    }

    #[inline(always)]
    pub fn goto(&mut self, addr: usize) {
        self.current_instruction = addr;
    }

    #[inline(always)]
    pub fn check_usage(&mut self, index: usize) {
        if !self.used_data.has(&index) {
            self.used_data.insert(index);
            let href = &self.heap[&index];
            match href {
                Heap::List(list) => {
                    for i in list.clone() {
                        self.check_usage(i)
                    }
                }
                Heap::ListAddress(index) => self.check_usage(*index),
                Heap::StringAddress(index) => self.check_usage(*index),
                Heap::ClosureAddress(index) => self.check_usage(*index),
                Heap::Closure(_, indextwo) => self.check_usage(*indextwo),
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
        if self.threshold > self.heap.len() {
            return;
        }

        // let latency = time::Instant::now();

        self.gc_count += 1;
        self.used_data.clear();
        for item in self.stack.clone().iter() {
            match item {
                VmData::List(index) => {
                    self.check_usage(*index);
                }
                VmData::String(index) => {
                    self.check_usage(*index);
                }
                VmData::Closure(index) => {
                    self.check_usage(*index);
                }
                _ => {}
            }
        }

        self.garbage_collected = 0;
        let keys_to_remove: Vec<usize> = self
            .heap
            .keys()
            .filter(|&&i| !self.used_data.has(&i))
            .cloned()
            .collect();

        for i in keys_to_remove {
            self.garbage_collected += 1;
            self.heap.remove(&i);
        }

        let delta = (self.heap.len().saturating_sub(self.garbage_collected) / 10) + 1;
        self.threshold = self.heap.len() * (2 * delta);

        // // print debug info
        // println!(
        //     "collected {}, threshold {}, heap length {}, latency {}, delta {}",
        //     self.garbage_collected,
        //     self.threshold,
        //     self.heap.len(),
        //     latency.elapsed().as_millis(),
        //     delta
        // );

        //dbg!(&self.garbage_collected);
    }

    #[inline(always)]
    pub fn free_heap(&mut self, index: usize) {
        self.free_space.push(index)
    }

    #[inline(always)]
    pub fn get_ref(&self, index: usize) -> &Heap {
        &self.heap[&index]
    }

    fn allocate_to_heap(&mut self, item: Heap) -> usize {
        self.collect_garbage();
        let mut index = 0;
        let mut index2 = self.heap.len();
        let mut fromstart = true;
        //let time = time::Instant::now();
        loop {
            if fromstart {
                if self.heap.contains_key(&index) {
                    index += 1;
                    fromstart = false;
                } else {
                    break;
                }
            } else if self.heap.contains_key(&index2) {
                index2 -= 1;
                fromstart = true;
            } else {
                index = index2;
                break;
            }
        }
        // println!("allocated in {}ms at {}", time.elapsed().as_millis(), index);
        self.heap.insert(index, item);
        index
    }

    #[inline(always)]
    pub fn allocate_vmdata_to_heap(&mut self, item: VmData) -> usize {
        match item {
            VmData::Function(v) => self.allocate_to_heap(Heap::Function(v)),
            VmData::Int(v) => self.allocate_to_heap(Heap::Int(v)),
            VmData::Float(v) => self.allocate_to_heap(Heap::Float(v)),
            VmData::Bool(v) => self.allocate_to_heap(Heap::Bool(v)),
            VmData::List(v) => self.allocate_to_heap(Heap::ListAddress(v)),
            VmData::None => self.allocate_to_heap(Heap::None),
            VmData::String(v) => self.allocate_to_heap(Heap::StringAddress(v)),
            VmData::Closure(v) => self.allocate_to_heap(Heap::ClosureAddress(v)),
            VmData::StackAddress(_) => todo!(),
            VmData::Struct(v) => self.allocate_to_heap(Heap::StructAddress(v)),
            VmData::Char(v) => self.allocate_to_heap(Heap::Char(v)),
        }
    }

    #[inline(always)]
    pub fn copy_heap(&mut self, copy: usize, target: usize) {
        if let Some(copy_value) = self.heap.get(&copy).cloned() {
            self.heap.insert(target, copy_value);
        }
    }

    #[inline(always)]
    pub fn delete_heap(&mut self, index: usize) {
        self.heap.insert(index, Heap::None);
    }

    #[inline(always)]
    pub fn allocate_new_heap(&mut self) -> usize {
        self.allocate_to_heap(Heap::None)
    }

    #[inline(always)]
    pub fn allocate_array(&mut self, array: Vec<usize>) -> usize {
        self.allocate_to_heap(Heap::List(array))
    }

    #[inline(always)]
    pub fn allocate_string(&mut self, str: Rc<str>) -> usize {
        self.allocate_to_heap(Heap::String(str))
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
        let returnvalue = *self.stack.last().unwrap();
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
