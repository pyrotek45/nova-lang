use std::{ops::Range, path::Path, rc::Rc};
#[cfg(test)]
mod tests;

mod keywords {
    use common::tokens::{
        KeyWord::*,
        TokenValue::{self, *},
    };
    pub const KEYWORDS: phf::Map<&'static str, TokenValue> = phf::phf_map! {
        "in" => Keyword(In),
        "true" => Bool(true),
        "false" => Bool(false),
    };
}
pub use keywords::KEYWORDS;

use common::{
    error::NovaError,
    fileposition::FilePosition,
    tokens::{Operator, StructuralSymbol, Token, TokenValue},
};

#[derive(Debug, Clone, Default)]
pub struct Lexer {
    pos: FilePosition,
    pub source: Rc<str>,
    remaining: Range<usize>,
}

impl Lexer {
    pub fn read_file(path: impl AsRef<Path>) -> Result<Self, NovaError> {
        match std::fs::read_to_string(path.as_ref()) {
            Ok(source) => Ok(Self::new(source, Some(path.as_ref()))),
            // a very detailed error message
            Err(_) => Err(NovaError::File {
                msg: format!(" '{}' is not a valid filepath", path.as_ref().display()).into(),
            }),
        }
    }
    pub fn new(source: impl Into<Rc<str>>, path: Option<&Path>) -> Self {
        let source = source.into();
        let remaining = 0..source.len();
        Lexer {
            source,
            remaining,
            pos: FilePosition {
                filepath: path.map(|p| p.into()),
                line: 1,
                col: 1,
            },
        }
    }
    fn remaining(&self) -> &str {
        self.source
            .get(self.remaining.clone())
            .expect("remaining range out of bounds for source")
    }
    fn advance(&mut self) -> Option<char> {
        let first = self.remaining().chars().next()?;
        self.remaining.start += first.len_utf8();
        match first {
            '\n' => {
                self.pos.line += 1;
                self.pos.col = 1;
            }
            _ => self.pos.col += 1,
        }
        Some(first)
    }
    fn peek(&self) -> Option<char> {
        self.remaining().chars().next()
    }
    fn advance_if(&mut self, predicate: impl FnOnce(char) -> bool) -> Option<char> {
        let advance = predicate(self.peek()?);
        if advance {
            return self.advance();
        }
        None
    }
    fn consume_if(&mut self, predicate: impl FnOnce(char) -> bool) -> bool {
        self.advance_if(predicate).is_some()
    }
    fn match_literal(&mut self, literal: &str) -> bool {
        let matches = self.peek_literal(literal);
        if matches {
            self.remaining.start += literal.len();
        }
        matches
    }
    fn peek_literal(&mut self, literal: &str) -> bool {
        self.remaining().starts_with(literal)
    }
    pub fn span(&self) -> Span {
        Span {
            pos: self.pos.clone(),
            start: self.remaining.start,
        }
    }
    pub fn consumed_from_range(&self, span: &Span) -> Range<usize> {
        span.start..self.remaining.start
    }
    pub fn consumed_from(&self, span: &Span) -> &str {
        self.source
            .get(self.consumed_from_range(span))
            .expect("span start or remaining range start out of bounds")
    }
    fn current_position(&self) -> FilePosition {
        self.pos.clone()
    }
    fn escape(c: char) -> Option<char> {
        Some(match c {
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            '0' => '\0',
            '\'' => '\'',
            '\"' => '\"',
            '\\' => '\\',
            _ => return None,
        })
    }
    pub fn tokenize(&mut self) -> Result<Vec<Token>, NovaError> {
        self.collect()
    }
}
#[derive(Debug, Clone)]
pub struct Span {
    pos: FilePosition,
    start: usize,
}

impl Iterator for Lexer {
    type Item = Result<Token, NovaError>;

