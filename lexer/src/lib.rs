use common::{
    error::NovaError,
    tokens::{Operator, Position, TType, Token, TokenList},
};

#[derive(Debug, PartialEq, Eq)]
enum ParsingState {
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
    input: String,
    output: TokenList,
    buffer: String,
    parsing: ParsingState,
}

pub fn new(filepath: &str) -> Result<Lexer, String> {
    match std::fs::read_to_string(filepath.clone()) {
        Ok(content) => Ok(Lexer {
            line: 1,
            row: 1,
            input: content,
            output: vec![],
            buffer: String::new(),
            parsing: ParsingState::Token,
            filepath: filepath.to_owned(),
        }),
        Err(_) => Err(format!("file: {} could not be opened", filepath)),
    }
}

impl Lexer {
    fn check_token_buffer(&mut self) -> Option<Token> {
        if !self.buffer.is_empty() {
            if let Ok(v) = self.buffer.parse() {
                return Some(if self.buffer.contains('.') {
                    self.parsing = ParsingState::Token;
                    Token::Float(
                        v,
                        Position {
                            line: self.line,
                            row: self.row - self.buffer.len(),
                        },
                    )
                } else {
                    Token::Integer(
                        v as i64,
                        Position {
                            line: self.line,
                            row: self.row - self.buffer.len(),
                        },
                    )
                });
            }
            // splits buffers begining with a number, 1.print()
            if self.buffer.contains('.') {
                let preset = self.row - self.buffer.len();
                let mut offset = 0;
                let split = self.buffer.split('.');
                for id in split {
                    if let Ok(v) = id.parse::<i64>() {
                        self.parsing = ParsingState::Token;
                        self.output.push(Token::Integer(
                            v as i64,
                            Position {
                                line: self.line,
                                row: preset + offset,
                            },
                        ));
                    } else {
                        self.output.push(Token::Identifier(
                            id.to_lowercase(),
                            Position {
                                line: self.line,
                                row: preset + offset,
                            },
                        ));
                    }
                    offset += id.len();
                    self.output.push(Token::Symbol(
                        '.',
                        Position {
                            line: self.line,
                            row: preset + offset,
                        },
                    ));
                    offset += 1;
                }
                self.output.pop();
                return None;
            }
            match self.buffer.to_lowercase().as_str() {
                "none" => {
                    return Some(Token::None(
                        TType::None,
                        Position {
                            line: self.line,
                            row: self.row - self.buffer.len(),
                        },
                    ))
                }
                "false" => {
                    return Some(Token::Bool(
                        false,
                        Position {
                            line: self.line,
                            row: self.row - self.buffer.len(),
                        },
                    ))
                }
                "true" => {
                    return Some(Token::Bool(
                        true,
                        Position {
                            line: self.line,
                            row: self.row - self.buffer.len(),
                        },
                    ))
                }
                "int" => {
                    return Some(Token::Type(
                        TType::Int,
                        Position {
                            line: self.line,
                            row: self.row - self.buffer.len(),
                        },
                    ))
                }
                "float" => {
                    return Some(Token::Type(
                        TType::Float,
                        Position {
                            line: self.line,
                            row: self.row - self.buffer.len(),
                        },
                    ))
                }
                "bool" => {
                    return Some(Token::Type(
                        TType::Bool,
                        Position {
                            line: self.line,
                            row: self.row - self.buffer.len(),
                        },
                    ))
                }
                "str" => {
                    return Some(Token::Type(
                        TType::Str,
                        Position {
                            line: self.line,
                            row: self.row - self.buffer.len(),
                        },
                    ))
                }
                "void" => {
                    return Some(Token::Type(
                        TType::Void,
                        Position {
                            line: self.line,
                            row: self.row - self.buffer.len(),
                        },
                    ))
                }
                _ => {
                    return Some(Token::Identifier(
                        self.buffer.to_lowercase(),
                        Position {
                            line: self.line,
                            row: self.row - self.buffer.len(),
                        },
                    ))
                }
            }
        }
        None
    }

    pub fn check_token(&mut self) -> Result<(), NovaError> {
        if let Some(token) = self.check_token_buffer() {
            self.output.push(token);
        }
        self.buffer.clear();
        Ok(())
    }

