use std::fmt::Display;

use crate::fileposition::FilePosition;
use crate::ttype::TType;
pub type TokenList = Vec<Token>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unary {
    Positive,
    Negitive,
    Not,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operator {
    Concat,
    AdditionAssignment,
    SubtractionAssignment,
    And,
    Or,
    Colon,
    GtrOrEqu,
    LssOrEqu,
    DoubleColon,
    RightArrow,
    LeftArrow,
    GreaterThan,
    LessThan,
    Assignment,
    Addition,
    Subtraction,
    Division,
    Multiplication,
    Equality,
    NotEqual,
    Access,
    ListAccess,
    Call,
    Modulo,
    Not,
    // special operators
    RightTilde,
    LeftTilde,
    InclusiveRange,
    ExclusiveRange,
}
#[derive(Debug, Clone, PartialEq)]
pub enum KeyWord {
    In,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Keyword {
        keyword: KeyWord,
        position: FilePosition,
    },
    Type {
        ttype: TType,
        position: FilePosition,
    },
    Identifier {
        name: String,
        position: FilePosition,
    },
    Integer {
        value: i64,
        position: FilePosition,
    },
    Float {
        value: f64,
        position: FilePosition,
    },
    String {
        value: String,
        position: FilePosition,
    },
    Char {
        value: char,
        position: FilePosition,
    },
    Symbol {
        symbol: char,
        position: FilePosition,
    },
    Bool {
        value: bool,
        position: FilePosition,
    },
    Operator {
        operator: Operator,
        position: FilePosition,
    },
    EOF {
        position: FilePosition,
    },
}

impl Token {
    pub fn position(&self) -> FilePosition {
        match self {
            Token::Type { position, .. }
            | Token::Identifier { position, .. }
            | Token::Integer { position, .. }
            | Token::Float { position, .. }
            | Token::String { position, .. }
            | Token::Char { position, .. }
            | Token::Symbol { position, .. }
            | Token::Bool { position, .. }
            | Token::Operator { position, .. }
            | Token::EOF { position } => position.clone(),
            Token::Keyword { position, .. } => position.clone(),
        }
    }

    pub fn get_bool(self) -> Option<bool> {
        if let Token::Bool { value, .. } = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn get_str(self) -> Option<String> {
        if let Token::String { value, .. } = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn get_id(self) -> Option<String> {
        if let Token::Identifier { name, .. } = self {
            Some(name)
        } else {
            None
        }
    }

    pub fn get_int(self) -> Option<i64> {
        if let Token::Integer { value, .. } = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn get_float(self) -> Option<f64> {
        if let Token::Float { value, .. } = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn expect_id(self) -> Option<String> {
        if let Token::Identifier { name, .. } = self {
            Some(name)
        } else {
            None
        }
    }

    pub fn is_identifier(&self) -> bool {
        matches!(self, Token::Identifier { .. })
    }

    pub fn is_id(&self, ident: &str) -> bool {
        if let Token::Identifier { name, .. } = self {
            ident == name
        } else {
            false
        }
    }

    pub fn line(&self) -> usize {
        match self {
            Token::Type { position, .. }
            | Token::Identifier { position, .. }
            | Token::Integer { position, .. }
            | Token::Float { position, .. }
            | Token::String { position, .. }
            | Token::Char { position, .. }
            | Token::Symbol { position, .. }
            | Token::Bool { position, .. }
            | Token::Operator { position, .. }
            | Token::EOF { position } => position.line,
            Token::Keyword { position, .. } => position.line,
        }
    }

    pub fn row(&self) -> usize {
        match self {
            Token::Type { position, .. }
            | Token::Identifier { position, .. }
            | Token::Integer { position, .. }
            | Token::Float { position, .. }
            | Token::String { position, .. }
            | Token::Char { position, .. }
            | Token::Symbol { position, .. }
            | Token::Bool { position, .. }
            | Token::Operator { position, .. }
            | Token::EOF { position } => position.row,
            Token::Keyword { position, .. } => position.row,
        }
    }

    pub fn is_symbol(&self, c: char) -> bool {
        if let Token::Symbol { symbol, .. } = self {
            *symbol == c
        } else {
            false
        }
    }

    pub fn is_relop(&self) -> bool {
        if let Token::Operator { operator, .. } = self {
            matches!(
                operator,
                Operator::Equality
                    | Operator::And
                    | Operator::Or
                    | Operator::GtrOrEqu
                    | Operator::LssOrEqu
                    | Operator::GreaterThan
                    | Operator::LessThan
                    | Operator::NotEqual
            )
        } else {
            false
        }
    }

    pub fn is_op(&self, op: Operator) -> bool {
        if let Token::Operator { operator, .. } = self {
            *operator == op
        } else {
            false
        }
    }

    pub fn is_adding_op(&self) -> bool {
        if let Token::Operator { operator, .. } = self {
            matches!(
                operator,
                Operator::Addition | Operator::Subtraction | Operator::Concat
            )
        } else {
            false
        }
    }

    pub fn is_multi_op(&self) -> bool {
        if let Token::Operator { operator, .. } = self {
            matches!(
                operator,
                Operator::Multiplication | Operator::Division | Operator::Modulo
            )
        } else {
            false
        }
    }

    pub fn is_assign(&self) -> bool {
        if let Token::Operator { operator, .. } = self {
            matches!(
                operator,
                Operator::Assignment
                    | Operator::AdditionAssignment
                    | Operator::SubtractionAssignment
            )
        } else {
            false
        }
    }

    pub fn is_eof(&self) -> bool {
        matches!(self, Token::EOF { .. })
    }

    pub fn get_operator(&self) -> Option<Operator> {
        if let Token::Operator { operator, .. } = self {
            Some(operator.clone())
        } else {
            None
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Type { ttype, .. } => write!(f, "Type({:?})", ttype),
            Token::Identifier { name, .. } => write!(f, "Identifier(\"{}\")", name),
            Token::Integer { value, .. } => write!(f, "Integer({})", value),
            Token::Float { value, .. } => write!(f, "Float({})", value),
            Token::String { value, .. } => write!(f, "String(\"{}\")", value),
            Token::Char { value, .. } => write!(f, "Char('{}')", value),
            Token::Symbol { symbol, .. } => write!(f, "Symbol('{}')", symbol),
            Token::Bool { value, .. } => write!(f, "Bool({})", value),
            Token::Operator { operator, .. } => write!(f, "Operator({:?})", operator),
            Token::EOF { .. } => write!(f, "EOF"),
            Token::Keyword { keyword, .. } => write!(f, "Keyword({:?})", keyword),
        }
    }
}
