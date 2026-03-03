//! Abstract Syntax Tree (AST) definitions for LightLang

use std::fmt;

/// Binary operators
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

/// Unary operators
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
}

/// Literals
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Number(i64),
    Float(f64),
    String(String),
    Boolean(bool),
}

/// Expressions
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Literal),
    Identifier(String),
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
}

/// Statements
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expression(Expr),
    VariableDecl {
        name: String,
        init: Option<Expr>,
        is_const: bool,
    },
    FunctionDecl {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    Return(Option<Expr>),
    Break,
    Continue,
}

/// Program
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

impl Program {
    pub fn new(statements: Vec<Stmt>) -> Self {
        Program { statements }
    }
}
