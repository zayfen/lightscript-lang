//! Parser implementation for LightLang

mod ast;

use crate::lexer::{Lexer, Token};
use ast::*;

pub type ParseResult<T> = Result<T, String>;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(source: &str) -> Self {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap_or_else(|_| vec![Token::EOF]);
        Parser { tokens, current: 0 }
    }
    
    pub fn parse(&mut self) -> ParseResult<Program> {
        let mut statements = Vec::new();
        while self.current < self.tokens.len() && !self.is_at_end() {
            statements.push(self.parse_stmt()?);
        }
        Ok(Program::new(statements))
    }
    
    fn parse_stmt(&mut self) -> ParseResult<Stmt> {
        match self.peek() {
            Some(Token::Let) => self.parse_var_decl(false),
            Some(Token::Const) => self.parse_var_decl(true),
            Some(Token::Function) => self.parse_function_decl(),
            Some(Token::If) => self.parse_if_stmt(),
            Some(Token::While) => self.parse_while_stmt(),
            Some(Token::Return) => self.parse_return_stmt(),
            _ => self.parse_expr_stmt(),
        }
    }
    
    fn parse_var_decl(&mut self, is_const: bool) -> ParseResult<Stmt> {
        self.advance();
        let name = self.expect_identifier()?;
        let init = if self.match_token(&Token::Equal) {
            self.advance();
            Some(self.parse_expr()?)
        } else {
            None
        };
        self.expect(Token::Semicolon)?;
        Ok(Stmt::VariableDecl { name, init, is_const })
    }
    
    fn parse_function_decl(&mut self) -> ParseResult<Stmt> {
        self.advance();
        let name = self.expect_identifier()?;
        self.expect(Token::LeftParen)?;
        let mut params = Vec::new();
        if !self.match_token(&Token::RightParen) {
            loop {
                params.push(self.expect_identifier()?);
                if !self.match_token(&Token::Comma) {
                    break;
                }
                self.advance();
            }
        }
        self.expect(Token::RightParen)?;
        let body = self.parse_block()?;
        Ok(Stmt::FunctionDecl { name, params, body })
    }
    
    fn parse_if_stmt(&mut self) -> ParseResult<Stmt> {
        self.advance();
        self.expect(Token::LeftParen)?;
        let condition = self.parse_expr()?;
        self.expect(Token::RightParen)?;
        let then_branch = self.parse_block()?;
        let else_branch = if self.match_token(&Token::Else) {
            self.advance();
            Some(self.parse_block()?)
        } else {
            None
        };
        Ok(Stmt::If { condition, then_branch, else_branch })
    }
    
    fn parse_while_stmt(&mut self) -> ParseResult<Stmt> {
        self.advance();
        self.expect(Token::LeftParen)?;
        let condition = self.parse_expr()?;
        self.expect(Token::RightParen)?;
        let body = self.parse_block()?;
        Ok(Stmt::While { condition, body })
    }
    
    fn parse_return_stmt(&mut self) -> ParseResult<Stmt> {
        self.advance();
        let value = if !self.match_token(&Token::Semicolon) {
            Some(self.parse_expr()?)
        } else {
            None
        };
        if value.is_some() {
            self.expect(Token::Semicolon)?;
        }
        Ok(Stmt::Return(value))
    }
    
    fn parse_block(&mut self) -> ParseResult<Vec<Stmt>> {
        self.expect(Token::LeftBrace)?;
        let mut statements = Vec::new();
        while !self.match_token(&Token::RightBrace) && !self.is_at_end() {
            statements.push(self.parse_stmt()?);
        }
        self.expect(Token::RightBrace)?;
        Ok(statements)
    }
    
    fn parse_expr_stmt(&mut self) -> ParseResult<Stmt> {
        let expr = self.parse_expr()?;
        self.expect(Token::Semicolon)?;
        Ok(Stmt::Expression(expr))
    }
    
    fn parse_expr(&mut self) -> ParseResult<Expr> {
        self.parse_comparison()
    }
    
    fn parse_comparison(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_addition()?;
        while let Some(op) = self.match_comparison_op() {
            self.advance();
            let right = self.parse_addition()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }
    
    fn parse_addition(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_multiplication()?;
        while let Some(op) = self.match_addition_op() {
            self.advance();
            let right = self.parse_multiplication()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }
    
    fn parse_multiplication(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_unary()?;
        while let Some(op) = self.match_multiplication_op() {
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }
    
    fn parse_unary(&mut self) -> ParseResult<Expr> {
        if let Some(op) = self.match_unary_op() {
            self.advance();
            let expr = self.parse_unary()?;
            return Ok(Expr::Unary {
                op,
                expr: Box::new(expr),
            });
        }
        self.parse_primary()
    }
    
    fn parse_primary(&mut self) -> ParseResult<Expr> {
        match self.peek().cloned() {
            Some(Token::Number(n)) => {
                self.advance();
                Ok(Expr::Literal(Literal::Number(n)))
            },
            Some(Token::Float(f)) => {
                self.advance();
                Ok(Expr::Literal(Literal::Float(f)))
            },
            Some(Token::String(s)) => {
                self.advance();
                Ok(Expr::Literal(Literal::String(s)))
            },
            Some(Token::Boolean(b)) => {
                self.advance();
                Ok(Expr::Literal(Literal::Boolean(b)))
            },
            Some(Token::Identifier(name)) => {
                self.advance();
                Ok(Expr::Identifier(name.to_string()))
            },
            Some(Token::LeftParen) => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(Token::RightParen)?;
                Ok(expr)
            },
            _ => Err(format!("Unexpected token: {:?}", self.peek()))
        }
    }
    
    // Helper methods
    fn advance(&mut self) {
        self.current += 1;
    }
    
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current).cloned()
    }
    
    fn is_at_end(&self) -> bool {
        matches!(self.peek(), Some(Token::EOF))
    }
    
    fn match_token(&self, token: &Token) -> bool {
        matches!(self.peek(), Some(t) if t == token)
    }
    
    fn expect(&mut self, token: Token) -> ParseResult<()> {
        if self.match_token(&token) {
            self.advance();
            Ok(())
        } else {
            Err(format!("Expected {:?}, got {:?}", self.peek()))
        }
    }
    
    fn expect_identifier(&mut self) -> ParseResult<String> {
        match self.peek() {
            Some(Token::Identifier(name)) => {
                let name = name.to_string();
                self.advance();
                Ok(name)
            },
            _ => Err(format!("Expected identifier, got {:?}", self.peek()))
        }
    }
    
    fn match_comparison_op(&self) -> Option<BinaryOp> {
        match self.peek() {
            Some(Token::EqualEqual) => Some(BinaryOp::Eq),
            Some(Token::BangEqual) => Some(BinaryOp::Ne),
            Some(Token::Less) => Some(BinaryOp::Lt),
            Some(Token::LessEqual) => Some(BinaryOp::Le),
            Some(Token::Greater) => Some(BinaryOp::Gt),
            Some(Token::GreaterEqual) => Some(BinaryOp::Ge),
            _ => None,
        }
    }
    
    fn match_addition_op(&self) -> Option<BinaryOp> {
        match self.peek() {
            Some(Token::Plus) => Some(BinaryOp::Add),
            Some(Token::Minus) => Some(BinaryOp::Sub),
            _ => None,
        }
    }
    
    fn match_multiplication_op(&self) -> Option<BinaryOp> {
        match self.peek() {
            Some(Token::Star) => Some(BinaryOp::Mul),
            Some(Token::Slash) => Some(BinaryOp::Div),
            Some(Token::Percent) => Some(BinaryOp::Mod),
            _ => None,
        }
    }
    
    fn match_unary_op(&self) -> Option<UnaryOp> {
        match self.peek() {
            Some(Token::Minus) => Some(UnaryOp::Neg),
            Some(Token::Bang) => Some(UnaryOp::Not),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number() {
        let mut parser = Parser::new("42");
        let program = parser.parse().unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_parse_binary_expr() {
        let mut parser = Parser::new("1 + 2");
        let program = parser.parse().unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_parse_var_decl() {
        let mut parser = Parser::new("let x = 42;");
        let program = parser.parse().unwrap();
        assert_eq!(program.statements.len(), 1);
    }
}
