use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use common::error::{NovaError, NovaResult};
use raylib::prelude::*;

use crate::memory_manager::{MemoryManager, VmData};

#[derive(Debug, Clone)]
pub enum Draw {
    Text(String, i32, i32, i32, Color),
    FPS(i32, i32),
    Rectangle(i32, i32, i32, i32, Color),
    RectangleLines(i32, i32, i32, i32, Color),
    RoundedRectangle(i32, i32, i32, i32, f32, Color),
    Circle(i32, i32, f32, Color),
    CircleLines(i32, i32, f32, Color),
    Triangle(f32, f32, f32, f32, f32, f32, Color),
    Line(i32, i32, i32, i32, Color),
    LineThick(i32, i32, i32, i32, f32, Color),
    ClearBackground(Color),
    Sprite(usize, i32, i32),
}

#[derive(Debug, Clone)]
pub enum CallType {
    Function(usize),
    Closure { target: usize, closure: usize },
}

pub struct State {
    pub memory: MemoryManager,
    pub program: Vec<u8>,
    pub callstack: Vec<CallType>,
    pub current_instruction: usize,
    pub offset: usize,
    pub window: Vec<usize>,
    pub raylib: Option<Rc<RefCell<RaylibHandle>>>,
    pub raylib_thread: Option<RaylibThread>,
    pub draw_queue: Vec<Draw>,
    pub sprites: Vec<Rc<Texture2D>>,
    pub current_dir: PathBuf,
    pub audio_initialized: bool,
    pub sounds: Vec<raylib::ffi::Sound>,
    pub music: Vec<raylib::ffi::Music>,
}

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State")
            .field("program_len", &self.program.len())
            .field("current_instruction", &self.current_instruction)
            .field("offset", &self.offset)
            .field("raylib", &self.raylib.is_some())
            .finish()
    }
}

impl Clone for State {
    fn clone(&self) -> Self {
        State {
            memory: self.memory.clone(),
            program: self.program.clone(),
            callstack: self.callstack.clone(),
            current_instruction: self.current_instruction,
            offset: self.offset,
            window: self.window.clone(),
            raylib: self.raylib.clone(),
            raylib_thread: None,
            draw_queue: self.draw_queue.clone(),
            sprites: self.sprites.clone(),
            current_dir: self.current_dir.clone(),
            audio_initialized: self.audio_initialized,
            sounds: vec![],
            music: vec![],
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl State {
    pub fn new() -> State {
        State {
            program: vec![],
            current_instruction: 0,
            callstack: vec![],
            offset: 0,
            window: vec![],
            memory: MemoryManager::new(10_000),
            raylib: None,
            raylib_thread: None,
            draw_queue: vec![],
            sprites: vec![],
            current_dir: PathBuf::new(),
            audio_initialized: false,
            sounds: vec![],
            music: vec![],
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
        let arr: [u8; LEN] = self.program[self.current_instruction..][..LEN]
            .try_into()
            .expect("program bytecode is malformed: unexpected end of instructions");
        self.current_instruction += LEN;
        arr
    }

    #[inline(always)]
    pub fn goto(&mut self, addr: usize) {
        self.current_instruction = addr;
    }

    #[inline(always)]
    pub fn offset_locals(&mut self, offset: usize, locals: usize) {
        self.offset = self.memory.stack.len() - offset;
        self.window.push(self.offset);
        for _ in 0..locals {
            self.memory.stack.push(VmData::None)
        }
    }

    #[inline(always)]
    pub fn alloc_locals(&mut self, size: usize) {
        self.offset = self.memory.stack.len();
        self.window.push(self.offset);
        for _ in 0..size {
            self.memory.stack.push(VmData::None)
        }
    }

    #[inline(always)]
    pub fn deallocate_registers(&mut self) -> NovaResult<()> {
        if let Some(window) = self.window.pop() {
            let remove = self.memory.stack.len() - window;
            for _ in 0..remove {
                self.memory.pop();
            }
        }
        let offset = self.window.last().ok_or_else(|| {
            Box::new(NovaError::Runtime {
                msg: "Internal error: call stack frame underflow during deallocation".into(),
            })
        })?;
        self.offset = *offset;
        Ok(())
    }

    pub fn deallocate_registers_with_return(&mut self) -> NovaResult<()> {
        let returnvalue = *self.memory.stack.last().ok_or_else(|| {
            Box::new(NovaError::Runtime {
                msg: "Internal error: stack is empty when returning a value".into(),
            })
        })?;

        // Python would INCREF the return value before cleaning locals
        self.memory.inc_value(returnvalue);
        // dbg!(returnvalue);
        if let Some(window) = self.window.pop() {
            let remove = self.memory.stack.len() - window;
            for _ in 0..remove {
                self.memory.pop();
            }
        }

        let offset = self.window.last().ok_or_else(|| {
            Box::new(NovaError::Runtime {
                msg: "Internal error: call stack frame underflow during return".into(),
            })
        })?;
        self.offset = *offset;

        // Push back the return value; now it's owned by the caller frame
        self.memory.stack.push(returnvalue);

        // Python would DECREF the return value in the callee, but we already did that by cleaning locals
        // Don't DECREF returnvalue here — it's now owned by the caller
        Ok(())
    }
}
