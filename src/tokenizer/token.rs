use crate::span::Span;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TokenKind {
    // Literals
    Identifier,
    String,
    Character,
    Integer,
    FloatingPoint,
    Boolean,
    // Keywords
    Import,
    Struct,
    Fn,
    Mut,
    Var,
    Const,
    For,
    In,
    Return,
    And,
    Or,
    // Types
    Int,
    Float,
    Bool,
    Char,
    // Operators
    Plus,
    PlusEquals,
    Minus,
    MinusEquals,
    Asterisk,
    AsteriskEquals,
    Slash,
    SlashEquals,
    Percent,
    PercentEquals,
    Equals,
    EqualsEquals,
    Bang,
    BangEquals,
    LessThan,
    LessThanEquals,
    GreaterThan,
    GreaterThanEquals,
    // Punctuation
    OpenParenthesis,
    CloseParenthesis,
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,
    Comma,
    Dot,
    Colon,
    Semicolon,
    Ampersand,
    Pipe,
    Arrow,
    // Misc
    Eof,
}

impl TokenKind {
    pub(crate) fn is_unary_operator(&self) -> bool {
        match self {
            TokenKind::Plus | TokenKind::Minus | TokenKind::Bang | TokenKind::Ampersand => true,
            _ => false,
        }
    }

    pub(crate) fn is_binary_operator(&self) -> bool {
        match self {
            TokenKind::Plus
            | TokenKind::Minus
            | TokenKind::Asterisk
            | TokenKind::Slash
            | TokenKind::Percent
            | TokenKind::EqualsEquals
            | TokenKind::BangEquals
            | TokenKind::LessThan
            | TokenKind::LessThanEquals
            | TokenKind::GreaterThan
            | TokenKind::GreaterThanEquals
            | TokenKind::And
            | TokenKind::Or => true,
            _ => false,
        }
    }
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TokenKind::Identifier => "<identifier>",
                TokenKind::String => "<string>",
                TokenKind::Character => "<character>",
                TokenKind::Integer => "<integer>",
                TokenKind::FloatingPoint => "<floating point>",
                TokenKind::Boolean => "<boolean>",
                TokenKind::Import => "import",
                TokenKind::Struct => "struct",
                TokenKind::Fn => "fn",
                TokenKind::Mut => "mut",
                TokenKind::Var => "var",
                TokenKind::Const => "const",
                TokenKind::For => "for",
                TokenKind::In => "in",
                TokenKind::Return => "return",
                TokenKind::And => "and",
                TokenKind::Or => "or",
                TokenKind::Int => "int",
                TokenKind::Float => "float",
                TokenKind::Bool => "bool",
                TokenKind::Char => "char",
                TokenKind::Plus => "+",
                TokenKind::PlusEquals => "+=",
                TokenKind::Minus => "-",
                TokenKind::MinusEquals => "-=",
                TokenKind::Asterisk => "*",
                TokenKind::AsteriskEquals => "*=",
                TokenKind::Slash => "/",
                TokenKind::SlashEquals => "/=",
                TokenKind::Percent => "%",
                TokenKind::PercentEquals => "%=",
                TokenKind::Equals => "=",
                TokenKind::EqualsEquals => "==",
                TokenKind::Bang => "!",
                TokenKind::BangEquals => "!=",
                TokenKind::LessThan => "<",
                TokenKind::LessThanEquals => "<=",
                TokenKind::GreaterThan => ">",
                TokenKind::GreaterThanEquals => ">=",
                TokenKind::OpenParenthesis => "(",
                TokenKind::CloseParenthesis => ")",
                TokenKind::OpenBrace => "{",
                TokenKind::CloseBrace => "}",
                TokenKind::OpenBracket => "[",
                TokenKind::CloseBracket => "]",
                TokenKind::Comma => ",",
                TokenKind::Dot => ".",
                TokenKind::Colon => ":",
                TokenKind::Semicolon => ";",
                TokenKind::Ampersand => "&",
                TokenKind::Pipe => "|",
                TokenKind::Arrow => "->",
                TokenKind::Eof => "<eof>",
            }
        )
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Token {
    pub(crate) kind: TokenKind,
    pub(crate) value: String,
    pub(crate) span: Span,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?}: {}", self.kind, self.value))
    }
}
