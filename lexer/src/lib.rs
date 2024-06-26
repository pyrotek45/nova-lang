use common::{
    error::{file_error, lexer_error, NovaError},
    fileposition::FilePosition,
    tokens::{Operator, Token, TokenList},
    ttype::TType,
};

#[derive(Debug, PartialEq, Eq)]
enum LexerState {
    Token,
    Char,
    String,
    Comment,
    Float,
}

#[derive(Debug)]
pub struct Lexer {
    line: usize,
    row: usize,
    filepath: String,
    source_file: String,
    token_list: TokenList,
    buffer: String,
    state: LexerState,
}

impl Default for Lexer {
    fn default() -> Self {
        Self {
            line: 1,
            row: 1,
            filepath: Default::default(),
            source_file: Default::default(),
            token_list: Default::default(),
            buffer: Default::default(),
            state: LexerState::Token,
        }
    }
}

impl Lexer {
    // Move opening file into seperate section
    pub fn new(filepath: &str) -> Result<Lexer, NovaError> {
        let source = match std::fs::read_to_string(filepath) {
            Ok(content) => content,
            Err(_) => return Err(file_error(format!(" '{filepath}' is not a valid filepath"))),
        };
        Ok(Lexer {
            line: 1,
            row: 1,
            filepath: filepath.to_string(),
            source_file: source,
            token_list: Default::default(),
            buffer: Default::default(),
            state: LexerState::Token,
        })
    }

    fn current_position(&self) -> FilePosition {
        return FilePosition {
            line: self.line,
            row: self.row,
            filepath: self.filepath.to_string(),
        };
    }

    fn current_position_buffer_offset(&self) -> FilePosition {
        return FilePosition {
            line: self.line,
            row: self.row - self.buffer.len(),
            filepath: self.filepath.clone(),
        };
    }

    fn current_position_plus_offset(&self, offset: usize) -> FilePosition {
        return FilePosition {
            line: self.line,
            row: self.row + offset,
            filepath: self.filepath.clone(),
        };
    }

    fn check_token_buffer(&mut self) -> Option<Token> {
        if !self.buffer.is_empty() {
            if let Ok(v) = self.buffer.parse() {
                return Some(if self.buffer.contains('.') {
                    self.state = LexerState::Token;
                    Token::Float(v, self.current_position_buffer_offset())
                } else {
                    Token::Integer(v as i64, self.current_position_buffer_offset())
                });
            }
            // splits buffers begining with a number, 1.print()
            if self.buffer.contains('.') {
                let preset = self.row - self.buffer.len();
                let mut offset = 0;
                let split = self.buffer.split('.');
                for id in split {
                    if let Ok(v) = id.parse::<i64>() {
                        self.state = LexerState::Token;
                        self.token_list.push(Token::Integer(
                            v as i64,
                            self.current_position_plus_offset(preset + offset),
                        ));
                    } else {
                        self.token_list.push(Token::Identifier(
                            id.to_string(),
                            self.current_position_plus_offset(preset + offset),
                        ));
                    }
                    offset += id.len();
                    self.token_list.push(Token::Symbol(
                        '.',
                        self.current_position_plus_offset(preset + offset),
                    ));
                    offset += 1;
                }
                self.token_list.pop();
                return None;
            }
            // consider adding keywords like let,if,for,type,fn
            match self.buffer.as_str() {
                "false" => return Some(Token::Bool(false, self.current_position_buffer_offset())),
                "true" => return Some(Token::Bool(true, self.current_position_buffer_offset())),
                "Int" => {
                    return Some(Token::Type(
                        TType::Int,
                        self.current_position_buffer_offset(),
                    ))
                }
                "Float" => {
                    return Some(Token::Type(
                        TType::Float,
                        self.current_position_buffer_offset(),
                    ))
                }
                "Bool" => {
                    return Some(Token::Type(
                        TType::Bool,
                        self.current_position_buffer_offset(),
                    ))
                }
                "String" => {
                    return Some(Token::Type(
                        TType::String,
                        self.current_position_buffer_offset(),
                    ))
                }
                "Any" => {
                    return Some(Token::Type(
                        TType::Any,
                        self.current_position_buffer_offset(),
                    ))
                }
                "Char" => {
                    return Some(Token::Type(
                        TType::Char,
                        self.current_position_buffer_offset(),
                    ))
                }
                _ => {
                    return Some(Token::Identifier(
                        self.buffer.to_string(),
                        self.current_position_buffer_offset(),
                    ))
                }
            }
        }
        None
    }

