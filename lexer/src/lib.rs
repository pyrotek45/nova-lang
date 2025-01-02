use std::{path::Path, rc::Rc};

use common::{
    error::NovaError,
    fileposition::{load_file_content, FilePosition},
    tokens::{KeyWord, Operator, Token, TokenList},
    ttype::TType,
};

#[derive(Debug, PartialEq, Eq, Clone)]
enum LexerState {
    Token,
    Char,
    String,
    Comment,
    Float,
    StringLiteral,
}

#[derive(Debug, Clone)]
pub struct Lexer {
    pos: FilePosition,
    pub source: Rc<str>,
    token_list: TokenList,
    buffer: String,
    state: LexerState,
    string_start_position: Vec<usize>,
    char_start_position: Vec<usize>,
    literal_pound_count: Vec<usize>,
}

impl Default for Lexer {
    fn default() -> Self {
        Self {
            pos: FilePosition::default(),
            source: Default::default(),
            token_list: Default::default(),
            buffer: Default::default(),
            state: LexerState::Token,
            string_start_position: vec![],
            char_start_position: vec![],
            literal_pound_count: vec![],
        }
    }
}

impl Lexer {
    pub fn new(path: &Path) -> Result<Lexer, NovaError> {
        let source = match load_file_content(path) {
            Ok(value) => value,
            Err(value) => return Err(value),
        };
        Ok(Lexer {
            pos: FilePosition {
                line: 1,
                row: 1,
                filepath: Some(path.into()),
            },
            source: source.into(),
            token_list: Default::default(),
            buffer: Default::default(),
            state: LexerState::Token,
            string_start_position: vec![],
            char_start_position: vec![],
            literal_pound_count: vec![],
        })
    }

    fn current_position(&self) -> FilePosition {
        self.pos.clone()
    }

    fn current_position_buffer_row(&self, row: usize) -> FilePosition {
        let mut pos = self.pos.clone();
        pos.row = row;
        pos
    }

    fn check_token_buffer(&mut self) -> Option<Token> {
        if self.buffer.is_empty() {
            return None;
        }
        if let Ok(v) = self.buffer.parse() {
            return Some(if self.buffer.contains('.') {
                self.state = LexerState::Token;
                Token::Float {
                    value: v,
                    position: self
                        .current_position_buffer_row(self.pos.row - self.buffer.chars().count()),
                }
            } else {
                Token::Integer {
                    value: v as i64,
                    position: self
                        .current_position_buffer_row(self.pos.row - self.buffer.chars().count()),
                }
            });
        }

        // Splits buffers beginning with a number, e.g., 1.print() -> 1 . print
        if self.buffer.contains('.') {
            let lastchar = self.buffer.chars().last();
            let split = self.buffer.split('.');
            for id in split {
                if let Ok(v) = id.parse::<i64>() {
                    self.state = LexerState::Token;
                    self.token_list.push(Token::Integer {
                        value: v,
                        position: self
                            .current_position_buffer_row(self.pos.row - id.chars().count()),
                    });
                } else {
                    self.token_list.push(Token::Identifier {
                        name: id.to_string(),
                        position: self
                            .current_position_buffer_row(self.pos.row - id.chars().count()),
                    });
                }
                self.token_list.push(Token::Symbol {
                    symbol: '.',
                    position: self.current_position_buffer_row(self.pos.row - id.chars().count()),
                });
            }

            self.token_list.pop();

            if let Some('.') = lastchar {
                self.token_list.push(Token::Symbol {
                    symbol: '.',
                    position: self.current_position_buffer_row(self.pos.row + 1),
                });
            }
            return None;
        }

        // Consider adding keywords like let, if, for, type, fn
        match self.buffer.as_str() {
            "false" => Some(Token::Bool {
                value: false,
                position: self
                    .current_position_buffer_row(self.pos.row - self.buffer.chars().count()),
            }),
            "true" => Some(Token::Bool {
                value: true,
                position: self
                    .current_position_buffer_row(self.pos.row - self.buffer.chars().count()),
            }),
            "Int" => Some(Token::Type {
                ttype: TType::Int,
                position: self
                    .current_position_buffer_row(self.pos.row - self.buffer.chars().count()),
            }),
            "Float" => Some(Token::Type {
                ttype: TType::Float,
                position: self
                    .current_position_buffer_row(self.pos.row - self.buffer.chars().count()),
            }),
            "Bool" => Some(Token::Type {
                ttype: TType::Bool,
                position: self
                    .current_position_buffer_row(self.pos.row - self.buffer.chars().count()),
            }),
            "String" => Some(Token::Type {
                ttype: TType::String,
                position: self
                    .current_position_buffer_row(self.pos.row - self.buffer.chars().count()),
            }),
            "Any" => Some(Token::Type {
                ttype: TType::Any,
                position: self
                    .current_position_buffer_row(self.pos.row - self.buffer.chars().count()),
            }),
            "Char" => Some(Token::Type {
                ttype: TType::Char,
                position: self
                    .current_position_buffer_row(self.pos.row - self.buffer.chars().count()),
            }),
            "in" => Some(Token::Keyword {
                keyword: KeyWord::In,
                position: self
                    .current_position_buffer_row(self.pos.row - self.buffer.chars().count()),
            }),
            _ => {
                //dbg!(a);
                Some(Token::Identifier {
                    name: self.buffer.to_string(),
                    position: self
                        .current_position_buffer_row(self.pos.row - self.buffer.chars().count()),
                })
            }
        }
    }

