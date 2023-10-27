use common::code::{Asm, Code};

pub fn new() -> Disassembler {
    Disassembler {
        depth: vec![],
        native_functions: common::table::new(),
        ip: 0,
    }
}

pub struct Disassembler {
    depth: Vec<usize>,
    pub native_functions: common::table::Table<String>,
    ip: usize,
}

impl Disassembler {
    pub fn dis_asm(&mut self, asm: Vec<Asm>) {
        println!("Main:");

        for instruction in asm {
            match instruction {
                Asm::ALLOCGLOBBALS(v) => println!("    Globals: {v}"),
                Asm::ALLOCLOCALS(v) => println!("    Locals: {v}"),
                Asm::OFFSET(v, l) => println!("    Offset: {v} ~ {l}"),
                Asm::STORE(v, id) => println!("    Sto L {id}: @{v}"),
                Asm::STOREGLOBAL(v, id) => println!("    Sto G {id}: @{v}"),
                Asm::GET(v, id) => println!("    Get L {id}: @{v}"),
                Asm::GETGLOBAL(v, id) => println!("    Get G {id}: @{v}"),
                Asm::STACKREF(v, id) => println!("    Ref @{v}: {id}"),
                Asm::FUNCTION(v, id) => println!("Func {id}: -> L{v}:"),
                Asm::CLOSURE(v) => println!("Closure: -> L{v}:"),
                Asm::RET(v) => println!("    Ret: {v}"),
                Asm::LABEL(v) => println!("L{v}:"),
                Asm::JUMPIFFALSE(v) => println!("Jif: -> L{v}:"),
                Asm::JMP(v) => println!("Jmp: -> L{v}:"),
                Asm::BJMP(v) => println!("Bjp: -> L{v}:"),
                Asm::INTEGER(v) => println!("    Int: {v}"),
                Asm::BOOL(v) => println!("    Bool: {v}"),
                Asm::STRING(v) => println!("    Str: {v}"),
                Asm::LIST(v) => println!("    List: {v}"),
                Asm::FLOAT(v) => println!("    Flt: {v}"),
                Asm::IADD => println!("    IAdd"),
                Asm::ISUB => println!("    ISub"),
                Asm::IDIV => println!("    IDiv"),
                Asm::IMUL => println!("    IMul"),
                Asm::FADD => println!("    FAdd"),
                Asm::FSUB => println!("    FSub"),
                Asm::FDIV => println!("    FDiv"),
                Asm::FMUL => println!("    FMul"),
                Asm::PRINT => println!("    Print"),
                Asm::ASSIGN => println!("    Assign"),
                Asm::DIRECTCALL(v, id) => println!("    Dcall {id}: @{v}"),
                Asm::CALL => println!("    Call"),
                Asm::ILSS => println!("    ILess"),
                Asm::IGTR => println!("    Igtr"),
                Asm::FLSS => println!("    Fless"),
                Asm::FGTR => println!("    Fgtr"),
                Asm::EQUALS => println!("    Equ"),
                Asm::FREE => println!("    Free"),
                Asm::CLONE => println!("    Clone"),
                Asm::IMODULO => println!("    Imod"),
                Asm::NOT => println!("    Not"),
                Asm::NEG => println!("    Neg"),
                Asm::PINDEX => println!("    Pin"),
                Asm::LINDEX => println!("    Lin"),
                Asm::TAILCALL(v, id) => println!("    Tcall {id}: @{v}"),
            }
        }
        println!();
    }

    fn out(&self, output: &str) {
        for _ in 0..self.depth.len() {
            print!("  ")
        }
        println!("{}", output)
    }

    fn next(&mut self, input: &mut std::vec::IntoIter<u8>) -> Option<u8> {
        if let Some(index) = self.depth.last() {
            if self.ip == *index {
                self.depth.pop();
            }
        }
        //println!("ip: {}", self.ip);
        self.ip += 1;
        input.next()
    }

