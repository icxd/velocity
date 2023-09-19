use crate::span::{Span, Spanned};

#[derive(Debug, Clone)]
pub(crate) struct Block<T> {
    pub(crate) ts: Vec<T>,
}

#[derive(Debug, Clone)]
pub(crate) enum Statement {
    Import(Spanned<String>, Option<Spanned<String>>),
    Struct(Spanned<String>, Block<Variable>),
    Function(
        Spanned<String>,
        Vec<Variable>,
        Spanned<Type>,
        Block<Statement>,
    ),
    Expression(Expression),
}
#[derive(Debug, Clone)]
pub(crate) enum Expression {
    Identifier(Spanned<String>),
    Call(Spanned<Box<Expression>>, Vec<Spanned<Expression>>),
    Access(Spanned<Box<Expression>>, Spanned<Box<Expression>>),
}

impl Expression {
    pub(crate) fn span(&self) -> Span {
        match self {
            Expression::Identifier(id) => id.1.clone(),
            Expression::Call(callee, _) => callee.1.clone(),
            Expression::Access(expr, _) => expr.1.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Type {
    Unit,
    Int,
    Float,
    Reference(Box<Type>),
    MutableReference(Box<Type>),
    Id(String),
    Polymorphic(String, Vec<Spanned<Type>>),
}

#[derive(Debug, Clone)]
pub(crate) struct Variable {
    pub(crate) name: Spanned<String>,
    pub(crate) ty: Spanned<Type>,
    pub(crate) initializer: Option<Spanned<Expression>>,
}
