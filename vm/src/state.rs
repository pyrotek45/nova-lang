use crate::memory_manager::{MemoryManager, VmData};

#[derive(Debug, Clone)]
pub enum CallType {
    Function(usize),
    Closure { target: usize, closure: usize },
}

#[derive(Debug, Clone)]
pub struct State {
    pub memory: MemoryManager,
    pub program: Vec<u8>,
    pub callstack: Vec<CallType>,
    pub current_instruction: usize,
    pub offset: usize,
    pub window: Vec<usize>,
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
    pub fn deallocate_registers(&mut self) {
        if let Some(window) = self.window.pop() {
            let remove = self.memory.stack.len() - window;
            for _ in 0..remove {
                self.memory.pop();
            }
        }
        self.offset = *self.window.last().unwrap();
    }

    pub fn deallocate_registers_with_return(&mut self) {
        let returnvalue = *self.memory.stack.last().unwrap();

        // Python would INCREF the return value before cleaning locals
        self.memory.inc_value(returnvalue);
        // dbg!(returnvalue);
        if let Some(window) = self.window.pop() {
            let remove = self.memory.stack.len() - window;
            for _ in 0..remove {
                self.memory.pop();
            }
        }

        self.offset = *self.window.last().unwrap();

        // Push back the return value; now it's owned by the caller frame
        self.memory.stack.push(returnvalue);

        // Python would DECREF the return value in the callee, but we already did that by cleaning locals
        // Don't DECREF returnvalue here — it's now owned by the caller
    }
}
