//! LightScript - A modern language inspired by JavaScript's best features
//!
//! LightScript 是一个受 JavaScript 启发的现代编程语言，
//! 保留了 JavaScript 的优秀特性，同时去除了历史包袱。

pub mod lexer;
pub mod parser;
pub mod semantic;
pub mod codegen;
pub mod ir;

pub use lexer::{Lexer, Token, TokenType};
pub use parser::{Parser, AST};
pub use semantic::{SemanticAnalyzer, SymbolTable};
pub use codegen::CodeGenerator;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum LightScriptError {
    #[error("Lexer error: {0}")]
    LexerError(String),
    
    #[error("Parser error: {0}")]
    ParserError(String),
    
    #[error("Semantic error: {0}")]
    SemanticError(String),
    
    #[error("Code generation error: {0}")]
    CodeGenError(String),
}

pub type Result<T> = std::result::Result<T, LightScriptError>;
