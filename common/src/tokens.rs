use std::{
    fmt::{self, Display},
    ops::Deref,
    rc::Rc,
};

use crate::fileposition::FilePosition;
pub type TokenList = Vec<Token>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unary {
    Positive,
    Negative,
    Not,
}

impl Display for Unary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Unary::Positive => f.write_str("+"),
            Unary::Negative => f.write_str("-"),
            Unary::Not => f.write_str("!"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    /// +=
    AddAssign,
    /// -=
    SubAssign,
    /// *=
    MulAssign,
    /// /=
    DivAssign,
    /// &&
    And,
    /// ||
    Or,
    /// ::
    DoubleColon,
    /// :
    Colon,
    /// >=
    GreaterOrEqual,
    /// <=
    LessOrEqual,
    /// ==
    Equal,
    /// =
    Assignment,
    /// ->
    RightArrow,
    /// >
    Greater,
    /// <
    Less,
    /// +
    Addition,
    /// -
    Subtraction,
    /// /
    Division,
    /// *
    Multiplication,
    /// %
    Modulo,
    /// !=
    NotEqual,
    /// !
    Not,
    // FIXME: No representation in code?
    Concat,
    // FIXME: Not implemented anywhere/no text repr?
    Access,
    // FIXME: Not implemented anywhere/no text repr?
    ListAccess,
    // FIXME: Not implemented anywhere/no text repr?
    Call,
    // special operators
    /// ~>
    RightTilde,
    // FIXME: Not implemented anywhere/no text repr?
    /// <~
    LeftTilde,
    /// ..=
    InclusiveRange,
    /// ..
    ExclusiveRange,
    /// fat arrow
    FatArrow,
    /// pipe arrow
    PipeArrow,
}

impl Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Operator::AddAssign => "+=",
            Operator::SubAssign => "-=",
            Operator::MulAssign => "*=",
            Operator::DivAssign => "/=",
            Operator::And => "&&",
            Operator::Or => "||",
            Operator::DoubleColon => "::",
            Operator::Colon => ":",
            Operator::GreaterOrEqual => ">=",
            Operator::LessOrEqual => "<=",
            Operator::Equal => "==",
            Operator::Assignment => "=",
            Operator::RightArrow => "->",
            Operator::Greater => ">",
            Operator::Less => "<",
            Operator::Addition => "+",
            Operator::Subtraction => "-",
            Operator::Division => "/",
            Operator::Multiplication => "*",
            Operator::Modulo => "%",
            Operator::NotEqual => "!=",
            Operator::Not => "!",
            Operator::Concat => "++",
            Operator::Access => ".",
            Operator::ListAccess => "[]",
            Operator::Call => "()",
            Operator::RightTilde => "~>",
            Operator::LeftTilde => "<~",
            Operator::InclusiveRange => "..=",
            Operator::ExclusiveRange => "..",
            Operator::FatArrow => "=>",
            Operator::PipeArrow => "|>",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyWord {
    In,
}

