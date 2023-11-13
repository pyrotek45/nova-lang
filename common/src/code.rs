pub struct Code {}

impl Code {
    pub const RET: u8 = 0;

    pub const INTEGER: u8 = 1;
    pub const FLOAT: u8 = 2;

    pub const IADD: u8 = 3;
    pub const ISUB: u8 = 4;
    pub const IMUL: u8 = 5;
    pub const IDIV: u8 = 6;

    pub const STORE: u8 = 7;
    pub const GET: u8 = 8;
    pub const STOREFAST: u8 = 9;

    pub const ASSIGN: u8 = 10;
    pub const ALLOCLOCALS: u8 = 11;

    pub const CALL: u8 = 12;
    pub const BLOCK: u8 = 13;
    pub const DIRECTCALL: u8 = 14;

    pub const NEWLIST: u8 = 15;

    pub const TRUE: u8 = 16;
    pub const FALSE: u8 = 17;

    pub const FUNCTION: u8 = 18;

    pub const IGTR: u8 = 20;

    pub const ILSS: u8 = 21;

    pub const JUMPIFFALSE: u8 = 22;

    pub const REC: u8 = 23;

    pub const IF: u8 = 24;
    pub const WHEN: u8 = 25;

    pub const EQUALS: u8 = 26;
    pub const IMODULO: u8 = 27;

    pub const REFID: u8 = 28;

    pub const CLOSURE: u8 = 29;
    pub const CID: u8 = 30;

    pub const STRING: u8 = 31;

    pub const FOR: u8 = 32;
    pub const BOUNCE: u8 = 33;

    pub const RANGE: u8 = 34;
    pub const FORINT: u8 = 35;

    pub const BYTE: u8 = 36;

    pub const NATIVE: u8 = 37;

    pub const STOREGLOBAL: u8 = 38;
    pub const GETGLOBAL: u8 = 39;
    pub const ALLOCATEGLOBAL: u8 = 40;

    pub const CHAR: u8 = 41;

    pub const POP: u8 = 42;

    pub const NEG: u8 = 43;

    pub const BREAK: u8 = 44;

    pub const NEWBINDING: u8 = 45;
    pub const POPBINDING: u8 = 46;

    pub const STOREBIND: u8 = 47;
    pub const GETBIND: u8 = 48;

    pub const LOOP: u8 = 49;

    pub const DATA: u8 = 50;

    pub const NOT: u8 = 51;

    pub const NONE: u8 = 52;
    pub const PRINT: u8 = 53;

    pub const JMP: u8 = 54;

    pub const LINDEX: u8 = 55;
    pub const PINDEX: u8 = 56;

    pub const BJMP: u8 = 57;

    pub const STACKREF: u8 = 58;

    pub const SCONST: u8 = 59;
    pub const LCONST: u8 = 60;

    pub const FREE: u8 = 61;
    pub const CLONE: u8 = 62;

    pub const FADD: u8 = 63;
    pub const FSUB: u8 = 64;
    pub const FMUL: u8 = 65;
    pub const FDIV: u8 = 66;

    pub const FGTR: u8 = 67;
    pub const FLSS: u8 = 68;

    pub const OFFSET: u8 = 69;
    pub const TAILCALL: u8 = 70;

    pub const AND: u8 = 71;
    pub const OR: u8 = 72;
    pub const DUP: u8 = 73;

    pub const ISSOME: u8 = 74;
    pub const UNWRAP: u8 = 75;

    pub const CONCAT: u8 = 76;
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
    UNWRAP,
    FREE,
    CLONE,

    // control flow
    LABEL(usize),
    JUMPIFFALSE(usize),
    JMP(usize),
    BJMP(usize),

    // large data
    STRING(String),
    LIST(usize),

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
    FUNCTION(usize),
    CLOSURE(usize),
    RET(bool),
    DCALL(u32),
    TCALL(u32),
    CALL,

    // list operations
    PIN,
    LIN,
    NATIVE(usize),
}
