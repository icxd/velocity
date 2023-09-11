use std::str::Chars;

use crate::error::{Error, Result};

use super::token::{Token, TokenKind};

pub(crate) fn tokenize(filename: &str, contents: Chars) -> Result<Vec<Token>> {
    let mut tokens: Vec<Token> = vec![];

    let mut line: usize = 1;
    let mut start: usize = 0;
    let mut end: usize = 0;

    let mut chars = contents.peekable();

    while let Some(c) = chars.next() {
        match c {
            ' ' | '\t' | '\r' => {
                start += 1;
                end += 1;
            }
            '\n' => {
                line += 1;
                start = 0;
                end = 0;
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut ident = String::new();
                ident.push(c);
                start = end + 1;
                end += 1;

                while let Some(c) = chars.peek() {
                    match c {
                        'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => {
                            ident.push(*c);
                            chars.next();
                            end += 1;
                        }
                        _ => break,
                    }
                }

                tokens.push(Token {
                    kind: match ident.as_str() {
                        "import" => TokenKind::Import,
                        "struct" => TokenKind::Struct,
                        "fn" => TokenKind::Fn,
                        "mut" => TokenKind::Mut,
                        "var" => TokenKind::Var,
                        "const" => TokenKind::Const,
                        "for" => TokenKind::For,
                        "in" => TokenKind::In,
                        "return" => TokenKind::Return,
                        "and" => TokenKind::And,
                        "or" => TokenKind::Or,
                        "true" => TokenKind::Boolean,
                        "false" => TokenKind::Boolean,
                        "int" => TokenKind::Int,
                        "float" => TokenKind::Float,
                        "bool" => TokenKind::Bool,
                        "char" => TokenKind::Char,
                        _ => TokenKind::Identifier,
                    },
                    value: ident,
                    span: (filename.to_string().into(), (line, start)..(line, end)),
                });
            }
            '0'..='9' => {
                start = end + 1;
                end += 1;
                let mut number = String::new();
                number.push(c);

                while let Some(c) = chars.peek() {
                    match c {
                        '0'..='9' => {
                            number.push(*c);
                            chars.next();
                            end += 1;
                        }
                        _ => break,
                    }
                }

                if let Some(c) = chars.peek() {
                    if *c == '.' {
                        number.push(*c);
                        chars.next();
                        end += 1;

                        while let Some(c) = chars.peek() {
                            match c {
                                '0'..='9' => {
                                    number.push(*c);
                                    chars.next();
                                    end += 1;
                                }
                                _ => break,
                            }
                        }

                        tokens.push(Token {
                            kind: TokenKind::FloatingPoint,
                            value: number,
                            span: (filename.to_string().into(), (line, start)..(line, end)),
                        });
                    } else {
                        tokens.push(Token {
                            kind: TokenKind::Integer,
                            value: number,
                            span: (filename.to_string().into(), (line, start)..(line, end)),
                        });
                    }
                } else {
                    tokens.push(Token {
                        kind: TokenKind::Integer,
                        value: number,
                        span: (filename.to_string().into(), (line, start)..(line, end)),
                    });
                }
            }
            '"' => {
                start = end + 1;
                end += 1;
                let mut string = String::new();

                while let Some(c) = chars.peek() {
                    match c {
                        '"' => {
                            chars.next();
                            end += 1;
                            break;
                        }
                        _ => {
                            string.push(*c);
                            chars.next();
                            end += 1;
                        }
                    }
                }

                tokens.push(Token {
                    kind: TokenKind::String,
                    value: string,
                    span: (filename.to_string().into(), (line, start)..(line, end)),
                });
            }
            '\'' => {
                start = end + 1;
                end += 1;
                let mut character = String::new();

                while let Some(c) = chars.peek() {
                    match c {
                        '\'' => {
                            chars.next();
                            end += 1;
                            break;
                        }
                        _ => {
                            character.push(*c);
                            chars.next();
                            end += 1;
                        }
                    }
                }

                tokens.push(Token {
                    kind: TokenKind::Character,
                    value: character,
                    span: (filename.to_string().into(), (line, start)..(line, end)),
                });
            }
            '+' => {
                start = end + 1;
                end += 1;
                if let Some(c) = chars.peek() {
                    if *c == '=' {
                        tokens.push(Token {
                            kind: TokenKind::PlusEquals,
                            value: "+=".to_string(),
                            span: (filename.to_string().into(), (line, start)..(line, end + 1)),
                        });
                        chars.next();
                        end += 1;
                    }
                } else {
                    tokens.push(Token {
                        kind: TokenKind::Plus,
                        value: "+".to_string(),
                        span: (filename.to_string().into(), (line, start)..(line, end)),
                    });
                }
            }
            '-' => {
                start = end + 1;
                end += 1;
                if let Some(c) = chars.peek() {
                    if *c == '=' {
                        tokens.push(Token {
                            kind: TokenKind::MinusEquals,
                            value: "-=".to_string(),
                            span: (filename.to_string().into(), (line, start)..(line, end + 1)),
                        });
                        chars.next();
                        end += 1;
                    } else if *c == '>' {
                        tokens.push(Token {
                            kind: TokenKind::Arrow,
                            value: "->".to_string(),
                            span: (filename.to_string().into(), (line, start)..(line, end + 1)),
                        });
                        chars.next();
                        end += 1;
                    }
                } else {
                    tokens.push(Token {
                        kind: TokenKind::Minus,
                        value: "-".to_string(),
                        span: (filename.to_string().into(), (line, start)..(line, end)),
                    });
                }
            }
            '*' => {
                start = end + 1;
                end += 1;
                if let Some(c) = chars.peek() {
                    if *c == '=' {
                        tokens.push(Token {
                            kind: TokenKind::AsteriskEquals,
                            value: "*=".to_string(),
                            span: (filename.to_string().into(), (line, start)..(line, end + 1)),
                        });
                        chars.next();
                        end += 1;
                    }
                } else {
                    tokens.push(Token {
                        kind: TokenKind::Asterisk,
                        value: "*".to_string(),
                        span: (filename.to_string().into(), (line, start)..(line, end)),
                    });
                }
            }
            '/' => {
                start = end + 1;
                end += 1;
                if let Some(c) = chars.peek() {
                    if *c == '=' {
                        tokens.push(Token {
                            kind: TokenKind::SlashEquals,
                            value: "/=".to_string(),
                            span: (filename.to_string().into(), (line, start)..(line, end + 1)),
                        });
                        chars.next();
                        end += 1;
                    } else if *c == '/' {
                        while let Some(c) = chars.peek() {
                            match c {
                                '\n' => {
                                    chars.next();
                                    end += 1;
                                    break;
                                }
                                _ => {
                                    chars.next();
                                    end += 1;
                                }
                            }
                        }
                    }
                } else {
                    tokens.push(Token {
                        kind: TokenKind::Slash,
                        value: "/".to_string(),
                        span: (filename.to_string().into(), (line, start)..(line, end)),
                    });
                }
            }
            '%' => {
                start = end + 1;
                end += 1;
                if let Some(c) = chars.peek() {
                    if *c == '=' {
                        tokens.push(Token {
                            kind: TokenKind::PercentEquals,
                            value: "%=".to_string(),
                            span: (filename.to_string().into(), (line, start)..(line, end + 1)),
                        });
                        chars.next();
                        end += 1;
                    }
                } else {
                    tokens.push(Token {
                        kind: TokenKind::Percent,
                        value: "%".to_string(),
                        span: (filename.to_string().into(), (line, start)..(line, end)),
                    });
                }
            }
            '=' => {
                start = end + 1;
                end += 1;
                if let Some(c) = chars.peek() {
                    if *c == '=' {
                        tokens.push(Token {
                            kind: TokenKind::EqualsEquals,
                            value: "==".to_string(),
                            span: (filename.to_string().into(), (line, start)..(line, end + 1)),
                        });
                        chars.next();
                        end += 1;
                    } else {
                        tokens.push(Token {
                            kind: TokenKind::Equals,
                            value: "=".to_string(),
                            span: (filename.to_string().into(), (line, start)..(line, end)),
                        });
                    }
                } else {
                    tokens.push(Token {
                        kind: TokenKind::Equals,
                        value: "=".to_string(),
                        span: (filename.to_string().into(), (line, start)..(line, end)),
                    });
                }
            }
            '!' => {
                start = end + 1;
                end += 1;
                if let Some(c) = chars.peek() {
                    if *c == '=' {
                        tokens.push(Token {
                            kind: TokenKind::BangEquals,
                            value: "!=".to_string(),
                            span: (filename.to_string().into(), (line, start)..(line, end + 1)),
                        });
                        chars.next();
                        end += 1;
                    } else {
                        tokens.push(Token {
                            kind: TokenKind::Bang,
                            value: "!".to_string(),
                            span: (filename.to_string().into(), (line, start)..(line, end)),
                        });
                    }
                } else {
                    tokens.push(Token {
                        kind: TokenKind::Bang,
                        value: "!".to_string(),
                        span: (filename.to_string().into(), (line, start)..(line, end)),
                    });
                }
            }
            '<' => {
                start = end + 1;
                end += 1;
                if let Some(c) = chars.peek() {
                    if *c == '=' {
                        tokens.push(Token {
                            kind: TokenKind::LessThanEquals,
                            value: "<=".to_string(),
                            span: (filename.to_string().into(), (line, start)..(line, end + 1)),
                        });
                        chars.next();
                        end += 1;
                    } else {
                        tokens.push(Token {
                            kind: TokenKind::LessThan,
                            value: "<".to_string(),
                            span: (filename.to_string().into(), (line, start)..(line, end)),
                        });
                    }
                } else {
                    tokens.push(Token {
                        kind: TokenKind::LessThan,
                        value: "<".to_string(),
                        span: (filename.to_string().into(), (line, start)..(line, end)),
                    });
                }
            }
            '>' => {
                start = end + 1;
                end += 1;
                if let Some(c) = chars.peek() {
                    if *c == '=' {
                        tokens.push(Token {
                            kind: TokenKind::GreaterThanEquals,
                            value: ">=".to_string(),
                            span: (filename.to_string().into(), (line, start)..(line, end + 1)),
                        });
                        chars.next();
                        end += 1;
                    } else {
                        tokens.push(Token {
                            kind: TokenKind::GreaterThan,
                            value: ">".to_string(),
                            span: (filename.to_string().into(), (line, start)..(line, end)),
                        });
                    }
                } else {
                    tokens.push(Token {
                        kind: TokenKind::GreaterThan,
                        value: ">".to_string(),
                        span: (filename.to_string().into(), (line, start)..(line, end)),
                    });
                }
            }
            '(' => {
                start = end + 1;
                end += 1;
                tokens.push(Token {
                    kind: TokenKind::OpenParenthesis,
                    value: "(".to_string(),
                    span: (filename.to_string().into(), (line, start)..(line, end)),
                });
            }
            ')' => {
                start = end + 1;
                end += 1;
                tokens.push(Token {
                    kind: TokenKind::CloseParenthesis,
                    value: ")".to_string(),
                    span: (filename.to_string().into(), (line, start)..(line, end)),
                });
            }
            '{' => {
                start = end + 1;
                end += 1;
                tokens.push(Token {
                    kind: TokenKind::OpenBrace,
                    value: "{".to_string(),
                    span: (filename.to_string().into(), (line, start)..(line, end)),
                });
            }
            '}' => {
                start = end + 1;
                end += 1;
                tokens.push(Token {
                    kind: TokenKind::CloseBrace,
                    value: "}".to_string(),
                    span: (filename.to_string().into(), (line, start)..(line, end)),
                });
            }
            '[' => {
                start = end + 1;
                end += 1;
                tokens.push(Token {
                    kind: TokenKind::OpenBracket,
                    value: "[".to_string(),
                    span: (filename.to_string().into(), (line, start)..(line, end)),
                });
            }
            ']' => {
                start = end + 1;
                end += 1;
                tokens.push(Token {
                    kind: TokenKind::CloseBracket,
                    value: "]".to_string(),
                    span: (filename.to_string().into(), (line, start)..(line, end)),
                });
            }
            ',' => {
                start = end + 1;
                end += 1;
                tokens.push(Token {
                    kind: TokenKind::Comma,
                    value: ",".to_string(),
                    span: (filename.to_string().into(), (line, start)..(line, end)),
                });
            }
            '.' => {
                start = end + 1;
                end += 1;
                tokens.push(Token {
                    kind: TokenKind::Dot,
                    value: ".".to_string(),
                    span: (filename.to_string().into(), (line, start)..(line, end)),
                });
            }
            ':' => {
                start = end + 1;
                end += 1;
                tokens.push(Token {
                    kind: TokenKind::Colon,
                    value: ":".to_string(),
                    span: (filename.to_string().into(), (line, start)..(line, end)),
                });
            }
            ';' => {
                start = end + 1;
                end += 1;
                tokens.push(Token {
                    kind: TokenKind::Semicolon,
                    value: ";".to_string(),
                    span: (filename.to_string().into(), (line, start)..(line, end)),
                });
            }
            '&' => {
                start = end + 1;
                end += 1;
                tokens.push(Token {
                    kind: TokenKind::Ampersand,
                    value: "&".to_string(),
                    span: (filename.to_string().into(), (line, start)..(line, end)),
                });
            }
            '|' => {
                start = end + 1;
                end += 1;
                tokens.push(Token {
                    kind: TokenKind::Pipe,
                    value: "|".to_string(),
                    span: (filename.to_string().into(), (line, start)..(line, end)),
                });
            }
            _ => {
                start = end;
                return Err(Error::new(
                    format!("unexpected character: {:?}", c),
                    (filename.to_string().into(), (line, start)..(line, end)),
                ));
            }
        }
    }

    tokens.push(Token {
        kind: TokenKind::Eof,
        value: "<eof>".to_string(),
        span: (filename.to_string().into(), (line, start)..(line, end)),
    });

    Ok(tokens)
}
