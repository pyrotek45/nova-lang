use std::collections::HashMap;

use common::code::{Asm, Code};

pub struct Assembler {
    pub input: Vec<Asm>,
    pub output: Vec<u8>,
    labels: HashMap<usize, usize>,
    forwardjumps: Vec<(usize, usize)>,
}

pub fn new(input: Vec<Asm>) -> Assembler {
    Assembler {
        input,
        output: vec![],
        labels: HashMap::default(),
        forwardjumps: vec![],
    }
}

impl Assembler {
    pub fn assemble(&mut self) {
        for instruction in self.input.iter().cloned() {
            match instruction {
                Asm::LABEL(label) => {
                    self.labels.insert(label, self.output.len());
                }
                Asm::RET(has_return) => {
                    if has_return {
                        self.output.push(Code::RET);
                        self.output.push(1);
                    } else {
                        self.output.push(Code::RET);
                        self.output.push(0);
                    }
                }
                Asm::INTEGER(int) => {
                    self.output.push(Code::INTEGER);
                    let int = (int as i64).to_le_bytes();
                    self.output.extend_from_slice(&int);
                }

                Asm::PRINT => self.output.push(Code::PRINT),
                Asm::ALLOCGLOBBALS(globals) => {
                    self.output.push(Code::ALLOCATEGLOBAL);
                    let allocations = (globals).to_le_bytes();
                    self.output.extend_from_slice(&allocations);
                }
                Asm::ALLOCLOCALS(locals) => {
                    self.output.push(Code::ALLOCLOCALS);
                    let allocations = (locals).to_le_bytes();
                    self.output.extend_from_slice(&allocations);
                }
                Asm::OFFSET(locals, varaibles) => {
                    self.output.push(Code::OFFSET);
                    let allocations = (locals).to_le_bytes();
                    self.output.extend_from_slice(&allocations);
                    let allocations = (varaibles - locals).to_le_bytes();
                    self.output.extend_from_slice(&allocations);
                }
                Asm::BOOL(bool) => {
                    if bool {
                        self.output.push(Code::TRUE)
                    } else {
                        self.output.push(Code::FALSE)
                    }
                }
                Asm::JUMPIFFALSE(target) => {
                    if let Some(destination) = self.labels.get(&target) {
                        self.output.push(Code::JUMPIFFALSE);
                        let t = (*destination as u32).to_le_bytes();
                        self.output.extend_from_slice(&t);
                    } else {
                        self.output.push(Code::JUMPIFFALSE);
                        self.forwardjumps.push((target, self.output.len()));
                        let t = (0 as u32).to_le_bytes();
                        self.output.extend_from_slice(&t);
                    }
                }
                Asm::JMP(target) => {
                    if let Some(destination) = self.labels.get(&target) {
                        self.output.push(Code::JMP);
                        let t = (*destination as u32).to_le_bytes();
                        self.output.extend_from_slice(&t);
                    } else {
                        self.output.push(Code::JMP);
                        self.forwardjumps.push((target, self.output.len()));
                        let t = (0 as u32).to_le_bytes();
                        self.output.extend_from_slice(&t);
                    }
                }
                Asm::BJMP(target) => {
                    if let Some(destination) = self.labels.get(&target) {
                        self.output.push(Code::BJMP);
                        let t = ((self.output.len() - destination + 4) as u32).to_le_bytes();
                        self.output.extend_from_slice(&t);
                    } else {
                        dbg!(target);
                        panic!()
                    }
                }
                Asm::IADD => self.output.push(Code::IADD),
                Asm::ISUB => self.output.push(Code::ISUB),
                Asm::IDIV => self.output.push(Code::IDIV),
                Asm::IMUL => self.output.push(Code::IMUL),
                Asm::FADD => self.output.push(Code::FADD),
                Asm::FSUB => self.output.push(Code::FSUB),
                Asm::FDIV => self.output.push(Code::FDIV),
                Asm::FMUL => self.output.push(Code::FMUL),
                Asm::FUNCTION(target, _) => {
                    if let Some(destination) = self.labels.get(&target) {
                        self.output.push(Code::FUNCTION);
                        let t = (*destination as u32).to_le_bytes();
                        self.output.extend_from_slice(&t);
                    } else {
                        self.output.push(Code::FUNCTION);
                        self.forwardjumps.push((target, self.output.len()));
                        let t = (0 as u32).to_le_bytes();
                        self.output.extend_from_slice(&t);
                    }
                }
                Asm::ASSIGN => self.output.push(Code::ASSIGN),
                Asm::STORE(index, _) => {
                    self.output.push(Code::STORE);
                    let bytes = (index as u32).to_le_bytes();
                    self.output.extend_from_slice(&bytes);
                }
                Asm::STOREGLOBAL(index, _) => {
                    self.output.push(Code::STOREGLOBAL);
                    self.output.extend_from_slice(&(index as u32).to_le_bytes());
                }
                Asm::GET(index, _) => {
                    self.output.push(Code::GET);
                    let bytes = (index as u32).to_le_bytes();
                    self.output.extend_from_slice(&bytes);
                }
                Asm::GETGLOBAL(index, _) => {
                    self.output.push(Code::GETGLOBAL);
                    let bytes = (index as u32).to_le_bytes();
                    self.output.extend_from_slice(&bytes);
                }
                Asm::DIRECTCALL(index, _) => {
                    self.output.push(Code::DIRECTCALL);
                    let bytes = (index as u32).to_le_bytes();
                    self.output.extend_from_slice(&bytes);
                }
                Asm::CALL => {
                    self.output.push(Code::CALL);
                }
                Asm::STACKREF(index, _) => {
                    self.output.push(Code::STACKREF);
                    let i = (index).to_le_bytes();
                    self.output.extend_from_slice(&i);
                }
                Asm::ILSS => {
                    self.output.push(Code::ILSS);
                }
                Asm::IGTR => {
                    self.output.push(Code::IGTR);
                }
                Asm::FLSS => {
                    self.output.push(Code::FLSS);
                }
                Asm::FGTR => {
                    self.output.push(Code::FGTR);
                }
                Asm::EQUALS => {
                    self.output.push(Code::EQUALS);
                }
                Asm::FREE => self.output.push(Code::FREE),
                Asm::CLONE => self.output.push(Code::CLONE),
                Asm::STRING(string) => {
                    self.output.push(Code::STRING);
                    let size = string.len().to_le_bytes();
                    self.output.extend_from_slice(&size);
                    let cast = string.as_bytes();
                    self.output.extend_from_slice(cast);
                }
                Asm::LIST(size) => {
                    self.output.push(Code::NEWLIST);
                    self.output.extend_from_slice(&(size).to_le_bytes()); // Number of fields
                }
                Asm::FLOAT(v) => {
                    self.output.push(Code::FLOAT);
                    let float = v.to_le_bytes();
                    self.output.extend_from_slice(&float);
                }
                Asm::IMODULO => self.output.push(Code::IMODULO),
                Asm::NOT => self.output.push(Code::NOT),
                Asm::NEG => self.output.push(Code::NEG),
                Asm::CLOSURE(target) => {
                    if let Some(destination) = self.labels.get(&target) {
                        self.output.push(Code::CLOSURE);
                        let t = (*destination as u32).to_le_bytes();
                        self.output.extend_from_slice(&t);
                    } else {
                        self.output.push(Code::CLOSURE);
                        self.forwardjumps.push((target, self.output.len()));
                        let t = (0 as u32).to_le_bytes();
                        self.output.extend_from_slice(&t);
                    }
                }
                Asm::PINDEX => self.output.push(Code::PINDEX),
                Asm::LINDEX => self.output.push(Code::LINDEX),
                Asm::TAILCALL(index, _) => {
                    self.output.push(Code::TAILCALL);
                    let bytes = (index as u32).to_le_bytes();
                    self.output.extend_from_slice(&bytes);
                }
            }
        }

        for (target, replace) in self.forwardjumps.iter() {
            if let Some(destination) = self.labels.get(&target) {
                // need to offset - 4 for the jump location

                let t = ((destination - replace - 4) as u32).to_le_bytes();
                self.output[replace + 0] = t[0];
                self.output[replace + 1] = t[1];
                self.output[replace + 2] = t[2];
                self.output[replace + 3] = t[3];
            } else {
                dbg!(target);
                panic!()
            }
        }
    }
}
