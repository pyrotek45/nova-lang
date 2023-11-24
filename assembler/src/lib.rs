use std::collections::HashMap;

use common::{
    code::{Asm, Code},
    tokens::{self, Position, TokenList},
};

#[derive(Debug)]
pub struct Assembler {
    pub runtime_error_table: HashMap<usize, Position>,
    pub input: Vec<Asm>,
    pub nva: Vec<Asm>,
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
        nva: vec![],
        runtime_error_table: HashMap::default(),
    }
}

pub fn new_empty() -> Assembler {
    Assembler {
        input: vec![],
        output: vec![],
        labels: HashMap::default(),
        forwardjumps: vec![],
        nva: vec![],
        runtime_error_table: HashMap::default(),
    }
}

impl Assembler {
    pub fn assemble_from_nva(&mut self, fileinput: TokenList) {
        let asmfile = fileinput.clone();
        let mut ci = 0;
        while ci < asmfile.len() {
            match &asmfile[ci] {
                tokens::Token::Type(_, _) => todo!(),
                tokens::Token::Identifier(command, _) => {
                    match command.as_str() {
                        "main" => {
                            ci += 2;
                        }
                        "function" => {
                            ci += 2;
                            self.nva.push(Asm::FUNCTION(
                                asmfile[ci].clone().get_int().unwrap() as usize
                            ));
                            ci += 1;
                        }
                        "closure" => {
                            ci += 2;
                            self.nva.push(Asm::CLOSURE(
                                asmfile[ci].clone().get_int().unwrap() as usize
                            ));
                            ci += 1;
                        }
                        "getg" => {
                            ci += 2;
                            self.nva.push(Asm::GETGLOBAL(
                                asmfile[ci].clone().get_int().unwrap() as u32
                            ));
                            ci += 1;
                        }
                        "getl" => {
                            ci += 2;
                            self.nva
                                .push(Asm::GET(asmfile[ci].clone().get_int().unwrap() as u32));
                            ci += 1;
                        }
                        "offset" => {
                            ci += 2;
                            self.nva.push(Asm::OFFSET(
                                asmfile[ci].clone().get_int().unwrap() as u32,
                                asmfile[ci + 1].clone().get_int().unwrap() as u32,
                            ));
                            ci += 2;
                        }
                        "lbl" => {
                            ci += 2;
                            self.nva
                                .push(Asm::LABEL(asmfile[ci].clone().get_int().unwrap() as usize));
                            ci += 1;
                        }
                        "dcall" => {
                            ci += 2;
                            self.nva
                                .push(Asm::DCALL(asmfile[ci].clone().get_int().unwrap() as u32));
                            ci += 2;
                        }
                        "tcall" => {
                            ci += 2;
                            self.nva
                                .push(Asm::TCALL(asmfile[ci].clone().get_int().unwrap() as u32));
                            ci += 1;
                        }
                        "global" => {
                            ci += 2;
                            self.nva.push(Asm::ALLOCGLOBBALS(
                                asmfile[ci].clone().get_int().unwrap() as u32,
                            ));
                            ci += 1;
                        }
                        "local" => {
                            ci += 2;
                            self.nva.push(Asm::ALLOCLOCALS(
                                asmfile[ci].clone().get_int().unwrap() as u32
                            ));
                            ci += 1;
                        }
                        "list" => {
                            ci += 2;
                            self.nva
                                .push(Asm::LIST(asmfile[ci].clone().get_int().unwrap() as usize));
                            ci += 1;
                        }
                        "storeg" => {
                            ci += 2;
                            self.nva.push(Asm::STOREGLOBAL(
                                asmfile[ci].clone().get_int().unwrap() as u32
                            ));
                            ci += 1;
                        }
                        "storel" => {
                            ci += 2;
                            self.nva
                                .push(Asm::STORE(asmfile[ci].clone().get_int().unwrap() as u32));
                            ci += 1;
                        }
                        "pushi" => {
                            ci += 2;
                            self.nva
                                .push(Asm::INTEGER(asmfile[ci].clone().get_int().unwrap()));
                            ci += 1;
                        }
                        "pushs" => {
                            ci += 2;
                            self.nva
                                .push(Asm::STRING(asmfile[ci].clone().get_str().unwrap()));
                            ci += 1;
                        }
                        "pushf" => {
                            ci += 2;
                            self.nva
                                .push(Asm::FLOAT(asmfile[ci].clone().get_float().unwrap()));
                            ci += 1;
                        }
                        "ref" => {
                            ci += 2;
                            self.nva
                                .push(Asm::STACKREF(asmfile[ci].clone().get_int().unwrap() as u32));
                            ci += 1;
                        }
                        "jmp" => {
                            ci += 2;
                            self.nva
                                .push(Asm::JMP(asmfile[ci].clone().get_int().unwrap() as usize));
                            ci += 1;
                        }
                        "jif" => {
                            ci += 2;
                            self.nva.push(Asm::JUMPIFFALSE(
                                asmfile[ci].clone().get_int().unwrap() as usize
                            ));
                            ci += 1;
                        }
                        "bjmp" => {
                            ci += 2;
                            self.nva
                                .push(Asm::BJMP(asmfile[ci].clone().get_int().unwrap() as usize));
                            ci += 1;
                        }
                        "pushb" => {
                            ci += 2;
                            if asmfile[ci].clone().get_int().unwrap() == 0 {
                                self.nva.push(Asm::BOOL(false))
                            } else if asmfile[ci].clone().get_int().unwrap() == 1 {
                                self.nva.push(Asm::BOOL(true))
                            } else {
                                panic!()
                            }
                            ci += 1;
                        }
                        "print" => {
                            ci += 1;
                            self.nva.push(Asm::PRINT);
                        }
                        "call" => {
                            ci += 1;
                            self.nva.push(Asm::CALL);
                        }
                        "not" => {
                            ci += 1;
                            self.nva.push(Asm::NOT);
                        }
                        "neg" => {
                            ci += 1;
                            self.nva.push(Asm::NEG);
                        }
                        "assign" => {
                            ci += 1;
                            self.nva.push(Asm::ASSIGN);
                        }
                        // ints
                        "iadd" => {
                            ci += 1;
                            self.nva.push(Asm::IADD);
                        }
                        "imul" => {
                            ci += 1;
                            self.nva.push(Asm::IMUL);
                        }
                        "isub" => {
                            ci += 1;
                            self.nva.push(Asm::ISUB);
                        }
                        "idiv" => {
                            ci += 1;
                            self.nva.push(Asm::IDIV);
                        }
                        "imod" => {
                            ci += 1;
                            self.nva.push(Asm::IMODULO);
                        }
                        // floats
                        "fadd" => {
                            ci += 1;
                            self.nva.push(Asm::FADD);
                        }
                        "fmul" => {
                            ci += 1;
                            self.nva.push(Asm::FMUL);
                        }
                        "fsub" => {
                            ci += 1;
                            self.nva.push(Asm::FSUB);
                        }
                        "fdiv" => {
                            ci += 1;
                            self.nva.push(Asm::FDIV);
                        }
                        "equ" => {
                            ci += 1;
                            self.nva.push(Asm::EQUALS);
                        }
                        "ilss" => {
                            ci += 1;
                            self.nva.push(Asm::ILSS);
                        }
                        "igtr" => {
                            ci += 1;
                            self.nva.push(Asm::IGTR);
                        }
                        "flss" => {
                            ci += 1;
                            self.nva.push(Asm::FLSS);
                        }
                        "fgtr" => {
                            ci += 1;
                            self.nva.push(Asm::FGTR);
                        }
                        "pin" => {
                            ci += 1;
                            self.nva.push(Asm::PIN(Position {
                                line: 0,
                                row: 0,
                                filepath: "asm".to_string(),
                            }));
                        }
                        "lin" => {
                            ci += 1;
                            self.nva.push(Asm::LIN);
                        }
                        "and" => {
                            ci += 1;
                            self.nva.push(Asm::AND);
                        }
                        "or" => {
                            ci += 1;
                            self.nva.push(Asm::OR);
                        }
                        "dup" => {
                            ci += 1;
                            self.nva.push(Asm::DUP);
                        }
                        "pop" => {
                            ci += 1;
                            self.nva.push(Asm::POP);
                        }
                        "none" => {
                            ci += 1;
                            self.nva.push(Asm::NONE);
                        }
                        "issome" => {
                            ci += 1;
                            self.nva.push(Asm::ISSOME);
                        }
                        "unwrap" => {
                            ci += 1;
                            self.nva.push(Asm::UNWRAP);
                        }
                        "concat" => {
                            ci += 1;
                            self.nva.push(Asm::CONCAT);
                        }
                        "char" | "pushc" => todo!(),
                        "ret" => {
                            ci += 2;
                            self.nva
                                .push(Asm::RET(asmfile[ci].clone().get_bool().unwrap()));
                            ci += 1;
                        }
                        a => {
                            dbg!(a);
                            todo!()
                        }
                    }
                }
                tokens::Token::Integer(_, _) => todo!(),
                tokens::Token::Float(_, _) => todo!(),
                tokens::Token::String(_, _) => todo!(),
                tokens::Token::Char(_, _) => todo!(),
                tokens::Token::Symbol(_, _) => todo!(),
                tokens::Token::Bool(_, _) => todo!(),
                tokens::Token::Operator(_, _) => todo!(),
                tokens::Token::NewLine(_) => ci += 1,
                tokens::Token::EOF(_) => ci += 1,
            }
        }
    }

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
                    let allocations = (varaibles).to_le_bytes();
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
                Asm::FUNCTION(target) => {
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
                Asm::STORE(index) => {
                    self.output.push(Code::STORE);
                    let bytes = (index as u32).to_le_bytes();
                    self.output.extend_from_slice(&bytes);
                }
                Asm::STOREGLOBAL(index) => {
                    self.output.push(Code::STOREGLOBAL);
                    self.output.extend_from_slice(&(index as u32).to_le_bytes());
                }
                Asm::GET(index) => {
                    self.output.push(Code::GET);
                    let bytes = (index as u32).to_le_bytes();
                    self.output.extend_from_slice(&bytes);
                }
                Asm::GETGLOBAL(index) => {
                    self.output.push(Code::GETGLOBAL);
                    let bytes = (index as u32).to_le_bytes();
                    self.output.extend_from_slice(&bytes);
                }
                Asm::DCALL(index) => {
                    self.output.push(Code::DIRECTCALL);
                    let bytes = (index as u32).to_le_bytes();
                    self.output.extend_from_slice(&bytes);
                }
                Asm::CALL => {
                    self.output.push(Code::CALL);
                }
                Asm::STACKREF(index) => {
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
                Asm::PIN(pos) => {
                    self.output.push(Code::PINDEX);
                    self.runtime_error_table.insert(self.output.len(), pos);
                }
                Asm::LIN => self.output.push(Code::LINDEX),
                Asm::TCALL(index) => {
                    self.output.push(Code::TAILCALL);
                    let bytes = (index as u32).to_le_bytes();
                    self.output.extend_from_slice(&bytes);
                }
                Asm::AND => self.output.push(Code::AND),
                Asm::OR => self.output.push(Code::OR),
                Asm::NATIVE(v) => {
                    self.output.push(Code::NATIVE);
                    let bytes = v.to_le_bytes();
                    self.output.extend_from_slice(&bytes);
                }
                Asm::DUP => self.output.push(Code::DUP),
                Asm::POP => self.output.push(Code::POP),
                Asm::NONE => self.output.push(Code::NONE),
                Asm::ISSOME => self.output.push(Code::ISSOME),
                Asm::UNWRAP => self.output.push(Code::UNWRAP),
                Asm::CONCAT => self.output.push(Code::CONCAT),
                Asm::Char(v) => {
                    self.output.push(Code::CHAR);
                    let cast = v as u8;
                    self.output.push(cast);
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
                println!("Assembly ERROR: No label {target} exist, exiting");
                std::process::exit(1);
            }
        }
    }
}
