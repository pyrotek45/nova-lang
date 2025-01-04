use std::{fmt::Display, ops::Deref, rc::Rc};

use crate::fileposition::FilePosition;
pub type TokenList = Vec<Token>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unary {
    Positive,
    Negative,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    // TODO: Add MulAssign, DivAssign and similar variants for the rest of the binary operators
    /// +=
    AddAssign,
    /// -=
    SubAssign,
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
    /// <-
    LeftArrow,
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
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyWord {
    In,
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
                    | Operator::And
                    | Operator::Or
                    | Operator::GreaterOrEqual
                    | Operator::LessOrEqual
                    | Operator::Greater
                    | Operator::Less
                    | Operator::NotEqual
            )
        )
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
            TokenValue::Operator(Operator::Assignment | Operator::AddAssign | Operator::SubAssign)
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use TokenValue::*;
        match self {
            Identifier(name) => write!(f, "Identifier(\"{name}\")"),
            Integer(value) => write!(f, "Integer({value})"),
            Float(value) => write!(f, "Float({value})"),
            StringLiteral(value) => write!(f, "String(\"{value}\")"),
            Char(value) => write!(f, "Char('{value}')"),
            Bool(value) => write!(f, "Bool({value})"),
            Operator(operator) => write!(f, "Operator({operator:?})"),
            StructuralSymbol(sym) => write!(f, "StructuralSymbol({sym:?})"),
            Keyword(keyword) => write!(f, "Keyword({keyword:?})"),
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}