    fn check_token(&mut self) {
        if let Some(token) = self.check_token_buffer() {
            self.token_list.push(token);
        }
        self.buffer.clear();
    }

    pub fn tokenize(&mut self) -> Result<TokenList, NovaError> {
        let tempstr = self.source.clone();
        let mut chars = tempstr.chars().peekable();

        while let Some(c) = chars.next() {
            // check for r and if so check for " and then parse for string literal
            if self.state == LexerState::StringLiteral {
                if c != '"' {
                    if c == '\n' {
                        self.pos.line += 1;
                        self.pos.row = 1;
                    } else {
                        self.pos.row += 1;
                    }
                    self.buffer.push(c);
                    continue;
                }
                // need to be able to recover if the string literal is not closed with the same number of pound signs
                let mut current_iter = chars.clone();
                if let Some(pound_count) = self.literal_pound_count.last().cloned() {
                    let mut pound_count2 = 0;
                    while let Some('#') = current_iter.peek() {
                        current_iter.next();
                        pound_count2 += 1;
                    }
                    if pound_count == pound_count2 {
                        self.state = LexerState::Token;
                        //dbg!("String Literal End");
                        let string_start = self.string_start_position.pop().unwrap();
                        self.token_list.push(Token::String {
                            value: self.buffer.clone(),
                            position: self.current_position_buffer_row(string_start),
                        });
                        self.literal_pound_count.pop();
                        self.pos.row += 1;
                        self.buffer.clear();
                        chars = current_iter;
                        continue;
                    } else {
                        self.pos.row += 1;
                        self.buffer.push(c);
                        continue;
                    }
                }
            }

            if self.state == LexerState::Comment {
                if c != '\n' {
                    self.pos.row += 1;
                    continue;
                }
                self.state = LexerState::Token;
                self.pos.line += 1;
                self.pos.row = 1;
                continue;
            }
            if self.state == LexerState::String {
                if c == '\\' {
                    match chars.peek() {
                        Some('n') => {
                            chars.next();
                            self.buffer.push('\n');
                            self.pos.row += 2;
                            continue;
                        }
                        Some('t') => {
                            chars.next();
                            self.buffer.push('\t');
                            self.pos.row += 2;
                            continue;
                        }
                        Some('r') => {
                            chars.next();
                            self.buffer.push('\r');
                            self.pos.row += 2;
                            continue;
                        }
                        Some('\'') => {
                            chars.next();
                            self.buffer.push('\'');
                            self.pos.row += 2;
                            continue;
                        }
                        Some('\"') => {
                            chars.next();
                            self.buffer.push('\"');
                            self.pos.row += 2;
                            continue;
                        }
                        Some('0') => {
                            chars.next();
                            self.buffer.push('\0');
                            self.pos.row += 2;
                            continue;
                        }
                        Some('\\') => {
                            chars.next();
                            self.buffer.push('\\');
                            self.pos.row += 2;
                            continue;
                        }
                        _ => {
                            return Err(NovaError::Lexing {
                                msg: "Expecting valid escape char".into(),
                                note: "".into(),
                                position: self.current_position(),
                            });
                        }
                    }
                }
                if c != '"' {
                    if c == '\n' {
                        self.pos.line += 1;
                        self.pos.row = 1;
                    } else {
                        self.pos.row += 1;
                    }
                    self.buffer.push(c);
                    continue;
                } else {
                    self.state = LexerState::Token;
                    let string_start = self.string_start_position.pop().unwrap();
                    self.token_list.push(Token::String {
                        value: self.buffer.clone(),
                        position: self.current_position_buffer_row(string_start),
                    });
                    self.pos.row += 1;
                    self.buffer.clear();
                    continue;
                }
            }
            if self.state == LexerState::Char {
                if c == '\\' {
                    match chars.peek() {
                        Some('n') => {
                            chars.next();
                            self.buffer.push('\n');
                            self.pos.row += 2;
                            continue;
                        }
                        Some('t') => {
                            chars.next();
                            self.buffer.push('\t');
                            self.pos.row += 2;
                            continue;
                        }
                        Some('r') => {
                            chars.next();
                            self.buffer.push('\r');
                            self.pos.row += 2;
                            continue;
                        }
                        Some('\'') => {
                            chars.next();
                            self.buffer.push('\'');
                            self.pos.row += 2;
                            continue;
                        }
                        Some('\"') => {
                            chars.next();
                            self.buffer.push('\"');
                            self.pos.row += 2;
                            continue;
                        }
                        Some('0') => {
                            chars.next();
                            self.buffer.push('\0');
                            self.pos.row += 2;
                            continue;
                        }
                        Some('\\') => {
                            chars.next();
                            self.buffer.push('\\');
                            self.pos.row += 2;
                            continue;
                        }
                        _ => {
                            return Err(NovaError::Lexing {
                                msg: "Expecting valid escape char".into(),
                                note: "".into(),
                                position: self.current_position(),
                            })
                        }
                    }
                } else if c == '\'' {
                    self.state = LexerState::Token;
                    // should throw error, cant have ''
                    if self.buffer.is_empty() || self.buffer.chars().count() > 1 {
                        return Err(NovaError::Lexing {
                            msg: "Expecting valid char".into(),
                            note: format!("? {}", self.buffer).into(),
                            position: self.current_position(),
                        });
                    }
                    let char_start = self.char_start_position.pop().unwrap();
                    self.token_list.push(Token::Char {
                        value: self.buffer.chars().next().unwrap(),
                        position: self.current_position_buffer_row(char_start),
                    });
                    self.pos.row += 1;
                    self.buffer.clear();
                    continue;
                } else {
                    self.buffer.push(c);
                    self.pos.row += 1;
                    continue;
                }
            }

            match c {
                '\'' => {
                    self.state = LexerState::Char;
                    self.check_token();
                    self.char_start_position.push(self.pos.row);
                }
                '"' => {
                    self.state = LexerState::String;
                    self.check_token();
                    self.string_start_position.push(self.pos.row);
                }
                '\n' => {
                    self.check_token();
                    self.pos.line += 1;
                    self.pos.row = 1;
                    continue;
                }
                '\r' => self.check_token(),
                '\t' => self.check_token(),
                '.' => {
                    if let Some('.') = chars.peek() {
                        chars.next();
                        if let Some('=') = chars.peek() {
                            chars.next();
                            self.check_token();
                            self.token_list.push(Token::Operator {
                                operator: Operator::ExclusiveRange,
                                position: self.current_position(),
                            });
                            self.pos.row += 3;
                            continue;
                        } else {
                            self.check_token();
                            self.token_list.push(Token::Operator {
                                operator: Operator::InclusiveRange,
                                position: self.current_position(),
                            });
                            self.pos.row += 2;
                            continue;
                        }
                    }
                    if self.state != LexerState::Float {
                        if let Ok(_v) = self.buffer.parse::<i64>() {
                            self.state = LexerState::Float;
                            self.buffer.push(c);
                        } else {
                            self.check_token();
                            self.token_list.push(Token::Symbol {
                                symbol: c,
                                position: self.current_position(),
                            });
                        }
                    } else {
                        self.check_token();
                        self.token_list.push(Token::Symbol {
                            symbol: c,
                            position: self.current_position(),
                        });
                    }
                }
                'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => {
                    // Check for raw strings
                    if c == 'r' {
                        // Check for pound signs and count them, then check for "
                        let mut pound_count = 0;
                        while let Some('#') = chars.peek() {
                            chars.next();
                            pound_count += 1;
                        }
                        if let Some(&'"') = chars.peek() {
                            chars.next();
                            self.literal_pound_count.push(pound_count);
                            self.state = LexerState::StringLiteral;
                            self.check_token();
                            self.string_start_position.push(self.pos.row);
                            continue;
                        } else {
                            // If not a raw string, push 'r' and any pound signs to the buffer
                            self.pos.row += 1;
                            self.buffer.push(c);
                            for _ in 0..pound_count {
                                self.pos.row += 1;
                                self.buffer.push('#');
                            }
                            continue;
                        }
                    }
                    // If not 'r', push the character to the buffer
                    self.buffer.push(c);
                }
                ' ' => self.check_token(),
                '+' | '*' | '/' | '-' | '=' | '<' | '>' | '%' | '!' | ':' | '&' | '|' | '~' => {
                    self.check_token();
                    // Handle multi-character operators and other specific cases
                    match c {
                        ':' => {
                            if let Some(':') = chars.peek() {
                                chars.next();
                                self.token_list.push(Token::Operator {
                                    operator: Operator::DoubleColon,
                                    position: self.current_position(),
                                });
                                self.pos.row += 1;
                            } else {
                                self.token_list.push(Token::Operator {
                                    operator: Operator::Colon,
                                    position: self.current_position(),
                                });
                            }
                        }
                        '%' => self.token_list.push(Token::Operator {
                            operator: Operator::Modulo,
                            position: self.current_position(),
                        }),
                        '>' => {
                            if let Some('=') = chars.peek() {
                                chars.next();
                                self.token_list.push(Token::Operator {
                                    operator: Operator::GtrOrEqu,
                                    position: self.current_position(),
                                });
                                self.pos.row += 1;
                            } else {
                                self.token_list.push(Token::Operator {
                                    operator: Operator::GreaterThan,
                                    position: self.current_position(),
                                });
                            }
                        }
                        '<' => {
                            if let Some('=') = chars.peek() {
                                chars.next();
                                self.token_list.push(Token::Operator {
                                    operator: Operator::LssOrEqu,
                                    position: self.current_position(),
                                });
                                self.pos.row += 1;
                                continue;
                            }
                            if let Some('-') = chars.peek() {
                                chars.next();
                                self.token_list.push(Token::Operator {
                                    operator: Operator::LeftArrow,
                                    position: self.current_position(),
                                });
                                self.pos.row += 1;
                            } else {
                                self.token_list.push(Token::Operator {
                                    operator: Operator::LessThan,
                                    position: self.current_position(),
                                });
                            }
                        }
                        '+' => {
                            if let Some('=') = chars.peek() {
                                chars.next();
                                self.token_list.push(Token::Operator {
                                    operator: Operator::AdditionAssignment,
                                    position: self.current_position(),
                                });
                                self.pos.row += 1;
                            } else {
                                self.token_list.push(Token::Operator {
                                    operator: Operator::Addition,
                                    position: self.current_position(),
                                });
                            }
                        }
                        '-' => {
                            if let Some('>') = chars.peek() {
                                chars.next();
                                self.token_list.push(Token::Operator {
                                    operator: Operator::RightArrow,
                                    position: self.current_position(),
                                });
                                self.pos.row += 1;
                            } else if let Some('=') = chars.peek() {
                                chars.next();
                                self.token_list.push(Token::Operator {
                                    operator: Operator::SubtractionAssignment,
                                    position: self.current_position(),
                                });
                                self.pos.row += 1;
                            } else {
                                self.token_list.push(Token::Operator {
                                    operator: Operator::Subtraction,
                                    position: self.current_position(),
                                });
                            }
                        }
                        '*' => self.token_list.push(Token::Operator {
                            operator: Operator::Multiplication,
                            position: self.current_position(),
                        }),
                        '/' => {
                            if let Some('/') = chars.peek() {
                                chars.next();
                                self.pos.row += 1;
                                self.state = LexerState::Comment;
                            } else {
                                self.token_list.push(Token::Operator {
                                    operator: Operator::Division,
                                    position: self.current_position(),
                                });
                            }
                        }
                        '=' => {
                            if let Some('=') = chars.peek() {
                                chars.next();
                                self.token_list.push(Token::Operator {
                                    operator: Operator::Equality,
                                    position: self.current_position(),
                                });
                                self.pos.row += 1;
                            } else {
                                self.token_list.push(Token::Operator {
                                    operator: Operator::Assignment,
                                    position: self.current_position(),
                                });
                            }
                        }
                        '!' => {
                            if let Some('=') = chars.peek() {
                                chars.next();
                                self.token_list.push(Token::Operator {
                                    operator: Operator::NotEqual,
                                    position: self.current_position(),
                                });
                                self.pos.row += 1;
                            } else {
                                self.token_list.push(Token::Operator {
                                    operator: Operator::Not,
                                    position: self.current_position(),
                                });
                            }
                        }
                        '&' => {
                            if let Some('&') = chars.peek() {
                                chars.next();
                                self.token_list.push(Token::Operator {
                                    operator: Operator::And,
                                    position: self.current_position(),
                                });
                                self.pos.row += 1;
                            } else {
                                self.token_list.push(Token::Symbol {
                                    symbol: c,
                                    position: self.current_position(),
                                });
                            }
                        }
                        '|' => {
                            if let Some('|') = chars.peek() {
                                chars.next();
                                self.token_list.push(Token::Operator {
                                    operator: Operator::Or,
                                    position: self.current_position(),
                                });
                                self.pos.row += 1;
                            } else {
                                self.token_list.push(Token::Symbol {
                                    symbol: c,
                                    position: self.current_position(),
                                });
                            }
                        }
                        '~' => {
                            if let Some('>') = chars.peek() {
                                chars.next();
                                self.token_list.push(Token::Operator {
                                    operator: Operator::RightTilde,
                                    position: self.current_position(),
                                });
                                self.pos.row += 1;
                            } else {
                                self.token_list.push(Token::Symbol {
                                    symbol: c,
                                    position: self.current_position(),
                                });
                            }
                        }
                        _ => {}
                    }
                }
                ';' | '(' | ')' | '[' | ']' | ',' | '{' | '}' | '$' | '@' | '?' | '#' => {
                    self.check_token();
                    self.token_list.push(Token::Symbol {
                        symbol: c,
                        position: self.current_position(),
                    });
                }
                _ => {}
            }
            self.pos.row += 1;
        }

        // check state and throw error if not token
        match self.state {
            LexerState::String => {
                return Err(NovaError::Lexing {
                    msg: "Expecting valid string".into(),
                    note: format!("? {}", self.buffer).into(),
                    position: self.current_position(),
                });
            }
            LexerState::StringLiteral => {
                return Err(NovaError::Lexing {
                    msg: "Expecting valid string literal".into(),
                    note: "".into(),
                    position: self.current_position(),
                });
            }
            _ => {}
        }

        self.check_token();
        self.token_list.push(Token::EOF {
            position: self.current_position(),
        });

        Ok(self.token_list.clone())
    }

    pub fn check(&self) {
        for token in self.token_list.iter() {
            common::error::print_line(&token.position(), &token.to_string());
        }
    }
}
