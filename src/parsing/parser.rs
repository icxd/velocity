use std::rc::Rc;

use crate::{
    error::{Error, Result},
    span::Spanned,
    tokenizer::token::{Token, TokenKind},
};

use super::ast::{BinaryOperator, Expression, SpannedString, Statement, Type, UnaryOperator};

#[derive(Debug, Clone)]
pub(crate) struct Parser {
    filename: Rc<String>,
    pub(crate) tokens: Vec<Token>,
    pub(crate) index: usize,
}

impl Parser {
    pub(crate) fn new(filename: Rc<String>, tokens: Vec<Token>) -> Self {
        Self {
            filename,
            tokens,
            index: 0,
        }
    }

    pub(crate) fn parse(&mut self) -> std::result::Result<Vec<Statement>, Vec<Error>> {
        let mut statements: Vec<Statement> = vec![];
        let mut errors: Vec<Error> = vec![];
        while !self.is_at_end() {
            match self.parse_statement() {
                Ok(statement) => statements.push(statement),
                Err(error) => {
                    errors.push(error);
                    unsafe { self.advance().unwrap_unchecked() };
                }
            }
        }
        if errors.is_empty() {
            Ok(statements)
        } else {
            Err(errors)
        }
    }

    fn parse_statement(&mut self) -> Result<Statement> {
        match self.current()?.kind {
            TokenKind::OpenBrace => self.parse_block(),
            TokenKind::Import => self.parse_import(),
            TokenKind::Struct => self.parse_struct(),
            TokenKind::Fn => self.parse_function(),
            TokenKind::Var => self.parse_variable(),
            TokenKind::Const => self.parse_constant(),
            TokenKind::For => self.parse_for(),
            TokenKind::Return => self.parse_return(),
            TokenKind::Eof => {
                self.advance()?;
                Ok(Statement::Garbage)
            }
            _ => {
                let expression = self.parse_expression()?;
                self.consume(TokenKind::Semicolon)?;
                Ok(Statement::Expression(expression))
            }
        }
    }

    fn parse_block(&mut self) -> Result<Statement> {
        let mut statements: Vec<Statement> = vec![];
        self.consume(TokenKind::OpenBrace)?;
        while !self.is_at_end() && !self.check(TokenKind::CloseBrace) {
            if self.current()?.kind == TokenKind::Semicolon {
                self.advance()?;
                continue;
            }
            statements.push(self.parse_statement()?);
        }
        self.consume(TokenKind::CloseBrace)?;
        Ok(Statement::Block(statements))
    }

    fn parse_import(&mut self) -> Result<Statement> {
        self.consume(TokenKind::Import)?;
        let identifier = self.consume(TokenKind::Identifier)?;
        let spanned_id: SpannedString = (identifier.value.clone(), identifier.span.clone());
        Ok(Statement::Import(spanned_id))
    }

    fn parse_struct(&mut self) -> Result<Statement> {
        self.consume(TokenKind::Struct)?;
        let identifier = self.consume(TokenKind::Identifier)?;
        let spanned_id: SpannedString = (identifier.value.clone(), identifier.span.clone());
        self.consume(TokenKind::OpenBrace)?;
        let mut members: Vec<(SpannedString, Type)> = vec![];
        while !self.check(TokenKind::CloseBrace) {
            let identifier = self.consume(TokenKind::Identifier)?;
            let spanned_id: SpannedString = (identifier.value.clone(), identifier.span.clone());
            self.consume(TokenKind::Colon)?;
            let type_ = self.parse_type()?;
            members.push((spanned_id, type_));
            if !self.check(TokenKind::CloseBrace) {
                self.consume(TokenKind::Comma)?;
            }
        }
        self.consume(TokenKind::CloseBrace)?;
        Ok(Statement::Struct(spanned_id, members))
    }

