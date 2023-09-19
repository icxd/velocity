use crate::error::{Error, Result};
use crate::span::Span;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TokenKind {
    // literals
    Identifier, // abc
    String,     // "abc"
    Integer,    // 123, 0x123, 0b1010, 17e+2, 17e-2, -123
    Floating,   // 123.456, 123.456e+2, 123.456e-2, -123.456
    // keywords
    As,     // as
    Const,  // const
    Fn,     // fn
    For,    // for
    In,     // in
    Import, // import
    Mut,    // mut
    Return, // return
    Struct, // struct
    Var,    // var
    // types
    Float, // float
    Int,   // int
    // punctuation
    LeftParenthesis,  // (
    RightParenthesis, // )
    LeftBrace,        // {
    RightBrace,       // }
    LeftBracket,      // [
    RightBracket,     // ]
    Comma,            // ,
    Dot,              // .
    Colon,            // :
    // Semicolon,        // ;
    ThinArrow, // ->
    // operators
    Plus,              // +
    PlusEquals,        // +=
    Minus,             // -
    MinusEquals,       // -=
    Asterisk,          // *
    AsteriskEquals,    // *=
    Slash,             // /
    SlashEquals,       // /=
    Percent,           // %
    PercentEquals,     // %=
    Equals,            // =
    EqualsEquals,      // ==
    Bang,              // !
    BangEquals,        // !=
    LessThan,          // <
    LessThanEquals,    // <=
    GreaterThan,       // >
    GreaterThanEquals, // >=
    // logical operators
    And, // &&
    Or,  // ||
    // bitwise operators
    BitwiseAnd, // &
    BitwiseOr,  // |
    BitwiseXor, // ^
    BitwiseNot, // ~
    // special
    Linefeed,
    Indent,
    Dedent,
    Eof,
}

#[derive(Debug, Clone)]
pub(crate) struct Token {
    pub(crate) kind: TokenKind,
    pub(crate) lexeme: Rc<String>,
    pub(crate) span: Span,
}

impl Token {
    pub(crate) fn new(kind: TokenKind, lexeme: Rc<String>, span: Span) -> Token {
        Token { kind, lexeme, span }
    }
}

pub(crate) struct Tokenizer {
    filename: Rc<String>,
    contents: Rc<String>,
    index: usize,
    line: usize,
    column: usize,
    indent_stack: Vec<(usize, bool)>, // (indent, continuation)
}

impl Tokenizer {
    pub(crate) fn new(filename: Rc<String>, contents: String) -> Tokenizer {
        Tokenizer {
            filename,
            contents: Rc::new(contents.clone()),
            index: 0,
            line: 1,
            column: 1,
            indent_stack: vec![(0, false)],
        }
    }

    fn single_token(&mut self, kind: TokenKind) -> Result<Token> {
        let token = Token::new(kind, Rc::new("".to_string()), self.construct_span(1));
        self.index += 1;
        self.column += 1;
        Ok(token)
    }

    fn double_token(
        &mut self,
        char_1: char,
        kind_1: TokenKind,
        char_2: char,
        kind_2: TokenKind,
    ) -> Result<Token> {
        self.index += 1;
        self.column += 1;
        let token = if self.current().unwrap() == char_2 {
            self.index += 1;
            self.column += 1;
            Token::new(
                kind_2,
                Rc::new(format!("{}{}", char_1, char_2)),
                self.construct_span(2),
            )
        } else {
            Token::new(kind_1, Rc::new(char_1.to_string()), self.construct_span(1))
        };
        Ok(token)
    }

    fn error(&self, message: &str, span: Span) -> Error {
        Error::new(message, span)
    }

    fn construct_span(&self, length: usize) -> Span {
        let start = (self.line, self.column);
        let end = (self.line, self.column + length);
        (self.filename.clone(), start..end)
    }

    fn current(&self) -> Option<char> {
        self.contents.chars().nth(self.index)
    }

