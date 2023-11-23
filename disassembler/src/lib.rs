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
        println!("main:");
        for instruction in asm {
            match instruction {
                Asm::ALLOCGLOBBALS(v) => println!("    global: {v}"),
                Asm::ALLOCLOCALS(v) => println!("    local: {v}"),
                Asm::OFFSET(v, l) => println!("    offset: {v} {l}"),
                Asm::STORE(v) => println!("    storel: {v}"),
                Asm::STOREGLOBAL(v) => println!("    storeg: {v}"),
                Asm::GET(v) => println!("    getl: {v}"),
                Asm::GETGLOBAL(v) => println!("    getg: {v}"),
                Asm::STACKREF(v) => println!("    Ref: {v}"),
                Asm::FUNCTION(v) => println!("function: {v}"),
                Asm::CLOSURE(v) => println!("closure: {v}"),
                Asm::RET(v) => println!("    Ret: {v}"),
                Asm::LABEL(v) => println!("lbl: {v}"),
                Asm::JUMPIFFALSE(v) => println!("    jif: {v}"),
                Asm::JMP(v) => println!("    jmp: {v}"),
                Asm::BJMP(v) => println!("    bjmp: {v}"),
                Asm::INTEGER(v) => println!("    pushi: {v}"),
                Asm::BOOL(v) => println!("    pushb: {v}"),
                Asm::STRING(v) => println!("    pushs: {v}"),
                Asm::LIST(v) => println!("    list: {v}"),
                Asm::FLOAT(v) => println!("    pushf: {v}"),
                Asm::IADD => println!("    iadd"),
                Asm::ISUB => println!("    isub"),
                Asm::IDIV => println!("    idiv"),
                Asm::IMUL => println!("    imul"),
                Asm::FADD => println!("    fadd"),
                Asm::FSUB => println!("    fsub"),
                Asm::FDIV => println!("    fdiv"),
                Asm::FMUL => println!("    fmul"),
                Asm::PRINT => println!("    print"),
                Asm::ASSIGN => println!("    assign"),
                Asm::DCALL(v) => println!("    dcall: {v}"),
                Asm::CALL => println!("    call"),
                Asm::ILSS => println!("    ilss"),
                Asm::IGTR => println!("    igtr"),
                Asm::FLSS => println!("    flss"),
                Asm::FGTR => println!("    fgtr"),
                Asm::EQUALS => println!("    equ"),
                Asm::FREE => println!("    free"),
                Asm::CLONE => println!("    clone"),
                Asm::IMODULO => println!("    imod"),
                Asm::NOT => println!("    not"),
                Asm::NEG => println!("    neg"),
                Asm::PIN(_) => println!("    pin"),
                Asm::LIN => println!("    lin"),
                Asm::TCALL(v) => println!("    tcall: {v}"),
                Asm::AND => println!("    and"),
                Asm::OR => println!("    or"),
                Asm::NATIVE(v) => println!("    native: {v}"),
                Asm::DUP => println!("    dup"),
                Asm::POP => println!("    pop"),
                Asm::NONE => println!("    none"),
                Asm::ISSOME => println!("    issome"),
                Asm::UNWRAP => println!("    unwrap"),
                Asm::CONCAT => println!("    concat"),
                Asm::Char(v) => println!("    char: {v}"),
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
