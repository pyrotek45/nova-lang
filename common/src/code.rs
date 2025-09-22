use std::rc::Rc;

use crate::fileposition::FilePosition;

pub struct Code {}

impl Code {
    // Function Control Flow
    pub const RET: u8 = 0;
    pub const OFFSET: u8 = 1;
    pub const CALL: u8 = 2;
    pub const DIRECTCALL: u8 = 3;
    pub const NATIVE: u8 = 4;

    // Function and Closure Construction 
    pub const FUNCTION: u8 = 5;
    pub const CLOSURE: u8 = 6;

    // Integer and its Operations
    pub const INTEGER: u8 = 7;
    pub const IADD: u8 = 8;
    pub const ISUB: u8 = 9;
    pub const IMUL: u8 = 10;
    pub const IDIV: u8 = 11;
    pub const IGTR: u8 = 12;
    pub const ILSS: u8 = 13;
    pub const IMODULO: u8 = 23;

    // Float and its Operations
    pub const FLOAT: u8 = 14;
    pub const FADD: u8 = 15;
    pub const FSUB: u8 = 16;
    pub const FMUL: u8 = 17;
    pub const FDIV: u8 = 18;
    pub const FGTR: u8 = 19;
    pub const FLSS: u8 = 20;

    // Number Operations
    pub const NEG: u8 = 21;
    pub const EQUALS: u8 = 22;
    
    // True/False and Logic Operations
    pub const TRUE: u8 = 24;
    pub const FALSE: u8 = 25;
    pub const NOT: u8 = 26;
    pub const AND: u8 = 27;
    pub const OR: u8 = 28;

    // Exit/Error
    pub const EXIT: u8 = 29;
    pub const ERROR: u8 = 30;

    // Control Flow
    pub const JUMPIFFALSE: u8 = 31;
    pub const BJMP: u8 = 32;
    pub const JMP: u8 = 33;

    // Vm Memory Setup
    pub const ALLOCATEGLOBAL: u8 = 34;
    pub const ALLOCLOCALS: u8 = 35;

    // Local Memory 
    pub const STORE: u8 = 36;
    pub const GET: u8 = 37;
    pub const ASSIGN: u8 = 38;

    // Global Memory
    pub const STOREGLOBAL: u8 = 39;
    pub const GETGLOBAL: u8 = 40;

    // List Construction and Operations
    pub const NEWLIST: u8 = 41;   
    pub const LINDEX: u8 = 42;
    pub const PINDEX: u8 = 43;
    
    // String Construction and Operations 
    pub const STRING: u8 = 44;
    pub const CONCAT: u8 = 45;

    // Optional Operations
    pub const NONE: u8 = 46;
    pub const ISSOME: u8 = 47;
    pub const UNWRAP: u8 = 48;
    
    // Char
    pub const CHAR: u8 = 49;

    // Stack Operations
    pub const DUP: u8 = 50;
    pub const POP: u8 = 51;

    // Memory Management
    pub const FREE: u8 = 52;
    pub const CLONE: u8 = 53;

    // Basic IO
    pub const PRINT: u8 = 54;

    // Unused
    pub const REC: u8 = 55;
    pub const IF: u8 = 56;
    pub const WHEN: u8 = 57;
    pub const REFID: u8 = 58;
    pub const CID: u8 = 59;
    pub const FOR: u8 = 60;
    pub const BOUNCE: u8 = 61;
    pub const RANGE: u8 = 62;
    pub const FORINT: u8 = 63;
    pub const BYTE: u8 = 64;
    pub const BREAK: u8 = 65;
    pub const NEWBINDING: u8 = 66;
    pub const POPBINDING: u8 = 67;
    pub const STOREBIND: u8 = 68;
    pub const GETBIND: u8 = 69;
    pub const LOOP: u8 = 70;
    pub const DATA: u8 = 71;
    pub const STACKREF: u8 = 72;
    pub const SCONST: u8 = 73;
    pub const LCONST: u8 = 74;
    pub const TAILCALL: u8 = 75;
    pub const STOREFAST: u8 = 76;
    pub const BLOCK: u8 = 77;
}

