use crate::{
    ast::{Block, Expression, Statement, Type, Variable},
    error::{Error, Result},
    span::spanned,
    tokenizer::{Token, TokenKind},
};

#[derive(Debug, Clone)]
pub(crate) struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub(crate) fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub(crate) fn parse(&mut self) -> Result<Vec<Statement>> {
        let mut statements = vec![];
        while !self.is_at_end() {
            if self.check(TokenKind::Linefeed) {
                self.advance();
                continue;
            }
            statements.push(self.statement()?);
        }
        Ok(statements)
    }

    fn statement(&mut self) -> Result<Statement> {
        match self.current().kind {
            TokenKind::Import => self.import(),
            TokenKind::Struct => self.struct_(),
            TokenKind::Fn => self.function(),
            _ => {
                let expr = self.expression()?;
                Ok(Statement::Expression(expr))
            }
        }
    }

    fn import(&mut self) -> Result<Statement> {
        self.consume(TokenKind::Import)?;
        let name = self.consume(TokenKind::Identifier)?.clone();
        let mut path = vec![name.lexeme.to_string().clone()];
        while self.check(TokenKind::Slash) {
            self.consume(TokenKind::Slash)?;
            let name = self.consume(TokenKind::Identifier)?.clone();
            path.push(name.lexeme.to_string().clone());
        }
        let alias = if self.check(TokenKind::As) {
            self.consume(TokenKind::As)?;
            let id = self.consume(TokenKind::Identifier)?;
            Some(spanned(id.lexeme.to_string().clone(), id.span.clone()))
        } else {
            None
        };
        Ok(Statement::Import(
            spanned(path.join("/"), name.span.clone()),
            alias,
        ))
    }

    fn struct_(&mut self) -> Result<Statement> {
        self.consume(TokenKind::Struct)?;
        let name = self.consume(TokenKind::Identifier)?.clone();
        let block = self.block(|parser| parser.struct_field())?;
        Ok(Statement::Struct(
            spanned(name.lexeme.to_string().clone(), name.span.clone()),
            block,
        ))
    }

    fn struct_field(&mut self) -> Result<Variable> {
        let name = self.consume(TokenKind::Identifier)?.clone();
        self.consume(TokenKind::Colon)?;
        let ty_span = self.current().span.clone();
        let ty = self.type_()?;
        Ok(Variable {
            name: spanned(name.lexeme.to_string().clone(), name.span.clone()),
            ty: spanned(ty, ty_span),
            initializer: None,
        })
    }

    fn function(&mut self) -> Result<Statement> {
        self.consume(TokenKind::Fn)?;
        let name = self.consume(TokenKind::Identifier)?.clone();
        self.consume(TokenKind::LeftParenthesis)?;
        let mut params = vec![];
        while !self.check(TokenKind::RightParenthesis) {
            let name = self.consume(TokenKind::Identifier)?.clone();
            self.consume(TokenKind::Colon)?;
            let ty_span = self.current().span.clone();
            let ty = self.type_()?;
            params.push(Variable {
                name: spanned(name.lexeme.to_string().clone(), name.span.clone()),
                ty: spanned(ty, ty_span),
                initializer: None,
            });
            if self.check(TokenKind::Comma) {
                self.consume(TokenKind::Comma)?;
            }
        }
        self.consume(TokenKind::RightParenthesis)?;
        let (ty, ty_span) = if self.check(TokenKind::ThinArrow) {
            self.consume(TokenKind::ThinArrow)?;
            let ty_span = self.current().span.clone();
            let ty = self.type_()?;
            (ty, ty_span)
        } else {
            (Type::Unit, self.current().span.clone())
        };
        let block = self.block(|parser| parser.statement())?;
        Ok(Statement::Function(
            spanned(name.lexeme.to_string().clone(), name.span.clone()),
            params,
            spanned(ty, ty_span),
            block,
        ))
    }

    fn expression(&mut self) -> Result<Expression> {
        let expr = self.primary()?;
        if self.check(TokenKind::LeftParenthesis) {
            self.consume(TokenKind::LeftParenthesis)?;
            let mut args = vec![];
            while !self.check(TokenKind::RightParenthesis) {
                args.push(spanned(self.expression()?, expr.span().clone()));
                if self.check(TokenKind::Comma) {
                    self.consume(TokenKind::Comma)?;
                }
            }
            self.consume(TokenKind::RightParenthesis)?;
            Ok(Expression::Call(
                spanned(Box::new(expr.clone()), expr.span().clone()),
                args,
            ))
        } else if self.check(TokenKind::Dot) {
            self.consume(TokenKind::Dot)?;
            Ok(Expression::Access(
                spanned(Box::new(expr.clone()), expr.span().clone()),
                spanned(Box::new(self.expression()?), expr.span().clone()),
            ))
        } else {
            Ok(expr)
        }
    }

    fn primary(&mut self) -> Result<Expression> {
        let token = self.advance();
        match token.kind {
            TokenKind::Identifier => Ok(Expression::Identifier(spanned(
                token.lexeme.to_string().clone(),
                token.span.clone(),
            ))),
            TokenKind::LeftParenthesis => {
                let expr = self.expression()?;
                self.consume(TokenKind::RightParenthesis)?;
                Ok(expr)
            }
            _ => Err(self.error(&token, "Expecting expression")),
        }
    }

    fn type_(&mut self) -> Result<Type> {
        let ty = match self.current().kind {
            TokenKind::Int => {
                self.consume(TokenKind::Int)?;
                Type::Int
            }
            TokenKind::Float => {
                self.consume(TokenKind::Float)?;
                Type::Float
            }
            TokenKind::Identifier => {
                let id = self.consume(TokenKind::Identifier)?.clone();
                if self.check(TokenKind::LeftBracket) {
                    self.consume(TokenKind::LeftBracket)?;
                    let mut tys = vec![];
                    while !self.check(TokenKind::RightBracket) {
                        tys.push(self.type_()?);
                        if self.check(TokenKind::Comma) {
                            self.consume(TokenKind::Comma)?;
                        }
                    }
                    self.consume(TokenKind::RightBracket)?;
                    Type::Polymorphic(
                        id.lexeme.to_string().clone(),
                        tys.into_iter()
                            .map(|ty| spanned(ty, id.span.clone()))
                            .collect(),
                    )
                } else {
                    Type::Id(id.lexeme.to_string().clone())
                }
            }
            TokenKind::BitwiseAnd => {
                self.consume(TokenKind::BitwiseAnd)?;
                if self.check(TokenKind::Mut) {
                    self.consume(TokenKind::Mut)?;
                    let ty = self.type_()?;
                    Type::MutableReference(Box::new(ty))
                } else {
                    let ty = self.type_()?;
                    Type::Reference(Box::new(ty))
                }
            }
            _ => unreachable!(),
        };
        Ok(ty)
    }

    fn block<T>(&mut self, _parse: impl Fn(&mut Self) -> Result<T>) -> Result<Block<T>> {
        let mut ts = vec![];
        self.consume(TokenKind::Colon)?;
        self.consume(TokenKind::Indent)?;
        while !self.check(TokenKind::Dedent) && !self.is_at_end() {
            ts.push(_parse(self)?);
            if self.check(TokenKind::Linefeed) {
                self.advance();
            }
        }
        self.consume(TokenKind::Dedent)?;
        Ok(Block { ts })
    }

    fn is_at_end(&self) -> bool {
        self.current().kind == TokenKind::Eof
    }

    fn current(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn advance(&mut self) -> Token {
        let token = self.current().clone();
        if !self.is_at_end() {
            self.current += 1;
        }
        token
    }

    fn check(&self, kind: TokenKind) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.current().kind == kind
        }
    }

    fn consume(&mut self, kind: TokenKind) -> Result<Token> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            Err(self.error(self.current(), "Expecting token"))
        }
    }

    fn error(&self, token: &Token, message: &str) -> crate::error::Error {
        Error::new(message.to_string(), token.span.clone())
    }
}