    fn check_token(&mut self) {
        if let Some(token) = self.check_token_buffer() {
            self.token_list.push(token);
        }
        self.buffer.clear();
    }

    pub fn tokenize(mut self) -> Result<TokenList, NovaError> {
        if self.filepath.is_empty() {
            // consider making the error take a Position struct
            return Err(lexer_error(
                format!("Missing filepath "),
                format!(""),
                0,
                0,
                "".to_string(),
            ));
        }

        let tempstr = self.source_file.clone();
        let mut chars = tempstr.chars().peekable();

        while let Some(c) = chars.next() {
            if self.state == LexerState::Comment {
                if c != '\n' {
                    self.row += 1;
                    continue;
                } else {
                    self.state = LexerState::Token;
                    self.line += 1;
                    self.row = 0;
                    continue;
                }
            }
            if self.state == LexerState::String {
                if c == '\\' {
                    match chars.peek() {
                        Some('n') => {
                            chars.next();
                            self.buffer.push('\n');
                            self.row += 1;
                            continue;
                        }
                        Some('t') => {
                            chars.next();
                            self.buffer.push('\t');
                            self.row += 1;
                            continue;
                        }
                        Some('r') => {
                            chars.next();
                            self.buffer.push('\r');
                            self.row += 1;
                            continue;
                        }
                        Some('\'') => {
                            chars.next();
                            self.buffer.push('\'');
                            self.row += 1;
                            continue;
                        }
                        Some('\"') => {
                            chars.next();
                            self.buffer.push('\"');
                            self.row += 1;
                            continue;
                        }
                        Some('0') => {
                            chars.next();
                            self.buffer.push('\0');
                            self.row += 1;
                            continue;
                        }
                        Some('\\') => {
                            chars.next();
                            self.buffer.push('\\');
                            self.row += 1;
                            continue;
                        }
                        _ => {
                            // consider making the error take a Position struct
                            return Err(common::error::lexer_error(
                                "Expecting valid escape char".to_string(),
                                "".to_string(),
                                self.line,
                                self.row - self.buffer.len(),
                                self.filepath.clone(),
                            ));
                        }
                    }
                }
                if c != '"' {
                    self.buffer.push(c);
                    continue;
                } else {
                    self.state = LexerState::Token;
                    self.token_list
                        .push(Token::String(self.buffer.clone(), self.current_position()));
                    self.row += 1;
                    self.row += self.buffer.len();
                    self.buffer.clear();
                    continue;
                }
            }
            if self.state == LexerState::Char {
                if c == '\\' {
                    match chars.peek() {
                        Some('n') => {
                            chars.next();
                            self.token_list
                                .push(Token::Char('\n', self.current_position()));
                            self.row += 1;
                            self.buffer.clear();
                            continue;
                        }
                        Some('t') => {
                            chars.next();
                            self.token_list
                                .push(Token::Char('\t', self.current_position()));
                            self.row += 1;
                            self.buffer.clear();
                            continue;
                        }
                        Some('r') => {
                            chars.next();
                            self.token_list
                                .push(Token::Char('\r', self.current_position()));
                            self.row += 1;
                            self.buffer.clear();
                            continue;
                        }
                        Some('h') => {
                            chars.next();
                            self.token_list.push(Token::String(
                                "\x1b[?25h".to_string(),
                                self.current_position(),
                            ));
                            self.row += 1;
                            self.buffer.clear();
                            continue;
                        }
                        Some('l') => {
                            chars.next();
                            self.token_list.push(Token::String(
                                "\x1b[?25l".to_string(),
                                self.current_position(),
                            ));
                            self.row += 1;
                            self.buffer.clear();
                            continue;
                        }
                        Some('\'') => {
                            chars.next();
                            self.token_list
                                .push(Token::Char('\'', self.current_position()));
                            self.row += 1;
                            self.buffer.clear();
                            continue;
                        }
                        Some('\"') => {
                            chars.next();
                            self.token_list
                                .push(Token::Char('\"', self.current_position()));
                            self.row += 1;
                            self.buffer.clear();
                            continue;
                        }
                        Some('0') => {
                            chars.next();
                            self.token_list
                                .push(Token::Char('\0', self.current_position()));
                            self.row += 1;
                            self.buffer.clear();
                            continue;
                        }
                        Some('\\') => {
                            chars.next();
                            self.token_list
                                .push(Token::Char('\\', self.current_position()));
                            self.row += 1;
                            self.buffer.clear();
                            continue;
                        }
                        _ => {
                            return Err(common::error::lexer_error(
                                "Expecting valid escape char".to_string(),
                                "".to_string(),
                                self.line,
                                self.row - self.buffer.len(),
                                self.filepath.clone(),
                            ));
                        }
                    }
                } else if c == '\'' {
                    self.state = LexerState::Token;
                    self.row += 1;
                    self.buffer.clear();
                    continue;
                } else {
                    self.token_list
                        .push(Token::Char(c, self.current_position()));
                    self.row += 1;
                    self.buffer.clear();
                    continue;
                }
            }

            match c {
                '\'' => {
                    self.row += 1;
                    self.state = LexerState::Char;
                    self.check_token();
                }
                '"' => {
                    self.state = LexerState::String;
                    self.check_token();
                }
                '\n' => {
                    self.check_token();
                    self.line += 1;
                    self.row = 0;
                }
                '\r' => {
                    self.check_token();
                }
                '\t' => {
                    self.check_token();
                }
                '.' => {
                    if self.state != LexerState::Float {
                        if let Ok(v) = self.buffer.parse() {
                            let _n: i64 = v;
                            self.state = LexerState::Float;
                            self.buffer.push(c);
                        } else {
                            self.check_token();
                            self.token_list
                                .push(Token::Symbol(c, self.current_position()));
                        }
                    } else {
                        self.check_token();
                        self.token_list
                            .push(Token::Symbol(c, self.current_position()));
                    }
                }
                'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => {
                    self.buffer.push(c);
                }
                ' ' => {
                    self.check_token();
                }
                '+' | '*' | '/' | '-' | '=' | '<' | '>' | '%' | '!' | ':' | '&' | '|' => {
                    self.check_token();
                    match c {
                        ':' => {
                            if let Some(':') = chars.peek() {
                                chars.next();
                                self.token_list.push(Token::Operator(
                                    Operator::DoubleColon,
                                    self.current_position(),
                                ));
                                self.row += 1;
                            } else {
                                self.token_list
                                    .push(Token::Operator(Operator::Colon, self.current_position()))
                            }
                        }
                        '%' => self
                            .token_list
                            .push(Token::Operator(Operator::Modulo, self.current_position())),
                        '>' => {
                            if let Some('=') = chars.peek() {
                                chars.next();
                                self.token_list.push(Token::Operator(
                                    Operator::GtrOrEqu,
                                    self.current_position(),
                                ));
                                self.row += 1;
                            } else {
                                self.token_list.push(Token::Operator(
                                    Operator::GreaterThan,
                                    self.current_position(),
                                ))
                            }
                        }
                        '<' => {
                            if let Some('=') = chars.peek() {
                                chars.next();
                                self.token_list.push(Token::Operator(
                                    Operator::LssOrEqu,
                                    self.current_position(),
                                ));
                                self.row += 1;
                                continue;
                            }
                            if let Some('-') = chars.peek() {
                                chars.next();

                                self.token_list.push(Token::Operator(
                                    Operator::LeftArrow,
                                    self.current_position(),
                                ));
                                self.row += 1;
                            } else {
                                self.token_list.push(Token::Operator(
                                    Operator::LessThan,
                                    self.current_position(),
                                ))
                            }
                        }
                        '+' => {
                            if let Some('=') = chars.peek() {
                                chars.next();
                                self.token_list.push(Token::Operator(
                                    Operator::AdditionAssignment,
                                    self.current_position(),
                                ));
                                self.row += 1;
                            } else {
                                self.token_list.push(Token::Operator(
                                    Operator::Addition,
                                    self.current_position(),
                                ))
                            }
                        }
                        '-' => {
                            if let Some('>') = chars.peek() {
                                chars.next();
                                self.token_list.push(Token::Operator(
                                    Operator::RightArrow,
                                    self.current_position(),
                                ));
                                self.row += 1;
                            } else if let Some('=') = chars.peek() {
                                chars.next();
                                self.token_list.push(Token::Operator(
                                    Operator::SubtractionAssignment,
                                    self.current_position(),
                                ));
                                self.row += 1;
                            } else {
                                self.token_list.push(Token::Operator(
                                    Operator::Subtraction,
                                    self.current_position(),
                                ))
                            }
                        }
                        '*' => self.token_list.push(Token::Operator(
                            Operator::Multiplication,
                            self.current_position(),
                        )),
                        '/' => {
                            if let Some('/') = chars.peek() {
                                chars.next();
                                self.row += 1;
                                self.state = LexerState::Comment;
                            } else {
                                self.token_list.push(Token::Operator(
                                    Operator::Division,
                                    self.current_position(),
                                ))
                            }
                        }
                        '=' => {
                            if let Some(&'=') = chars.peek() {
                                chars.next();
                                self.token_list.push(Token::Operator(
                                    Operator::Equality,
                                    self.current_position(),
                                ));
                                self.row += 1;
                            } else {
                                self.token_list.push(Token::Operator(
                                    Operator::Assignment,
                                    self.current_position(),
                                ))
                            }
                        }
                        '!' => {
                            if let Some(&'=') = chars.peek() {
                                chars.next();
                                self.token_list.push(Token::Operator(
                                    Operator::NotEqual,
                                    self.current_position(),
                                ));
                                self.row += 1;
                            } else {
                                self.token_list
                                    .push(Token::Operator(Operator::Not, self.current_position()))
                            }
                        }
                        '&' => {
                            if let Some(&'&') = chars.peek() {
                                chars.next();
                                self.token_list
                                    .push(Token::Operator(Operator::And, self.current_position()));
                                self.row += 1;
                            } else {
                                self.token_list
                                    .push(Token::Symbol(c, self.current_position()));
                            }
                        }
                        '|' => {
                            if let Some(&'|') = chars.peek() {
                                chars.next();
                                self.token_list
                                    .push(Token::Operator(Operator::Or, self.current_position()));
                                self.row += 1;
                            } else {
                                self.token_list
                                    .push(Token::Symbol(c, self.current_position()));
                            }
                        }
                        _ => {}
                    }
                }
                ';' | '(' | ')' | '[' | ']' | ',' | '{' | '}' | '$' | '@' | '?' | '#' => {
                    self.check_token();
                    self.token_list
                        .push(Token::Symbol(c, self.current_position()));
                }
                _ => {}
            }
            self.row += 1;
        }

        self.check_token();
        self.token_list.push(Token::EOF(self.current_position()));

        Ok(self.token_list)
    }
}