    fn next(&mut self) -> Option<Self::Item> {
        use crate::Operator::*;
        use crate::StructuralSymbol::*;
        use TokenValue::*;
        fn allocate_without_excess(text: &str) -> Rc<str> {
            let mut out = std::string::String::new();
            out.reserve_exact(text.len());
            out += text;
            out.into()
        }
        fn capture_int_digits(scanner: &mut Lexer) {
            while scanner.consume_if(|c| matches!(c, '0'..='9' | 'a'..='f' )) {}
        }
        fn try_parse_int(
            scanner: &Lexer,
            body: &str,
            radix: u32,
            kind: &str,
        ) -> Result<TokenValue, NovaError> {
            match i64::from_str_radix(body, radix) {
                Ok(n) => Ok(Integer(n)),
                Err(err) => Err(NovaError::Lexing {
                    msg: format!("Invalid integer literal {body}").into(),
                    note: format!("Error while attempting to parse {kind} integer: {err}",).into(),
                    position: scanner.current_position(),
                }),
            }
        }
        let (span, value) = loop {
            let span = self.span();

            let value = match self.advance()? {
                '\n' | '\r' | ' ' | '\t' => continue,

                '/' if self.match_literal("/") => {
                    while self.consume_if(|c| c != '\n') {}
                    continue;
                }
                '/' if self.match_literal("*") => {
                    match self.remaining().find("*/") {
                        Some(star) => {
                            self.remaining.start += star + 2;
                        }
                        None => {
                            self.remaining.start = self.remaining.end;
                            return Some(Err(NovaError::Lexing {
                                msg: "Unterminated Block comment(/* ... */)".into(),
                                note: format!(
                                    "no terminating */ in {:?}",
                                    self.consumed_from(&span),
                                )
                                .into(),
                                position: self.current_position(),
                            }));
                        }
                    }
                    continue;
                }
                '\'' => {
                    let unterminated_err = |s: &Lexer| {
                        Some(Err(NovaError::Lexing {
                            msg: "Unterminated char literal".into(),
                            note: "".into(),
                            position: s.current_position(),
                        }))
                    };
                    let Some(c) = self.advance() else {
                        return unterminated_err(self);
                    };
                    let c = match c {
                        '\\' => {
                            let Some(c) = self.advance() else {
                                return unterminated_err(self);
                            };
                            let Some(escaped) = Self::escape(c) else {
                                return Some(Err(NovaError::Lexing {
                                    msg: "Invalid escape sequence in char literal.".into(),
                                    note: format!("Attempted to use escape sequence \\{c}").into(),
                                    position: self.current_position(),
                                }));
                            };
                            escaped
                        }
                        c => c,
                    };
                    if !self.consume_if(|c| c == '\'') {
                        return unterminated_err(self);
                    };
                    TokenValue::Char(c)
                }
                'r' if self.peek_literal("#") => {
                    // Raw ("r#*[[:LITERAL:]]") syntax
                    let pound_count =
                        std::iter::from_fn(|| self.consume_if(|c| c == '#').then_some(())).count();
                    // TODO: raw literals that aren't strings
                    let Some('"') = self.advance() else {
                        return Some(Err(NovaError::Lexing {
                            msg: "Invalid raw literal".into(),
                            note: format!(
                                "no literal following raw specifier {}",
                                self.consumed_from(&span),
                            )
                            .into(),
                            position: self.current_position(),
                        }));
                    };
                    let body = self.span();
                    loop {
                        let Some(c) = self.advance() else {
                            return Some(Err(NovaError::Lexing {
                                msg: "Unterminated raw string literal".into(),
                                note: format!(
                                    "no terminating \" in {:?}",
                                    self.consumed_from(&span),
                                )
                                .into(),
                                position: self.current_position(),
                            }));
                        };
                        if c == '"'
                            && self
                                .remaining()
                                .bytes()
                                .take_while(|&b| b == b'#')
                                .take(pound_count)
                                .count()
                                >= pound_count
                        {
                            let text = self.consumed_from(&body);
                            // Remove trailing "
                            let text = &text[..text.len() - 1];
                            let text = allocate_without_excess(text);

                            // Consume #
                            for _ in 0..pound_count {
                                _ = self.advance();
                            }
                            break TokenValue::StringLiteral(text);
                        }
                    }
                }
                '"' => {
                    let mut body = String::new();
                    loop {
                        let Some(c) = self.advance() else {
                            return Some(Err(NovaError::Lexing {
                                msg: "Unterminated string literal".into(),
                                note: format!(
                                    "no terminating \" in {:?}",
                                    self.consumed_from(&span),
                                )
                                .into(),
                                position: self.current_position(),
                            }));
                        };
                        match c {
                            '"' => break,
                            '\\' => {
                                let Some(c) = self.advance() else {
                                    // Ignore \ without following symbol as that means the string
                                    // is unterminated
                                    continue;
                                };
                                // TODO: Hex ASCII/BYTE escapes using \x41 syntax?
                                let Some(escaped) = Self::escape(c) else {
                                    return Some(Err(NovaError::Lexing {
                                        msg: "Invalid escape sequence in string literal.".into(),
                                        note: format!("Attempted to use escape sequence \\{c}")
                                            .into(),
                                        position: self.current_position(),
                                    }));
                                };
                                body.push(escaped);
                            }
                            c => body.push(c),
                        }
                    }

                    return Some(Ok(Token {
                        value: TokenValue::StringLiteral(body.into()),
                        position: span.pos,
                    }));
                }

                '0' if self.match_literal("b") => {
                    let body = self.span();
                    capture_int_digits(self);
                    match try_parse_int(self, self.consumed_from(&body), 2, "binary") {
                        Ok(n) => n,
                        Err(err) => return Some(Err(err)),
                    }
                }
                '0' if self.match_literal("o") => {
                    let body = self.span();
                    capture_int_digits(self);
                    match try_parse_int(self, self.consumed_from(&body), 8, "octal") {
                        Ok(n) => n,
                        Err(err) => return Some(Err(err)),
                    }
                }
                '0' if self.match_literal("x") => {
                    let body = self.span();
                    capture_int_digits(self);
                    match try_parse_int(self, self.consumed_from(&body), 16, "hex") {
                        Ok(n) => n,
                        Err(err) => return Some(Err(err)),
                    }
                }
                c @ ('0'..='9' | '.')
                    if c != '.' || self.peek().is_some_and(|c| c.is_ascii_digit()) =>
                {
                    capture_int_digits(self);
                    let int_part = self.consumed_from(&span);

                    let float = self.remaining().starts_with('.')
                        && self.remaining()[1..]
                            .chars()
                            .next()
                            .is_some_and(|c| !c.is_alphabetic() && c != '.');

                    let float = float || c == '.';
                    if float {
                        // Capture .
                        self.advance_if(|c| c == '.');
                        // Capture rest of the digits
                        capture_int_digits(self);
                        let float = self.consumed_from(&span);
                        match float.parse() {
                            Ok(f) => Float(f),
                            Err(err) => {
                                return Some(Err(NovaError::Lexing {
                                    msg: format!("Invalid float literal {float}").into(),
                                    note: format!("Error while attempting to parse float: {err}",)
                                        .into(),
                                    position: self.current_position(),
                                }))
                            }
                        }
                    } else {
                        match try_parse_int(self, int_part, 10, "decimal") {
                            Ok(n) => n,
                            Err(err) => {
                                return Some(Err(err));
                            }
                        }
                    }
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    while self.consume_if(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '_' | '0'..='9'))
                    {
                    }
                    let ident = self.consumed_from(&span);
                    KEYWORDS
                        .get(ident)
                        .cloned()
                        .unwrap_or_else(|| Identifier(allocate_without_excess(ident)))
                }
                '+' if self.match_literal("=") => Operator(AddAssign),
                '-' if self.match_literal("=") => Operator(SubAssign),
                '&' if self.match_literal("&") => Operator(And),
                '|' if self.match_literal(">") => Operator(PipeArrow),
                '|' if self.match_literal("|") => Operator(Or),
                ':' if self.match_literal(":") => Operator(DoubleColon),
                ':' => Operator(Colon),
                '>' if self.match_literal("=") => Operator(GreaterOrEqual),
                '<' if self.match_literal("=") => Operator(LessOrEqual),
                '=' if self.match_literal("=") => Operator(Equal),
                '=' if self.match_literal(">") => Operator(FatArrow),
                '=' => Operator(Assignment),
                '-' if self.match_literal(">") => Operator(RightArrow),
                '<' if self.match_literal("-") => Operator(LeftArrow),
                '>' => Operator(Greater),
                '<' => Operator(Less),
                '+' => Operator(Addition),
                '-' => Operator(Subtraction),
                '/' => Operator(Division),
                '*' => Operator(Multiplication),
                '%' => Operator(Modulo),
                '!' if self.match_literal("=") => Operator(NotEqual),
                '!' => Operator(Not),
                '~' if self.match_literal(">") => Operator(RightTilde),
                '~' if self.match_literal(">") => Operator(RightTilde),
                '.' if self.match_literal(".=") => Operator(InclusiveRange),
                '.' if self.match_literal(".") => Operator(ExclusiveRange),

                ';' => StructuralSymbol(Semicolon),
                '(' => StructuralSymbol(LeftParen),
                ')' => StructuralSymbol(RightParen),
                '[' => StructuralSymbol(LeftSquareBracket),
                ']' => StructuralSymbol(RightSquareBracket),
                '{' => StructuralSymbol(LeftBrace),
                '}' => StructuralSymbol(RightBrace),
                ',' => StructuralSymbol(Comma),
                '$' => StructuralSymbol(DollarSign),
                '@' => StructuralSymbol(At),
                '?' => StructuralSymbol(QuestionMark),
                '#' => StructuralSymbol(Pound),

                '.' => StructuralSymbol(Dot),
                '|' => StructuralSymbol(Pipe),
                '&' => StructuralSymbol(Ampersand),
                '~' => StructuralSymbol(Tilde),
                c => {
                    return Some(Err(NovaError::Lexing {
                        msg: format!("Unexpected character {c:?}").into(),
                        note: "".into(),
                        position: self.current_position(),
                    }));
                }
            };
            break (span, value);
        };
        Some(Ok(Token {
            value,
            position: span.pos,
        }))
    }
}