    fn parse_function(&mut self) -> Result<Statement> {
        self.consume(TokenKind::Fn)?;
        let identifier = self.consume(TokenKind::Identifier)?;
        let spanned_id: SpannedString = (identifier.value.clone(), identifier.span.clone());
        self.consume(TokenKind::OpenParenthesis)?;
        let mut parameters: Vec<(SpannedString, Type)> = vec![];
        while !self.check(TokenKind::CloseParenthesis) {
            let identifier = self.consume(TokenKind::Identifier)?;
            let spanned_id: SpannedString = (identifier.value.clone(), identifier.span.clone());
            self.consume(TokenKind::Colon)?;
            let type_ = self.parse_type()?;
            parameters.push((spanned_id, type_));
            if !self.check(TokenKind::CloseParenthesis) {
                self.consume(TokenKind::Comma)?;
            }
        }
        self.consume(TokenKind::CloseParenthesis)?;
        let return_type = if self.check(TokenKind::Arrow) {
            self.consume(TokenKind::Arrow)?;
            Some(self.parse_type()?)
        } else {
            None
        };
        let body = Box::new(self.parse_statement()?);
        Ok(Statement::Function(
            spanned_id,
            parameters,
            return_type,
            body,
        ))
    }

    fn parse_variable(&mut self) -> Result<Statement> {
        self.consume(TokenKind::Var)?;
        let identifier = self.consume(TokenKind::Identifier)?;
        let spanned_id: SpannedString = (identifier.value.clone(), identifier.span.clone());
        let type_: Option<Type> = if self.check(TokenKind::Colon) {
            self.consume(TokenKind::Colon)?;
            Some(self.parse_type()?)
        } else {
            None
        };
        self.consume(TokenKind::Equals)?;
        let expression = self.parse_expression()?;
        self.consume(TokenKind::Semicolon)?;
        Ok(Statement::Variable(spanned_id, type_, expression))
    }

    fn parse_constant(&mut self) -> Result<Statement> {
        self.consume(TokenKind::Const)?;
        let identifier = self.consume(TokenKind::Identifier)?;
        let spanned_id: SpannedString = (identifier.value.clone(), identifier.span.clone());
        let type_: Option<Type> = if self.check(TokenKind::Colon) {
            self.consume(TokenKind::Colon)?;
            Some(self.parse_type()?)
        } else {
            None
        };
        self.consume(TokenKind::Equals)?;
        let expression = self.parse_expression()?;
        self.consume(TokenKind::Semicolon)?;
        Ok(Statement::Constant(spanned_id, type_, expression))
    }

    fn parse_for(&mut self) -> Result<Statement> {
        self.consume(TokenKind::For)?;
        self.consume(TokenKind::OpenParenthesis)?;
        let identifier = self.consume(TokenKind::Identifier)?;
        let spanned_id: SpannedString = (identifier.value.clone(), identifier.span.clone());
        self.consume(TokenKind::In)?;
        let expression = self.parse_expression()?;
        self.consume(TokenKind::CloseParenthesis)?;
        let body = Box::new(self.parse_statement()?);
        Ok(Statement::For(spanned_id, expression, body))
    }

    fn parse_return(&mut self) -> Result<Statement> {
        self.consume(TokenKind::Return)?;
        let expression = if !self.check(TokenKind::Semicolon) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        Ok(Statement::Return(expression))
    }

    fn parse_expression(&mut self) -> Result<Expression> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Expression> {
        let expression = self.parse_or()?;
        if self.check(TokenKind::Equals) {
            self.advance()?;
            let value = self.parse_assignment()?;
            Ok(Expression::Assignment(
                Box::new(expression),
                Box::new(value),
            ))
        } else {
            Ok(expression)
        }
    }

    fn parse_or(&mut self) -> Result<Expression> {
        let mut expression = self.parse_and()?;
        while self.check(TokenKind::Or) {
            self.advance()?;
            let right = self.parse_expression()?;
            expression =
                Expression::Binary(Box::new(expression), BinaryOperator::Or, Box::new(right));
        }
        Ok(expression)
    }

    fn parse_and(&mut self) -> Result<Expression> {
        let mut expression = self.parse_equality()?;
        while self.check(TokenKind::And) {
            self.advance()?;
            let right = self.parse_expression()?;
            expression =
                Expression::Binary(Box::new(expression), BinaryOperator::And, Box::new(right));
        }
        Ok(expression)
    }