    pub fn dis(
        &mut self,
        mut input: std::vec::IntoIter<u8>,
    ) -> Result<(), common::error::NovaError> {
        while let Some(code) = self.next(&mut input) {
            print!("{} : ", self.ip);
            match code {
                Code::NONE => {
                    self.out(&format!("None"));
                }
                Code::LINDEX => self.out(&format!("LINDEX")),
                Code::PINDEX => self.out(&format!("PINDEX")),
                Code::JMP => {
                    let int = u32::from_le_bytes([
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                    ]);
                    self.out(&format!("JMP {}", int))
                }
                Code::BJMP => {
                    let int = u32::from_le_bytes([
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                    ]);
                    self.out(&format!("BJMP {}", int))
                }
                Code::RET => {
                    let wr = self.next(&mut input).unwrap();
                    self.out(&format!("Return with {}", wr));
                }
                Code::INTEGER => {
                    let int = usize::from_le_bytes([
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                    ]);
                    self.out(&format!("Push I{}", int))
                }
                Code::STACKREF => {
                    let int = usize::from_le_bytes([
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                    ]);
                    self.out(&format!("Stack ref {}", int))
                }
                Code::BYTE => {
                    let int = self.next(&mut input).unwrap() as i64;
                    self.out(&format!("Push I{}", int))
                }
                Code::FLOAT => {
                    let fl = f64::from_le_bytes([
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                    ]);
                    self.out(&format!("Push Float {}", fl))
                }
                Code::IADD => self.out("iAdd"),
                Code::ISUB => self.out("iSub"),
                Code::IMUL => self.out("iMul"),
                Code::IDIV => self.out("iDiv"),
                Code::FADD => self.out("fAdd"),
                Code::FSUB => self.out("fSub"),
                Code::FMUL => self.out("fMul"),
                Code::FDIV => self.out("fDiv"),
                Code::STORE => {
                    let index = u32::from_le_bytes([
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                    ]);
                    self.out(&format!("Store ID {}", index))
                }
                Code::GET => {
                    let index = u32::from_le_bytes([
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                    ]);
                    self.out(&format!("ID {}", index))
                }
                Code::ASSIGN => self.out("Assign"),
                Code::ALLOCLOCALS => {
                    let allocations = u32::from_le_bytes([
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                    ]);
                    self.out(&format!("Register allocation {}", allocations))
                }
                Code::OFFSET => {
                    let allocations = i32::from_le_bytes([
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                    ]);
                    let locals = i32::from_le_bytes([
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                    ]);
                    self.out(&format!("Offset {}, locals {}", allocations, locals))
                }
                Code::BLOCK => {
                    let jump = usize::from_le_bytes([
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                    ]);
                    self.depth.push(self.ip + jump);
                    self.out("Block:")
                }
                Code::CALL => self.out("Call"),
                Code::DIRECTCALL => {
                    let target = u32::from_le_bytes([
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                    ]);
                    self.out(&format!("Direct call {}", target))
                }
                Code::NEWLIST => {
                    let size = u64::from_le_bytes([
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                    ]);
                    self.out(&format!("Create list: size of {}", size))
                }
                Code::TRUE => self.out("Push True"),
                Code::FALSE => self.out("Push False"),
                Code::STOREFAST => {
                    let index = u32::from_le_bytes([
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                    ]);
                    self.out(&format!("StoreFast ID {}", index))
                }
                Code::FUNCTION => {
                    let jump = usize::from_le_bytes([
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                    ]);
                    self.depth.push(self.ip + jump);
                    self.out("Function:")
                }
                Code::IGTR => self.out("Greater than"),
                Code::ILSS => self.out("Less than"),
                Code::JUMPIFFALSE => {
                    let jump = u32::from_le_bytes([
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                    ]);
                    self.out(&format!("Jump if false: {}", jump))
                }
                Code::REC => self.out("Recursive call"),
                Code::WHEN => self.out("When"),
                Code::IF => self.out("If"),
                Code::EQUALS => self.out("Equals"),
                Code::IMODULO => self.out("Modulo"),
                Code::REFID => {
                    let index = u16::from_le_bytes([
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                    ]);
                    self.out(&format!("Referance ID {}", index))
                }

                Code::CLOSURE => {
                    let jump = usize::from_le_bytes([
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                    ]);
                    self.depth.push(self.ip + jump);
                    self.out("Closure:")
                }

                Code::CID => {
                    let index = usize::from_le_bytes([
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                    ]);
                    self.out(&format!("Closure ID {}", index))
                }
                Code::PRINT => self.out(&format!("Print")),

                Code::STRING => {
                    let mut string = vec![];
                    let size = usize::from_le_bytes([
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                    ]);
                    for _ in 0..size {
                        string.push(self.next(&mut input).unwrap());
                    }
                    let string = match String::from_utf8(string) {
                        Ok(ok) => ok,
                        Err(_) => panic!(),
                    };
                    self.out(&format!("Push String: {}", string))
                }

                Code::FOR => self.out("For"),

                Code::RANGE => self.out("Range"),
                Code::NATIVE => {
                    let index = usize::from_le_bytes([
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                    ]);

                    if let Some(function) = self.native_functions.retreive(index) {
                        self.out(&format!("Function: {}", function))
                    }
                }
                Code::ALLOCATEGLOBAL => {
                    let size = u32::from_le_bytes([
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                    ]);
                    self.out(&format!("Global allocation {}", size))
                }

                Code::GETGLOBAL => {
                    let index = u32::from_le_bytes([
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                    ]);

                    self.out(&format!("Global ID {}", index))
                }

                Code::STOREGLOBAL => {
                    let index = u32::from_le_bytes([
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                    ]);

                    self.out(&format!("Store Global ID {}", index))
                }

                Code::CHAR => {
                    let c = self.next(&mut input).unwrap();

                    self.out(&format!("Push Char {}", c as char))
                }

                Code::POP => self.out("Pop"),
                Code::NEG => self.out("Neg"),
                Code::BREAK => self.out("Break"),
                Code::NEWBINDING => self.out("Create Bindings"),
                Code::POPBINDING => self.out("Remove Bindings"),
                Code::GETBIND => {
                    let index = usize::from_le_bytes([
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                        self.next(&mut input).unwrap(),
                    ]);

                    self.out(&format!("Get Binding {}", index))
                }
                Code::STOREBIND => self.out("Store New Binding"),
                Code::LOOP => self.out("Loop"),
                _ => {}
            }
        }

        Ok(())
    }
}
