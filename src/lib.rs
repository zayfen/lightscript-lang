pub mod lexer;

// 暂时注释掉未实现的模块
// pub mod parser;
// pub mod semantic;
// pub mod ir;
// pub mod codegen;

pub use lexer::{Lexer, Token};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer() {
        let mut lexer = Lexer::new("let x = 42;");
        let tokens = lexer.tokenize().unwrap();
        assert!(tokens.len() > 0);
    }
}