    fn parse_equality(&mut self) -> Result<Expression> {
        let mut expression = self.parse_comparison()?;
        while self.check(TokenKind::EqualsEquals) || self.check(TokenKind::BangEquals) {
            let operator = match self.current()?.kind {
                TokenKind::EqualsEquals => BinaryOperator::EqualsEquals,
                TokenKind::BangEquals => BinaryOperator::BangEquals,
                _ => unreachable!(),
            };
            self.advance()?;
            let right = self.parse_expression()?;
            expression = Expression::Binary(Box::new(expression), operator, Box::new(right));
        }
        Ok(expression)
    }

    fn parse_comparison(&mut self) -> Result<Expression> {
        let mut expression = self.parse_term()?;
        while self.check(TokenKind::LessThan)
            || self.check(TokenKind::LessThanEquals)
            || self.check(TokenKind::GreaterThan)
            || self.check(TokenKind::GreaterThanEquals)
        {
            let operator = match self.current()?.kind {
                TokenKind::LessThan => BinaryOperator::LessThan,
                TokenKind::LessThanEquals => BinaryOperator::LessThanEquals,
                TokenKind::GreaterThan => BinaryOperator::GreaterThan,
                TokenKind::GreaterThanEquals => BinaryOperator::GreaterThanEquals,
                _ => unreachable!(),
            };
            self.advance()?;
            let right = self.parse_expression()?;
            expression = Expression::Binary(Box::new(expression), operator, Box::new(right));
        }
        Ok(expression)
    }

    fn parse_term(&mut self) -> Result<Expression> {
        let mut expression = self.parse_factor()?;
        while self.check(TokenKind::Plus) || self.check(TokenKind::Minus) {
            let operator = match self.current()?.kind {
                TokenKind::Plus => BinaryOperator::Plus,
                TokenKind::Minus => BinaryOperator::Minus,
                _ => unreachable!(),
            };
            self.advance()?;
            let right = self.parse_expression()?;
            expression = Expression::Binary(Box::new(expression), operator, Box::new(right));
        }
        Ok(expression)
    }

    fn parse_factor(&mut self) -> Result<Expression> {
        let mut expression = self.parse_unary()?;
        while self.check(TokenKind::Asterisk)
            || self.check(TokenKind::Slash)
            || self.check(TokenKind::Percent)
        {
            let operator = match self.current()?.kind {
                TokenKind::Asterisk => BinaryOperator::Asterisk,
                TokenKind::Slash => BinaryOperator::Slash,
                TokenKind::Percent => BinaryOperator::Percent,
                _ => unreachable!(),
            };
            self.advance()?;
            let right = self.parse_expression()?;
            expression = Expression::Binary(Box::new(expression), operator, Box::new(right));
        }
        Ok(expression)
    }

    fn parse_unary(&mut self) -> Result<Expression> {
        if self.current()?.kind.is_unary_operator() {
            let operator = match self.current()?.kind {
                TokenKind::Plus => UnaryOperator::Plus,
                TokenKind::Minus => UnaryOperator::Minus,
                TokenKind::Bang => UnaryOperator::Bang,
                TokenKind::Ampersand => {
                    self.advance()?;
                    if self.check(TokenKind::Mut) {
                        UnaryOperator::MutableAddressOf
                    } else {
                        UnaryOperator::AddressOf
                    }
                }
                _ => unreachable!(),
            };
            self.advance()?;
            let expression = self.parse_unary()?;
            Ok(Expression::Unary(operator, Box::new(expression)))
        } else {
            self.parse_call()
        }
    }

