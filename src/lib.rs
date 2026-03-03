//! LightLang - A modern systems programming language

pub mod lexer;
pub mod parser;
pub mod semantic;
pub mod ir;
pub mod codegen;

pub use lexer::{Lexer, Token};
pub use parser::Parser;
pub use semantic::SemanticAnalyzer;
pub use ir::{IRModule, IRBuilder};
pub use codegen::{CodeGenerator, X86_64Generator, LLVMTextGenerator};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_pipeline() {
        // Lexing
        let mut lexer = Lexer::new("let x = 42;");
        let tokens = lexer.tokenize().unwrap();
        assert!(tokens.len() > 0);
        
        // Parsing
        let mut parser = Parser::new("let x = 42;");
        let program = parser.parse().unwrap();
        assert_eq!(program.statements.len(), 1);
        
        // Semantic analysis
        let analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&program);
        assert!(result.is_ok());
        
        // IR generation
        let builder = IRBuilder::new();
        let module = builder.build(&program);
        assert_eq!(module.functions.len(), 1);
        
        // Code generation (LLVM IR)
        let gen = LLVMTextGenerator::new();
        let output = gen.generate(&module).unwrap();
        assert!(output.contains("define"));
    }
}