    pub fn tokenize(&mut self) -> Result<&TokenList, NovaError> {
        let tempstr = self.input.clone();
        let mut chars = tempstr.chars().peekable();

        while let Some(c) = chars.next() {
            if self.parsing == ParsingState::Comment {
                if c != '\n' {
                    continue;
                } else {
                    self.parsing = ParsingState::Token
                }
            }
            if self.parsing == ParsingState::String {
                if c != '"' {
                    self.buffer.push(c);
                    continue;
                } else {
                    self.parsing = ParsingState::Token;
                    self.output.push(Token::String(
                        self.buffer.clone(),
                        Position {
                            line: self.line,
                            row: self.row,
                        },
                    ));
                    self.row += 1;
                    self.row += self.buffer.len();
                    self.buffer.clear();
                    continue;
                }
            }
            if self.parsing == ParsingState::Char {
                if c != '\'' {
                    self.buffer.push(c);
                    continue;
                } else {
                    self.parsing = ParsingState::Token;
                    if self.buffer.len() > 1 {
                        return Err(common::error::lexer_error(
                            "Char cannot contain more than one character".to_string(),
                            "Try using double quotes instead, if you need a string".to_string(),
                            self.line,
                            self.row - self.buffer.len(),
                            self.filepath.clone(),
                        ));
                    }
                    if let Some(ch) = self.buffer.chars().next() {
                        self.output.push(Token::Char(
                            ch,
                            Position {
                                line: self.line,
                                row: self.row,
                            },
                        ));
                    }
                    self.row += 1;
                    self.buffer.clear();
                    continue;
                }
            }
            match c {
                '\'' => {
                    self.row += 1;
                    self.parsing = ParsingState::Char;
                    self.check_token()?;
                }
                '"' => {
                    self.parsing = ParsingState::String;
                    self.check_token()?;
                }
                '\n' => {
                    self.check_token()?;
                    self.output.push(Token::NewLine(Position {
                        line: self.line,
                        row: self.row,
                    }));
                    self.line += 1;
                    self.row = 0;
                }
                '.' => {
                    if self.parsing != ParsingState::Float {
                        if let Ok(v) = self.buffer.parse() {
                            let _n: i64 = v;
                            self.parsing = ParsingState::Float;
                            self.buffer.push(c);
                        } else {
                            self.check_token()?;
                            self.output.push(Token::Symbol(
                                c,
                                Position {
                                    line: self.line,
                                    row: self.row,
                                },
                            ));
                        }
                    } else {
                        self.check_token()?;
                        self.output.push(Token::Symbol(
                            c,
                            Position {
                                line: self.line,
                                row: self.row,
                            },
                        ));
                    }
                }
                'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => {
                    self.buffer.push(c);
                }
                ' ' => {
                    self.check_token()?;
                }
                '+' | '*' | '/' | '-' | '=' | '<' | '>' | '%' | '!' | ':' | '&' | '|' => {
                    self.check_token()?;
                    match c {
                        ':' => {
                            if let Some(':') = chars.peek() {
                                chars.next();
                                self.row += 1;
                                self.output.push(Token::Operator(
                                    Operator::DoubleColon,
                                    Position {
                                        line: self.line,
                                        row: self.row - 1,
                                    },
                                ))
                            } else {
                                self.output.push(Token::Operator(
                                    Operator::Colon,
                                    Position {
                                        line: self.line,
                                        row: self.row,
                                    },
                                ))
                            }
                        }
                        '%' => self.output.push(Token::Operator(
                            Operator::Modulo,
                            Position {
                                line: self.line,
                                row: self.row,
                            },
                        )),
                        '>' => {
                            if let Some('=') = chars.peek() {
                                chars.next();
                                self.row += 1;
                                self.output.push(Token::Operator(
                                    Operator::GtrOrEqu,
                                    Position {
                                        line: self.line,
                                        row: self.row - 1,
                                    },
                                ))
                            } else {
                                self.output.push(Token::Operator(
                                    Operator::GreaterThan,
                                    Position {
                                        line: self.line,
                                        row: self.row,
                                    },
                                ))
                            }
                        }
                        '<' => {
                            if let Some('=') = chars.peek() {
                                chars.next();
                                self.row += 1;
                                self.output.push(Token::Operator(
                                    Operator::LssOrEqu,
                                    Position {
                                        line: self.line,
                                        row: self.row - 1,
                                    },
                                ))
                            } else {
                                self.output.push(Token::Operator(
                                    Operator::LessThan,
                                    Position {
                                        line: self.line,
                                        row: self.row,
                                    },
                                ))
                            }
                        }
                        '+' => {
                            if let Some('=') = chars.peek() {
                                chars.next();
                                self.row += 1;
                                self.output.push(Token::Operator(
                                    Operator::AdditionAssignment,
                                    Position {
                                        line: self.line,
                                        row: self.row - 1,
                                    },
                                ))
                            } else {
                                self.output.push(Token::Operator(
                                    Operator::Addition,
                                    Position {
                                        line: self.line,
                                        row: self.row,
                                    },
                                ))
                            }
                        }
                        '-' => {
                            if let Some('>') = chars.peek() {
                                chars.next();
                                self.row += 1;
                                self.output.push(Token::Operator(
                                    Operator::RightArrow,
                                    Position {
                                        line: self.line,
                                        row: self.row - 1,
                                    },
                                ))
                            } else if let Some('=') = chars.peek() {
                                chars.next();
                                self.row += 1;
                                self.output.push(Token::Operator(
                                    Operator::SubtractionAssignment,
                                    Position {
                                        line: self.line,
                                        row: self.row - 1,
                                    },
                                ))
                            } else {
                                self.output.push(Token::Operator(
                                    Operator::Subtraction,
                                    Position {
                                        line: self.line,
                                        row: self.row,
                                    },
                                ))
                            }
                        }
                        '*' => self.output.push(Token::Operator(
                            Operator::Multiplication,
                            Position {
                                line: self.line,
                                row: self.row,
                            },
                        )),
                        '/' => {
                            if let Some('/') = chars.peek() {
                                chars.next();
                                self.row += 1;
                                self.parsing = ParsingState::Comment;
                            } else {
                                self.output.push(Token::Operator(
                                    Operator::Division,
                                    Position {
                                        line: self.line,
                                        row: self.row,
                                    },
                                ))
                            }
                        }
                        '=' => {
                            if let Some(&'=') = chars.peek() {
                                chars.next();
                                self.row += 1;
                                self.output.push(Token::Operator(
                                    Operator::Equality,
                                    Position {
                                        line: self.line,
                                        row: self.row - 1,
                                    },
                                ))
                            } else {
                                self.output.push(Token::Operator(
                                    Operator::Assignment,
                                    Position {
                                        line: self.line,
                                        row: self.row,
                                    },
                                ))
                            }
                        }
                        '!' => {
                            if let Some(&'=') = chars.peek() {
                                chars.next();
                                self.row += 1;
                                self.output.push(Token::Operator(
                                    Operator::NotEqual,
                                    Position {
                                        line: self.line,
                                        row: self.row - 1,
                                    },
                                ))
                            } else {
                                self.output.push(Token::Operator(
                                    Operator::Not,
                                    Position {
                                        line: self.line,
                                        row: self.row,
                                    },
                                ))
                            }
                        }
                        '&' => {
                            if let Some(&'&') = chars.peek() {
                                chars.next();
                                self.row += 1;
                                self.output.push(Token::Operator(
                                    Operator::And,
                                    Position {
                                        line: self.line,
                                        row: self.row - 1,
                                    },
                                ))
                            } else {
                                self.output.push(Token::Symbol(
                                    c,
                                    Position {
                                        line: self.line,
                                        row: self.row,
                                    },
                                ));
                            }
                        }
                        '|' => {
                            if let Some(&'|') = chars.peek() {
                                chars.next();
                                self.row += 1;
                                self.output.push(Token::Operator(
                                    Operator::Or,
                                    Position {
                                        line: self.line,
                                        row: self.row - 1,
                                    },
                                ))
                            } else {
                                self.output.push(Token::Symbol(
                                    c,
                                    Position {
                                        line: self.line,
                                        row: self.row,
                                    },
                                ));
                            }
                        }
                        _ => {}
                    }
                }
                ';' | '(' | ')' | '[' | ']' | ',' | '{' | '}' | '$' | '@' => {
                    self.check_token()?;
                    self.output.push(Token::Symbol(
                        c,
                        Position {
                            line: self.line,
                            row: self.row,
                        },
                    ));
                }
                error => {
                    return Err(common::error::lexer_error(
                        format!("Unknown char {error}"),
                        format!("Remove char {error}"),
                        self.line,
                        self.row,
                        self.filepath.clone(),
                    ));
                }
            }
            self.row += 1;
        }
        self.check_token()?;
        self.output.push(Token::EOF(Position {
            line: self.line,
            row: self.row,
        }));
        Ok(&self.output)
    }
}