    fn parse_call(&mut self) -> Result<Expression> {
        let mut expression = self.parse_primary()?;
        while self.check(TokenKind::OpenParenthesis)
            || self.check(TokenKind::OpenBracket)
            || self.check(TokenKind::OpenBrace)
            || self.check(TokenKind::Dot)
        {
            if self.check(TokenKind::OpenParenthesis) {
                self.advance()?;
                let mut arguments: Vec<Expression> = vec![];
                while !self.check(TokenKind::CloseParenthesis) {
                    arguments.push(self.parse_expression()?);
                    if !self.check(TokenKind::CloseParenthesis) {
                        self.consume(TokenKind::Comma)?;
                    }
                }
                self.consume(TokenKind::CloseParenthesis)?;
                expression = Expression::Call(Box::new(expression), arguments);
            } else if self.check(TokenKind::OpenBracket) {
                self.advance()?;
                let index = self.parse_expression()?;
                self.consume(TokenKind::CloseBracket)?;
                expression = Expression::Index(Box::new(expression), Box::new(index));
            } else if self.check(TokenKind::OpenBrace) {
                self.advance()?;
                let mut members: Vec<(Option<SpannedString>, Expression)> = vec![];
                while !self.check(TokenKind::CloseBrace) {
                    let identifier = if self.check(TokenKind::Identifier) {
                        let identifier = self.consume(TokenKind::Identifier)?;
                        let spanned_id: SpannedString =
                            (identifier.value.clone(), identifier.span.clone());
                        Some(spanned_id)
                    } else {
                        None
                    };
                    self.consume(TokenKind::Equals)?;
                    let expression = self.parse_expression()?;
                    members.push((identifier, expression));
                    if !self.check(TokenKind::CloseBrace) {
                        self.consume(TokenKind::Comma)?;
                    }
                }
                self.consume(TokenKind::CloseBrace)?;
                expression = Expression::StructLiteral(Box::new(expression), members);
            } else if self.check(TokenKind::Dot) {
                self.advance()?;
                let identifier = self.consume(TokenKind::Identifier)?;
                let spanned_id: SpannedString = (identifier.value.clone(), identifier.span.clone());
                expression = Expression::Member(Box::new(expression), spanned_id);
            }
        }
        Ok(expression)
    }

    fn parse_primary(&mut self) -> Result<Expression> {
        if self.check(TokenKind::Identifier) {
            let identifier = self.consume(TokenKind::Identifier)?;
            let spanned_id: SpannedString = (identifier.value.clone(), identifier.span.clone());
            Ok(Expression::Identifier(spanned_id))
        } else if self.check(TokenKind::String) {
            let string = self.consume(TokenKind::String)?;
            let spanned_string: SpannedString = (string.value.clone(), string.span.clone());
            Ok(Expression::String(spanned_string))
        } else if self.check(TokenKind::Character) {
            let character = self.consume(TokenKind::Character)?;
            let c = match character.value.chars().next() {
                Some(c) => c,
                None => unreachable!(),
            };
            let spanned_character: Spanned<char> = (c, character.span.clone());
            Ok(Expression::Character(spanned_character))
        } else if self.check(TokenKind::Integer) {
            let integer = self.consume(TokenKind::Integer)?;
            let value = integer.value.parse::<i64>().unwrap();
            let spanned_integer: Spanned<i64> = (value, integer.span.clone());
            Ok(Expression::Integer(spanned_integer))
        } else if self.check(TokenKind::FloatingPoint) {
            let float = self.consume(TokenKind::FloatingPoint)?;
            let value = float.value.parse::<f64>().unwrap();
            let spanned_float: Spanned<f64> = (value, float.span.clone());
            Ok(Expression::FloatingPoint(spanned_float))
        } else if self.check(TokenKind::Boolean) {
            let boolean = self.consume(TokenKind::Boolean)?;
            let value = match boolean.value.as_str() {
                "true" => true,
                "false" => false,
                _ => unreachable!(),
            };
            let spanned_boolean: Spanned<bool> = (value, boolean.span.clone());
            Ok(Expression::Boolean(spanned_boolean))
        } else if self.check(TokenKind::OpenBracket) {
            self.advance()?;
            let mut elements: Vec<Expression> = vec![];
            while !self.check(TokenKind::CloseBracket) {
                elements.push(self.parse_expression()?);
                if !self.check(TokenKind::CloseBracket) {
                    self.consume(TokenKind::Comma)?;
                }
            }
            self.consume(TokenKind::CloseBracket)?;
            Ok(Expression::ArrayLiteral(elements))
        } else {
            Err(Error::new(
                format!("expected an expression, but got {}", self.current()?.kind),
                self.current()?.span.clone(),
            ))
        }
    }