    pub(crate) fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token()?;
            if token.kind == TokenKind::Eof {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }
        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token> {
        if let None = self.current() {
            return Ok(Token::new(
                TokenKind::Eof,
                Rc::new("".to_string()),
                self.construct_span(0),
            ));
        }
        let c = self.current().unwrap();
        match c {
            // whitespace
            ' ' | '\t' | '\r' => {
                self.index += 1;
                self.column += 1;
                self.next_token()
            }
            // linefeed
            '\n' => {
                // skip newline character
                self.index += 1;
                self.line += 1;
                self.column = 1;
                // calculate indentation
                let mut indent: usize = 0;
                let mut continuation = false;
                loop {
                    match self.current() {
                        Some(' ') => {
                            indent += 1;
                            self.index += 1;
                            self.column += 1;
                        }
                        Some('\t') => {
                            indent += 4;
                            self.index += 1;
                            self.column += 1;
                        }
                        Some('\r') => {
                            self.index += 1;
                            self.column += 1;
                        }
                        Some('\\') => {
                            continuation = true;
                            self.index += 1;
                            self.column += 1;
                        }
                        _ => break,
                    }
                }
                // compare indentation
                let indent_stack_clone = self.indent_stack.clone();
                let (prev_indent, prev_continuation) = indent_stack_clone.last().unwrap();
                if indent > *prev_indent {
                    self.indent_stack.push((indent, continuation));
                    Ok(Token::new(
                        TokenKind::Indent,
                        Rc::new("".to_string()),
                        self.construct_span(1),
                    ))
                } else if indent < *prev_indent {
                    self.indent_stack.pop();
                    if let Some((prev_indent, _)) = self.indent_stack.last() {
                        if indent < *prev_indent {
                            return Err(
                                self.error("inconsistent indentation", self.construct_span(1))
                            );
                        }
                    }
                    Ok(Token::new(
                        TokenKind::Dedent,
                        Rc::new("".to_string()),
                        self.construct_span(1),
                    ))
                } else {
                    if continuation && !*prev_continuation {
                        return Err(self.error("inconsistent continuation", self.construct_span(1)));
                    }
                    Ok(Token::new(
                        TokenKind::Linefeed,
                        Rc::new("".to_string()),
                        self.construct_span(1),
                    ))
                }
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                self.index += 1;
                self.column += 1;
                let mut value = c.to_string();
                loop {
                    match self.current() {
                        Some('a'..='z') | Some('A'..='Z') | Some('0'..='9') | Some('_') => {
                            value.push(self.current().unwrap());
                            self.index += 1;
                            self.column += 1;
                        }
                        _ => break,
                    }
                }
                let kind = match value.as_str() {
                    "as" => TokenKind::As,
                    "const" => TokenKind::Const,
                    "fn" => TokenKind::Fn,
                    "for" => TokenKind::For,
                    "in" => TokenKind::In,
                    "import" => TokenKind::Import,
                    "mut" => TokenKind::Mut,
                    "return" => TokenKind::Return,
                    "struct" => TokenKind::Struct,
                    "var" => TokenKind::Var,
                    "float" => TokenKind::Float,
                    "int" => TokenKind::Int,
                    _ => TokenKind::Identifier,
                };
                Ok(Token::new(
                    kind,
                    Rc::new(value.clone()),
                    self.construct_span(value.len()),
                ))
            }
            '0'..='9' => {
                let mut value = c.to_string();
                self.index += 1;
                self.column += 1;
                loop {
                    match self.current() {
                        Some('0'..='9') => {
                            value.push(self.current().unwrap());
                            self.index += 1;
                            self.column += 1;
                        }
                        Some('.') => {
                            value.push(self.current().unwrap());
                            self.index += 1;
                            self.column += 1;
                            loop {
                                match self.current() {
                                    Some('0'..='9') => {
                                        value.push(self.current().unwrap());
                                        self.index += 1;
                                        self.column += 1;
                                    }
                                    _ => break,
                                }
                            }
                            break;
                        }
                        Some('e') | Some('E') => {
                            value.push(self.current().unwrap());
                            self.index += 1;
                            self.column += 1;
                            match self.current() {
                                Some('+') | Some('-') => {
                                    value.push(self.current().unwrap());
                                    self.index += 1;
                                    self.column += 1;
                                }
                                _ => {}
                            }
                            loop {
                                match self.current() {
                                    Some('0'..='9') => {
                                        value.push(self.current().unwrap());
                                        self.index += 1;
                                        self.column += 1;
                                    }
                                    _ => break,
                                }
                            }
                            break;
                        }
                        _ => break,
                    }
                }
                let kind = if value.contains('.') || value.contains('e') || value.contains('E') {
                    TokenKind::Floating
                } else {
                    TokenKind::Integer
                };
                Ok(Token::new(
                    kind,
                    Rc::new(value.clone()),
                    self.construct_span(value.len()),
                ))
            }
            '"' => {
                self.index += 1;
                self.column += 1;
                let mut value = String::new();
                loop {
                    match self.current() {
                        Some('"') => {
                            self.index += 1;
                            self.column += 1;
                            break;
                        }
                        Some('\\') => {
                            self.index += 1;
                            self.column += 1;
                            match self.current() {
                                Some('n') => {
                                    value.push('\n');
                                    self.index += 1;
                                    self.column += 1;
                                }
                                Some('r') => {
                                    value.push('\r');
                                    self.index += 1;
                                    self.column += 1;
                                }
                                Some('t') => {
                                    value.push('\t');
                                    self.index += 1;
                                    self.column += 1;
                                }
                                Some('\\') => {
                                    value.push('\\');
                                    self.index += 1;
                                    self.column += 1;
                                }
                                Some('"') => {
                                    value.push('"');
                                    self.index += 1;
                                    self.column += 1;
                                }
                                _ => {
                                    return Err(self
                                        .error("illegal escape sequence", self.construct_span(1)))
                                }
                            }
                        }
                        Some(c) => {
                            value.push(c);
                            self.index += 1;
                            self.column += 1;
                        }
                        None => {
                            return Err(self.error("unexpected end of file", self.construct_span(1)))
                        }
                    }
                }
                Ok(Token::new(
                    TokenKind::String,
                    Rc::new(value.clone()),
                    self.construct_span(value.len() + 2),
                ))
            }
            // punctuation
            '(' => self.single_token(TokenKind::LeftParenthesis),
            ')' => self.single_token(TokenKind::RightParenthesis),
            '{' => self.single_token(TokenKind::LeftBrace),
            '}' => self.single_token(TokenKind::RightBrace),
            '[' => self.single_token(TokenKind::LeftBracket),
            ']' => self.single_token(TokenKind::RightBracket),
            ',' => self.single_token(TokenKind::Comma),
            '.' => self.single_token(TokenKind::Dot),
            ':' => self.single_token(TokenKind::Colon),
            ';' => Err(self.error(
                "semicolon isn't used as a statement terminator",
                self.construct_span(1),
            )),
            // operators
            '+' => self.double_token('+', TokenKind::Plus, '=', TokenKind::PlusEquals),
            '-' => {
                if self.contents.chars().nth(self.index + 1) == Some('>') {
                    self.index += 1;
                    self.column += 1;
                    self.double_token('-', TokenKind::ThinArrow, '>', TokenKind::ThinArrow)
                } else {
                    self.double_token('-', TokenKind::Minus, '=', TokenKind::MinusEquals)
                }
            }
            '*' => self.double_token('*', TokenKind::Asterisk, '=', TokenKind::AsteriskEquals),
            '/' => {
                if self.contents.chars().nth(self.index + 1) == Some('/') {
                    self.index += 1;
                    self.column += 1;
                    loop {
                        match self.current() {
                            Some('\n') => {
                                self.index += 1;
                                self.line += 1;
                                self.column = 1;
                                break;
                            }
                            Some(_) => {
                                self.index += 1;
                                self.column += 1;
                            }
                            None => break,
                        }
                    }
                    self.next_token()
                } else {
                    self.double_token('/', TokenKind::Slash, '=', TokenKind::SlashEquals)
                }
            }
            '%' => self.double_token('%', TokenKind::Percent, '=', TokenKind::PercentEquals),
            '=' => self.double_token('=', TokenKind::Equals, '=', TokenKind::EqualsEquals),
            '!' => self.double_token('!', TokenKind::Bang, '=', TokenKind::BangEquals),
            '<' => self.double_token('<', TokenKind::LessThan, '=', TokenKind::LessThanEquals),
            '>' => self.double_token(
                '>',
                TokenKind::GreaterThan,
                '=',
                TokenKind::GreaterThanEquals,
            ),
            '&' => self.double_token('&', TokenKind::BitwiseAnd, '&', TokenKind::And),
            '|' => self.double_token('|', TokenKind::BitwiseOr, '|', TokenKind::Or),
            '^' => self.single_token(TokenKind::BitwiseXor),
            '~' => self.single_token(TokenKind::BitwiseNot),
            _ => Err(self.error(
                format!("illegal character '{}'", c).as_str(),
                self.construct_span(1),
            )),
        }
    }
}
