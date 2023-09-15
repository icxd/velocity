use crate::span::{Span, Spanned};

pub(crate) type SpannedString = Spanned<String>;

#[derive(Debug, Clone)]
pub(crate) enum Statement {
    Block(Vec<Statement>),
    Import(SpannedString),
    Struct {
        spanned_id: SpannedString,
        generic_parameters: Vec<SpannedString>,
        fields: Vec<(SpannedString, Type)>,
    },
    Enum {
        spanned_id: SpannedString,
        generic_parameters: Vec<SpannedString>,
        base_type: Option<Type>,
        variants: Vec<(SpannedString, Option<Expression>)>,
    },
    Union {
        spanned_id: SpannedString,
        generic_parameters: Vec<SpannedString>,
        tagged: bool,
        variants: Vec<(Option<SpannedString>, Type)>,
    },
    Function(
        SpannedString,
        Vec<(SpannedString, Type)>,
        Option<Type>,
        Box<Statement>,
    ),
    Variable(SpannedString, Option<Type>, Expression),
    Constant(SpannedString, Option<Type>, Expression),
    For(SpannedString, Expression, Box<Statement>),
    Return(Option<Expression>),
    Expression(Expression),
    Garbage(Span),
}

#[derive(Debug, Clone)]
pub(crate) enum Expression {
    Identifier(SpannedString),
    String(SpannedString),
    Character(Spanned<char>),
    Integer(Spanned<i64>),
    FloatingPoint(Spanned<f64>),
    Boolean(Spanned<bool>),
    Binary(Box<Expression>, BinaryOperator, Box<Expression>),
    Unary(UnaryOperator, Box<Expression>),
    Call(Box<Expression>, Vec<Expression>),
    Index(Box<Expression>, Box<Expression>),
    Member(Box<Expression>, SpannedString),
    Assignment(Box<Expression>, Box<Expression>),
    StructLiteral(Box<Expression>, Vec<(Option<SpannedString>, Expression)>),
    ArrayLiteral(Vec<Expression>),
}

impl Expression {
    pub(crate) fn span(&self) -> Span {
        match self {
            Expression::Identifier(spanned) => spanned.1.clone(),
            Expression::String(spanned) => spanned.1.clone(),
            Expression::Character(spanned) => spanned.1.clone(),
            Expression::Integer(spanned) => spanned.1.clone(),
            Expression::FloatingPoint(spanned) => spanned.1.clone(),
            Expression::Boolean(spanned) => spanned.1.clone(),
            Expression::Binary(lhs, _, _) => lhs.span(),
            Expression::Unary(_, expr) => expr.span(),
            Expression::Call(expr, _) => expr.span(),
            Expression::Index(expr, _) => expr.span(),
            Expression::Member(expr, _) => expr.span(),
            Expression::Assignment(lhs, _) => lhs.span(),
            Expression::StructLiteral(expr, _) => expr.span(),
            Expression::ArrayLiteral(_) => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Type {
    Integer,
    UnsignedInteger,
    FloatingPoint,
    Boolean,
    Character,
    Reference(Box<Type>),                  // &T
    MutableReference(Box<Type>),           // &mut T
    Identifier(SpannedString),             // T
    Polymorphic(SpannedString, Vec<Type>), // T[T, ...]
    Member(Box<Type>, SpannedString),      // T.T
    Function(Vec<Type>, Box<Type>),        // fn(T, ...) -> T
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum BinaryOperator {
    Plus,
    Minus,
    Asterisk,
    Slash,
    Percent,
    EqualsEquals,
    BangEquals,
    LessThan,
    LessThanEquals,
    GreaterThan,
    GreaterThanEquals,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum UnaryOperator {
    Plus,
    Minus,
    Bang,
    AddressOf,
    MutableAddressOf,
}
