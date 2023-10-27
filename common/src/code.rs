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
}

#[derive(Debug, Clone)]
pub enum Asm {
    // memory managment
    ALLOCGLOBBALS(u32),
    ALLOCLOCALS(u32),
    OFFSET(u32, u32),

    STORE(u32, String),
    STOREGLOBAL(u32, String),

    GET(u32, String),
    GETGLOBAL(u32, String),

    STACKREF(u32, String),
    ASSIGN,

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

    // built ins
    PRINT,

    // functions
    FUNCTION(usize, String),
    CLOSURE(usize),
    RET(bool),
    DIRECTCALL(u32, String),
    TAILCALL(u32, String),
    CALL,

    // list operations
    PINDEX,
    LINDEX,
}