pub fn byte_to_string(byte: u8) -> String {
    match byte {
        Code::RET => "RET",
        Code::INTEGER => "INTEGER",
        Code::FLOAT => "FLOAT",
        Code::IADD => "IADD",
        Code::ISUB => "ISUB",
        Code::IMUL => "IMUL",
        Code::IDIV => "IDIV",
        Code::STORE => "STORE",
        Code::GET => "GET",
        Code::STOREFAST => "STOREFAST",
        Code::ASSIGN => "ASSIGN",
        Code::ALLOCLOCALS => "ALLOCLOCALS",
        Code::CALL => "CALL",
        Code::BLOCK => "BLOCK",
        Code::DIRECTCALL => "DIRECTCALL",
        Code::NEWLIST => "NEWLIST",
        Code::TRUE => "TRUE",
        Code::FALSE => "FALSE",
        Code::FUNCTION => "FUNCTION",
        Code::IGTR => "IGTR",
        Code::ILSS => "ILSS",
        Code::JUMPIFFALSE => "JUMPIFFALSE",
        Code::REC => "REC",
        Code::IF => "IF",
        Code::WHEN => "WHEN",
        Code::EQUALS => "EQUALS",
        Code::IMODULO => "IMODULO",
        Code::REFID => "REFID",
        Code::CLOSURE => "CLOSURE",
        Code::CID => "CID",
        Code::STRING => "STRING",
        Code::FOR => "FOR",
        Code::BOUNCE => "BOUNCE",
        Code::RANGE => "RANGE",
        Code::FORINT => "FORINT",
        Code::BYTE => "BYTE",
        Code::NATIVE => "NATIVE",
        Code::STOREGLOBAL => "STOREGLOBAL",
        Code::GETGLOBAL => "GETGLOBAL",
        Code::ALLOCATEGLOBAL => "ALLOCATEGLOBAL",
        Code::CHAR => "CHAR",
        Code::POP => "POP",
        Code::NEG => "NEG",
        Code::BREAK => "BREAK",
        Code::NEWBINDING => "NEWBINDING",
        Code::POPBINDING => "POPBINDING",
        Code::STOREBIND => "STOREBIND",
        Code::GETBIND => "GETBIND",
        Code::LOOP => "LOOP",
        Code::DATA => "DATA",
        Code::NOT => "NOT",
        Code::NONE => "NONE",
        Code::PRINT => "PRINT",
        Code::JMP => "JMP",
        Code::LINDEX => "LINDEX",
        Code::PINDEX => "PINDEX",
        Code::BJMP => "BJMP",
        Code::STACKREF => "STACKREF",
        Code::SCONST => "SCONST",
        Code::LCONST => "LCONST",
        Code::FREE => "FREE",
        Code::CLONE => "CLONE",
        Code::FADD => "FADD",
        Code::FSUB => "FSUB",
        Code::FMUL => "FMUL",
        Code::FDIV => "FDIV",
        Code::FGTR => "FGTR",
        Code::FLSS => "FLSS",
        Code::OFFSET => "OFFSET",
        Code::TAILCALL => "TAILCALL",
        Code::AND => "AND",
        Code::OR => "OR",
        Code::DUP => "DUP",
        Code::ISSOME => "ISSOME",
        Code::UNWRAP => "UNWRAP",
        _ => "Unknown", // Handle the case where the byte is not in the enum.
    }
    .to_string()
}

#[derive(Debug, Clone)]
pub enum Asm {
    // memory managment
    ALLOCGLOBBALS(u32),
    ALLOCLOCALS(u32),
    OFFSET(u32, u32),

    STORE(u32),
    STOREGLOBAL(u32),

    GET(u32),
    GETGLOBAL(u32),

    STACKREF(u32),
    ASSIGN,

    NONE,
    ISSOME,
    UNWRAP(FilePosition),
    FREE,
    CLONE,

    // control flow
    LABEL(u64),
    JUMPIFFALSE(u64),
    JMP(u64),
    BJMP(u64),

    // large data
    STRING(Rc<str>),
    LIST(u64),

    // values
    INTEGER(i64),
    FLOAT(f64),
    BOOL(bool),
    Char(char),

    // operations
    IADD,
    ISUB,
    IDIV,
    IMUL,
    IMODULO,

    ILSS,
    IGTR,

    FADD,
    FSUB,
    FDIV,
    FMUL,

    FLSS,
    FGTR,

    EQUALS,
    NOT,
    NEG,

    AND,
    OR,

    CONCAT,

    DUP,
    POP,
    // built ins
    PRINT,

    // functions
    FUNCTION(u64),
    CLOSURE(u64),
    RET(bool),
    DCALL(u32),
    TCALL(u32),
    CALL,

    // list operations
    PIN(FilePosition),
    LIN(FilePosition),
    NATIVE(u64),

    EXIT,
    ERROR(FilePosition),
}
