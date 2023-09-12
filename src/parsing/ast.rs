use crate::span::Spanned;

pub(crate) type SpannedString = Spanned<String>;

#[derive(Debug, Clone)]
pub(crate) enum Statement {
    Block(Vec<Statement>),
    Import(SpannedString),
    Struct(SpannedString, Vec<(SpannedString, Type)>),
    Function(
        SpannedString,
        Vec<(SpannedString, Type)>,
        Option<Type>,
        Box<Statement>,
    ),
    Variable(SpannedString, Type, Expression),
    Constant(SpannedString, Type, Expression),
    For(SpannedString, Expression, Box<Statement>),
    Return(Option<Expression>),
    Expression(Expression),
}

#[derive(Debug, Clone)]
pub(crate) enum Expression {
    Identifier(SpannedString),
    String(SpannedString),
    Character(SpannedString),
    Integer(SpannedString),
    FloatingPoint(SpannedString),
    Boolean(SpannedString),
    Binary(Box<Expression>, BinaryOperator, Box<Expression>),
    Unary(UnaryOperator, Box<Expression>),
    Call(Box<Expression>, Vec<Expression>),
    Index(Box<Expression>, Box<Expression>),
    Member(Box<Expression>, SpannedString),
    Assignment(Box<Expression>, Box<Expression>),
    StructLiteral(Box<Expression>, Vec<(Option<SpannedString>, Expression)>),
    ArrayLiteral(Vec<Expression>),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Type {
    Integer,
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