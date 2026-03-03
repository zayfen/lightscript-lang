pub mod lexer;

pub mod parser;

pub use lexer::{Lexer, Token};
pub use parser::{Parser, Program};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_integration() {
        let mut lexer = Lexer::new("let x = 42;");
        let tokens = lexer.tokenize().unwrap();
        assert!(tokens.len() > 0);
    }

    #[test]
    fn test_parser_integration() {
        let mut parser = Parser::new("let x = 42;");
        let program = parser.parse().unwrap();
        assert_eq!(program.statements.len(), 1);
    }
}
