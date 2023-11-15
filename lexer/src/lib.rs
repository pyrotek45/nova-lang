use common::{
    error::{file_error, lexer_error, NovaError},
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
    source: String,
    tokens: TokenList,
    buffer: String,
    parsing: ParsingState,
}

impl Default for Lexer {
    fn default() -> Self {
        Self {
            line: 1,
            row: 1,
            filepath: Default::default(),
            source: Default::default(),
            tokens: Default::default(),
            buffer: Default::default(),
            parsing: ParsingState::Token,
        }
    }
}

impl Lexer {
    
    pub fn new(filepath: &str) -> Result<Lexer, NovaError> {
        let source = match std::fs::read_to_string(filepath) {
            Ok(content) => content,
            Err(_) => return Err(file_error(format!(" '{filepath}' is not a valid filepath"))),
        };
        Ok(Lexer {
            line: 1,
            row: 1,
            filepath: filepath.to_string(),
            source,
            tokens: Default::default(),
            buffer: Default::default(),
            parsing: ParsingState::Token,
        })
    }

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
                        self.tokens.push(Token::Integer(
                            v as i64,
                            Position {
                                line: self.line,
                                row: preset + offset,
                            },
                        ));
                    } else {
                        self.tokens.push(Token::Identifier(
                            id.to_string(),
                            Position {
                                line: self.line,
                                row: preset + offset,
                            },
                        ));
                    }
                    offset += id.len();
                    self.tokens.push(Token::Symbol(
                        '.',
                        Position {
                            line: self.line,
                            row: preset + offset,
                        },
                    ));
                    offset += 1;
                }
                self.tokens.pop();
                return None;
            }
            match self.buffer.as_str() {
                "False" => {
                    return Some(Token::Bool(
                        false,
                        Position {
                            line: self.line,
                            row: self.row - self.buffer.len(),
                        },
                    ))
                }
                "True" => {
                    return Some(Token::Bool(
                        true,
                        Position {
                            line: self.line,
                            row: self.row - self.buffer.len(),
                        },
                    ))
                }
                "Int" => {
                    return Some(Token::Type(
                        TType::Int,
                        Position {
                            line: self.line,
                            row: self.row - self.buffer.len(),
                        },
                    ))
                }
                "Float" => {
                    return Some(Token::Type(
                        TType::Float,
                        Position {
                            line: self.line,
                            row: self.row - self.buffer.len(),
                        },
                    ))
                }
                "Bool" => {
                    return Some(Token::Type(
                        TType::Bool,
                        Position {
                            line: self.line,
                            row: self.row - self.buffer.len(),
                        },
                    ))
                }
                "String" => {
                    return Some(Token::Type(
                        TType::String,
                        Position {
                            line: self.line,
                            row: self.row - self.buffer.len(),
                        },
                    ))
                }
                "Any" => {
                    return Some(Token::Type(
                        TType::Any,
                        Position {
                            line: self.line,
                            row: self.row - self.buffer.len(),
                        },
                    ))
                }
                "Char" => {
                    return Some(Token::Type(
                        TType::Char,
                        Position {
                            line: self.line,
                            row: self.row - self.buffer.len(),
                        },
                    ))
                }
                _ => {
                    return Some(Token::Identifier(
                        self.buffer.to_string(),
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

    fn check_token(&mut self) -> Result<(), NovaError> {
        if let Some(token) = self.check_token_buffer() {
            self.tokens.push(token);
        }
        self.buffer.clear();
        Ok(())
    }

    pub fn tokenize(mut self) -> Result<TokenList, NovaError> {
        if self.filepath.is_empty() {
            return Err(lexer_error(
                format!("Missing filepath "),
                format!(""),
                0,
                0,
                "".to_string(),
            ));
        }

        let tempstr = self.source.clone();
        let mut chars = tempstr.chars().peekable();

        while let Some(c) = chars.next() {
            if self.parsing == ParsingState::Comment {
                if c != '\n' {
                    self.row += 1;
                    continue;
                } else {
                    self.parsing = ParsingState::Token;
                    self.line += 1;
                    self.row = 0;
                    continue;
                }
            }
            if self.parsing == ParsingState::String {
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
                    self.parsing = ParsingState::Token;
                    self.tokens.push(Token::String(
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
                        self.tokens.push(Token::Char(
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
                    self.tokens.push(Token::NewLine(Position {
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
                            match self.tokens.last() {
                                Some(Token::NewLine(_)) => {
                                    self.tokens.pop();
                                }
                                _ => {}
                            }
                            self.tokens.push(Token::Symbol(
                                c,
                                Position {
                                    line: self.line,
                                    row: self.row,
                                },
                            ));
                        }
                    } else {
                        self.check_token()?;
                        match self.tokens.last() {
                            Some(Token::NewLine(_)) => {
                                self.tokens.pop();
                            }
                            _ => {}
                        }
                        self.tokens.push(Token::Symbol(
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
                                self.tokens.push(Token::Operator(
                                    Operator::DoubleColon,
                                    Position {
                                        line: self.line,
                                        row: self.row - 1,
                                    },
                                ))
                            } else {
                                self.tokens.push(Token::Operator(
                                    Operator::Colon,
                                    Position {
                                        line: self.line,
                                        row: self.row,
                                    },
                                ))
                            }
                        }
                        '%' => self.tokens.push(Token::Operator(
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
                                self.tokens.push(Token::Operator(
                                    Operator::GtrOrEqu,
                                    Position {
                                        line: self.line,
                                        row: self.row - 1,
                                    },
                                ))
                            } else {
                                self.tokens.push(Token::Operator(
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
                                self.tokens.push(Token::Operator(
                                    Operator::LssOrEqu,
                                    Position {
                                        line: self.line,
                                        row: self.row - 1,
                                    },
                                ))
                            } else {
                                self.tokens.push(Token::Operator(
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
                                self.tokens.push(Token::Operator(
                                    Operator::AdditionAssignment,
                                    Position {
                                        line: self.line,
                                        row: self.row - 1,
                                    },
                                ))
                            } else {
                                self.tokens.push(Token::Operator(
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
                                self.tokens.push(Token::Operator(
                                    Operator::RightArrow,
                                    Position {
                                        line: self.line,
                                        row: self.row - 1,
                                    },
                                ))
                            } else if let Some('=') = chars.peek() {
                                chars.next();
                                self.row += 1;
                                self.tokens.push(Token::Operator(
                                    Operator::SubtractionAssignment,
                                    Position {
                                        line: self.line,
                                        row: self.row - 1,
                                    },
                                ))
                            } else {
                                self.tokens.push(Token::Operator(
                                    Operator::Subtraction,
                                    Position {
                                        line: self.line,
                                        row: self.row,
                                    },
                                ))
                            }
                        }
                        '*' => self.tokens.push(Token::Operator(
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
                                self.tokens.push(Token::Operator(
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
                                self.tokens.push(Token::Operator(
                                    Operator::Equality,
                                    Position {
                                        line: self.line,
                                        row: self.row - 1,
                                    },
                                ))
                            } else {
                                self.tokens.push(Token::Operator(
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
                                self.tokens.push(Token::Operator(
                                    Operator::NotEqual,
                                    Position {
                                        line: self.line,
                                        row: self.row - 1,
                                    },
                                ))
                            } else {
                                self.tokens.push(Token::Operator(
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
                                self.tokens.push(Token::Operator(
                                    Operator::And,
                                    Position {
                                        line: self.line,
                                        row: self.row - 1,
                                    },
                                ))
                            } else {
                                self.tokens.push(Token::Symbol(
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
                                self.tokens.push(Token::Operator(
                                    Operator::Or,
                                    Position {
                                        line: self.line,
                                        row: self.row - 1,
                                    },
                                ))
                            } else {
                                self.tokens.push(Token::Symbol(
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
                ';' | '(' | ')' | '[' | ']' | ',' | '{' | '}' | '$' | '@' | '?' | '#' => {
                    self.check_token()?;
                    self.tokens.push(Token::Symbol(
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
        self.tokens.push(Token::EOF(Position {
            line: self.line,
            row: self.row,
        }));
        Ok(self.tokens)
    }
}
