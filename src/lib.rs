//! Ziv - A modern systems programming language

pub mod codegen;
pub mod compiler;
pub mod ir;
pub mod lexer;
pub mod parser;
pub mod semantic;
pub use ziv_stdlib as stdlib;

pub use codegen::{CodeGenerator, LLVMTextGenerator, X86_64Generator};
pub use compiler::Compiler;
pub use ir::{IRBuilder, IRModule};
pub use lexer::{Lexer, Token};
pub use parser::Parser;
pub use semantic::{SemanticAnalyzer, Symbol, SymbolKind, Type};
