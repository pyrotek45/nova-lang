use common::{
    error::NovaError,
    fileposition::{load_file_content, FilePosition},
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
    string_start_position: Vec<usize>,
    char_start_position: Vec<usize>,
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
            string_start_position: vec![],
            char_start_position: vec![],
        }
    }
}

impl Lexer {
    pub fn new(filepath: &str) -> Result<Lexer, NovaError> {
        let source = match load_file_content(filepath) {
            Ok(value) => value,
            Err(value) => return Err(value),
        };
        Ok(Lexer {
            line: 1,
            row: 1,
            filepath: filepath.to_string(),
            source_file: source,
            token_list: Default::default(),
            buffer: Default::default(),
            state: LexerState::Token,
            string_start_position: vec![],
            char_start_position: vec![],
        })
    }

    fn current_position(&self) -> FilePosition {
        return FilePosition {
            line: self.line,
            row: self.row,
            filepath: self.filepath.to_string(),
        };
    }

    fn current_position_buffer_row(&self, row: usize) -> FilePosition {
        return FilePosition {
            line: self.line,
            row: row,
            filepath: self.filepath.clone(),
        };
    }

    fn check_token_buffer(&mut self) -> Option<Token> {
        if !self.buffer.is_empty() {
            if let Ok(v) = self.buffer.parse() {
                return Some(if self.buffer.contains('.') {
                    self.state = LexerState::Token;
                    Token::Float {
                        value: v,
                        position: self
                            .current_position_buffer_row(self.row - self.buffer.chars().count()),
                    }
                } else {
                    Token::Integer {
                        value: v as i64,
                        position: self.current_position(),
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
                            value: v as i64,
                            position: self
                                .current_position_buffer_row(self.row - id.chars().count()),
                        });
                    } else {
                        self.token_list.push(Token::Identifier {
                            name: id.to_string(),
                            position: self
                                .current_position_buffer_row(self.row - id.chars().count()),
                        });
                    }
                    self.token_list.push(Token::Symbol {
                        symbol: '.',
                        position: self.current_position_buffer_row(self.row - id.chars().count()),
                    });
                }

                self.token_list.pop();

                if let Some('.') = lastchar {
                    self.token_list.push(Token::Symbol {
                        symbol: '.',
                        position: self.current_position_buffer_row(self.row + 1),
                    });
                }
                return None;
            }

            // Consider adding keywords like let, if, for, type, fn
            match self.buffer.as_str() {
                "false" => {
                    return Some(Token::Bool {
                        value: false,
                        position: self
                            .current_position_buffer_row(self.row - self.buffer.chars().count()),
                    })
                }
                "true" => {
                    return Some(Token::Bool {
                        value: true,
                        position: self
                            .current_position_buffer_row(self.row - self.buffer.chars().count()),
                    })
                }
                "Int" => {
                    return Some(Token::Type {
                        ttype: TType::Int,
                        position: self
                            .current_position_buffer_row(self.row - self.buffer.chars().count()),
                    })
                }
                "Float" => {
                    return Some(Token::Type {
                        ttype: TType::Float,
                        position: self
                            .current_position_buffer_row(self.row - self.buffer.chars().count()),
                    })
                }
                "Bool" => {
                    return Some(Token::Type {
                        ttype: TType::Bool,
                        position: self
                            .current_position_buffer_row(self.row - self.buffer.chars().count()),
                    })
                }
                "String" => {
                    return Some(Token::Type {
                        ttype: TType::String,
                        position: self
                            .current_position_buffer_row(self.row - self.buffer.chars().count()),
                    })
                }
                "Any" => {
                    return Some(Token::Type {
                        ttype: TType::Any,
                        position: self
                            .current_position_buffer_row(self.row - self.buffer.chars().count()),
                    })
                }
                "Char" => {
                    return Some(Token::Type {
                        ttype: TType::Char,
                        position: self
                            .current_position_buffer_row(self.row - self.buffer.chars().count()),
                    })
                }
                _ => {
                    return Some(Token::Identifier {
                        name: self.buffer.to_string(),
                        position: self
                            .current_position_buffer_row(self.row - self.buffer.chars().count()),
                    })
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

    pub fn tokenize(&mut self) -> Result<TokenList, NovaError> {
        if self.filepath.is_empty() {
            // Consider making the error take a Position struct
            return Err(NovaError::Lexing {
                msg: String::from("File is missing"),
                note: String::from("Check the files location was typed correctly"),
                position: self.current_position(),
            });
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
                    self.row = 1;
                    continue;
                }
            }
            if self.state == LexerState::String {
                if c == '\\' {
                    match chars.peek() {
                        Some('n') => {
                            chars.next();
                            self.buffer.push('\n');
                            self.row += 2;
                            continue;
                        }
                        Some('t') => {
                            chars.next();
                            self.buffer.push('\t');
                            self.row += 2;
                            continue;
                        }
                        Some('r') => {
                            chars.next();
                            self.buffer.push('\r');
                            self.row += 2;
                            continue;
                        }
                        Some('\'') => {
                            chars.next();
                            self.buffer.push('\'');
                            self.row += 2;
                            continue;
                        }
                        Some('\"') => {
                            chars.next();
                            self.buffer.push('\"');
                            self.row += 2;
                            continue;
                        }
                        Some('0') => {
                            chars.next();
                            self.buffer.push('\0');
                            self.row += 2;
                            continue;
                        }
                        Some('\\') => {
                            chars.next();
                            self.buffer.push('\\');
                            self.row += 2;
                            continue;
                        }
                        _ => {
                            println!("{}", self.line);
                            return Err(NovaError::Lexing {
                                msg: String::from("Expecting valid escape char"),
                                note: String::from(""),
                                position: self.current_position(),
                            });
                        }
                    }
                }
                if c != '"' {
                    if c == '\n' {
                        self.line += 1;
                        self.row = 1;
                    } else {
                        self.row += 1;
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
                    self.row += 1;
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
                            self.row += 2;
                            continue;
                        }
                        Some('t') => {
                            chars.next();
                            self.buffer.push('\t');
                            self.row += 2;
                            continue;
                        }
                        Some('r') => {
                            chars.next();
                            self.buffer.push('\r');
                            self.row += 2;
                            continue;
                        }
                        Some('\'') => {
                            chars.next();
                            self.buffer.push('\'');
                            self.row += 2;
                            continue;
                        }
                        Some('\"') => {
                            chars.next();
                            self.buffer.push('\"');
                            self.row += 2;
                            continue;
                        }
                        Some('0') => {
                            chars.next();
                            self.buffer.push('\0');
                            self.row += 2;
                            continue;
                        }
                        Some('\\') => {
                            chars.next();
                            self.buffer.push('\\');
                            self.row += 2;
                            continue;
                        }
                        _ => {
                            return Err(NovaError::Lexing {
                                msg: String::from("Expecting valid escape char"),
                                note: String::from(""),
                                position: self.current_position(),
                            })
                        }
                    }
                } else if c == '\'' {
                    self.state = LexerState::Token;
                    // should throw error, cant have ''
                    if self.buffer.is_empty() || self.buffer.chars().count() > 1 {
                        return Err(NovaError::Lexing {
                            msg: String::from("Expecting valid char"),
                            note: String::from(format!("? {}", self.buffer)),
                            position: self.current_position(),
                        });
                    }
                    let char_start = self.char_start_position.pop().unwrap();
                    self.token_list.push(Token::Char {
                        value: self.buffer.chars().next().unwrap(),
                        position: self.current_position_buffer_row(char_start),
                    });
                    self.row += 1;
                    self.buffer.clear();
                    continue;
                } else {
                    self.buffer.push(c);
                    self.row += 1;
                    continue;
                }
            }

            match c {
                '\'' => {
                    self.state = LexerState::Char;
                    self.check_token();
                    self.char_start_position.push(self.row);
                }
                '"' => {
                    self.state = LexerState::String;
                    self.check_token();
                    self.string_start_position.push(self.row);
                }
                '\n' => {
                    self.check_token();
                    self.line += 1;
                    self.row = 1;
                    continue;
                }
                '\r' => self.check_token(),
                '\t' => self.check_token(),
                '.' => {
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
                    self.buffer.push(c);
                }
                ' ' => self.check_token(),
                '+' | '*' | '/' | '-' | '=' | '<' | '>' | '%' | '!' | ':' | '&' | '|' => {
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
                                self.row += 1;
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
                                self.row += 1;
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
                                self.row += 1;
                                continue;
                            }
                            if let Some('-') = chars.peek() {
                                chars.next();
                                self.token_list.push(Token::Operator {
                                    operator: Operator::LeftArrow,
                                    position: self.current_position(),
                                });
                                self.row += 1;
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
                                self.row += 1;
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
                                self.row += 1;
                            } else if let Some('=') = chars.peek() {
                                chars.next();
                                self.token_list.push(Token::Operator {
                                    operator: Operator::SubtractionAssignment,
                                    position: self.current_position(),
                                });
                                self.row += 1;
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
                                self.row += 1;
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
                                self.row += 1;
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
                                self.row += 1;
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
                                self.row += 1;
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
                                self.row += 1;
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
            self.row += 1;
        }

        self.check_token();
        self.token_list.push(Token::EOF {
            position: self.current_position(),
        });

        Ok(self.token_list.clone())
    }

    pub fn check(&self) {
        for token in self.token_list.iter() {
            common::error::print_line(&token.position(), &format!("{}", token.to_string()));
        }
    }
}