impl Display for KeyWord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyWord::In => f.write_str("in"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum StructuralSymbol {
    ///;
    Semicolon,
    /// (
    LeftParen,
    /// )
    RightParen,
    /// [
    LeftSquareBracket,
    /// ]
    RightSquareBracket,
    /// ,
    Comma,
    /// {
    LeftBrace,
    /// }
    RightBrace,
    /// $
    DollarSign,
    /// @
    At,
    /// ?
    QuestionMark,
    /// #
    Pound,
    /// .
    Dot,
    /// &
    Ampersand,
    /// ~
    Tilde,
    /// |
    Pipe,
}

impl Display for StructuralSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            StructuralSymbol::Semicolon => ";",
            StructuralSymbol::LeftParen => "(",
            StructuralSymbol::RightParen => ")",
            StructuralSymbol::LeftSquareBracket => "[",
            StructuralSymbol::RightSquareBracket => "]",
            StructuralSymbol::Comma => ",",
            StructuralSymbol::LeftBrace => "{",
            StructuralSymbol::RightBrace => "}",
            StructuralSymbol::DollarSign => "$",
            StructuralSymbol::At => "@",
            StructuralSymbol::QuestionMark => "?",
            StructuralSymbol::Pound => "#",
            StructuralSymbol::Dot => ".",
            StructuralSymbol::Ampersand => "&",
            StructuralSymbol::Tilde => "~",
            StructuralSymbol::Pipe => "|",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenValue {
    Keyword(KeyWord),
    Identifier(Rc<str>),
    Integer(i64),
    Float(f64),
    StringLiteral(Rc<str>),
    Char(char),
    StructuralSymbol(StructuralSymbol),
    Bool(bool),
    Operator(Operator),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub value: TokenValue,
    pub position: FilePosition,
}

impl Token {
    pub fn position(&self) -> FilePosition {
        self.position.clone()
    }

    pub fn get_bool(&self) -> Option<bool> {
        let TokenValue::Bool(b) = self.value else {
            return None;
        };
        Some(b)
    }

    pub fn into_str(self) -> Option<Rc<str>> {
        let TokenValue::StringLiteral(s) = self.value else {
            return None;
        };
        Some(s)
    }

    pub fn get_int(&self) -> Option<i64> {
        let TokenValue::Integer(n) = self.value else {
            return None;
        };
        Some(n)
    }

    pub fn get_float(self) -> Option<f64> {
        let TokenValue::Float(n) = self.value else {
            return None;
        };
        Some(n)
    }

    pub fn into_ident(self) -> Option<Rc<str>> {
        let TokenValue::Identifier(i) = self.value else {
            return None;
        };
        Some(i)
    }

    pub fn is_identifier(&self) -> bool {
        matches!(self.value, TokenValue::Identifier(_))
    }

    pub fn is_id(&self, ident: &str) -> bool {
        matches!(&self.value, TokenValue::Identifier(id) if id.deref() == ident)
    }

    pub fn line(&self) -> usize {
        self.position.line
    }
    pub fn col(&self) -> usize {
        self.position.col
    }

    pub fn is_symbol(&self, s: StructuralSymbol) -> bool {
        matches!(&self.value, &TokenValue::StructuralSymbol(symbol) if symbol == s)
    }

    pub fn is_keyword(&self, keyword: KeyWord) -> bool {
        matches!(&self.value, TokenValue::Keyword(kw) if kw == &keyword)
    }

    pub fn is_relop(&self) -> bool {
        matches!(
            &self.value,
            TokenValue::Operator(
                Operator::Equal
                    | Operator::GreaterOrEqual
                    | Operator::LessOrEqual
                    | Operator::Greater
                    | Operator::Less
                    | Operator::NotEqual
            )
        )
    }

    pub fn is_logical_op(&self) -> bool {
        matches!(
            &self.value,
            TokenValue::Operator(Operator::And | Operator::Or)
        )
    }

    pub fn is_logical_and(&self) -> bool {
        matches!(&self.value, TokenValue::Operator(Operator::And))
    }

    pub fn is_logical_or(&self) -> bool {
        matches!(&self.value, TokenValue::Operator(Operator::Or))
    }

    pub fn is_op(&self, op: Operator) -> bool {
        matches!(&self.value, TokenValue::Operator(operator) if *operator == op)
    }

    pub const fn is_adding_op(&self) -> bool {
        matches!(
            self.value,
            TokenValue::Operator(Operator::Addition | Operator::Subtraction | Operator::Concat)
        )
    }

    pub const fn is_multi_op(&self) -> bool {
        matches!(
            self.value,
            TokenValue::Operator(Operator::Multiplication | Operator::Division | Operator::Modulo)
        )
    }

    pub const fn is_assign(&self) -> bool {
        matches!(
            self.value,
            TokenValue::Operator(Operator::Assignment | Operator::AddAssign | Operator::SubAssign | Operator::MulAssign | Operator::DivAssign)
        )
    }

    pub fn get_operator(&self) -> Option<Operator> {
        let TokenValue::Operator(op) = &self.value else {
            return None;
        };
        Some(*op)
    }
}

impl Display for TokenValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenValue::Identifier(name) => write!(f, "`{name}`"),
            TokenValue::Integer(value) => write!(f, "`{value}`"),
            TokenValue::Float(value) => write!(f, "`{value}`"),
            TokenValue::StringLiteral(value) => write!(f, "`\"{value}\"`"),
            TokenValue::Char(value) => write!(f, "`'{value}'`"),
            TokenValue::Bool(value) => write!(f, "`{value}`"),
            TokenValue::Operator(op) => write!(f, "`{op}`"),
            TokenValue::StructuralSymbol(sym) => write!(f, "`{sym}`"),
            TokenValue::Keyword(kw) => write!(f, "`{kw}`"),
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}