    fn parse_type(&mut self) -> Result<Type> {
        let mut type_ = self.parse_reference()?;
        while self.check(TokenKind::Dot) {
            self.advance()?;
            let identifier = self.consume(TokenKind::Identifier)?;
            let spanned_id: SpannedString = (identifier.value.clone(), identifier.span.clone());
            type_ = Type::Member(Box::new(type_), spanned_id);
        }
        Ok(type_)
    }

    fn parse_reference(&mut self) -> Result<Type> {
        if self.check(TokenKind::Ampersand) {
            self.advance()?;
            if self.check(TokenKind::Mut) {
                self.advance()?;
                let type_ = self.parse_reference()?;
                Ok(Type::MutableReference(Box::new(type_)))
            } else {
                let type_ = self.parse_reference()?;
                Ok(Type::Reference(Box::new(type_)))
            }
        } else if self.check(TokenKind::Identifier) {
            let identifier = self.consume(TokenKind::Identifier)?;
            let spanned_id: SpannedString = (identifier.value.clone(), identifier.span.clone());
            if self.check(TokenKind::OpenBracket) {
                self.advance()?;
                let mut parameters: Vec<Type> = vec![];
                while !self.check(TokenKind::CloseBracket) {
                    parameters.push(self.parse_type()?);
                    if !self.check(TokenKind::CloseBracket) {
                        self.consume(TokenKind::Comma)?;
                    }
                }
                self.consume(TokenKind::CloseBracket)?;
                Ok(Type::Polymorphic(spanned_id, parameters))
            } else {
                Ok(Type::Identifier(spanned_id))
            }
        } else if self.check(TokenKind::Fn) {
            self.advance()?;
            self.consume(TokenKind::OpenParenthesis)?;
            let mut parameters: Vec<Type> = vec![];
            while !self.check(TokenKind::CloseParenthesis) {
                parameters.push(self.parse_type()?);
                if !self.check(TokenKind::CloseParenthesis) {
                    self.consume(TokenKind::Comma)?;
                }
            }
            self.consume(TokenKind::CloseParenthesis)?;
            self.consume(TokenKind::Arrow)?;
            let return_type = self.parse_type()?;
            Ok(Type::Function(parameters, Box::new(return_type)))
        } else if self.check(TokenKind::Int) {
            self.advance()?;
            Ok(Type::Integer)
        } else if self.check(TokenKind::Float) {
            self.advance()?;
            Ok(Type::FloatingPoint)
        } else if self.check(TokenKind::Bool) {
            self.advance()?;
            Ok(Type::Boolean)
        } else if self.check(TokenKind::Char) {
            self.advance()?;
            Ok(Type::Character)
        } else {
            Err(Error::new(
                format!("expected a type, but got {}", self.current()?.kind),
                self.current()?.span.clone(),
            ))
        }
    }

    fn is_at_end(&self) -> bool {
        self.index >= self.tokens.len()
    }

    fn current(&self) -> Result<Token> {
        if self.is_at_end() {
            Err(Error::new(
                format!("unexpected end of file"),
                self.previous().span.clone(),
            ))
        } else {
            Ok(self.tokens[self.index].clone())
        }
    }
    fn advance(&mut self) -> Result<Token> {
        if !self.is_at_end() {
            self.index += 1;
        }
        Ok(self.previous().clone())
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.index - 1]
    }

    fn check(&self, kind: TokenKind) -> bool {
        if self.is_at_end() {
            false
        } else {
            unsafe { self.current().unwrap_unchecked().kind == kind }
        }
    }

    fn consume(&mut self, kind: TokenKind) -> Result<Token> {
        if self.check(kind.clone()) {
            let token = self.current()?;
            self.advance()?;
            Ok(token)
        } else {
            Err(Error::new(
                format!("expected '{}', but got {}", kind, self.current()?.kind),
                self.current()?.span.clone(),
            ))
        }
    }
}
