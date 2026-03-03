//! Lexer module - 词法分析器
//!
//! 将源代码转换为 Token 流

use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LexerError {
    #[error("Unexpected character '{0}' at line {1}, column {2}")]
    UnexpectedChar(char, usize, usize),
    
    #[error("Unterminated string at line {0}")]
    UnterminatedString(usize),
    
    #[error("Invalid number '{0}' at line {1}")]
    InvalidNumber(String, usize),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenType {
    // Literals
    Number(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Identifier(String),
    
    // Keywords
    Let,
    Const,
    Function,
    Return,
    If,
    Else,
    While,
    For,
    True,
    False,
    Null,
    
    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equal,
    EqualEqual,
    BangEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
    Bang,
    
    // Delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Semicolon,
    Colon,
    Dot,
    Arrow,
    
    // Special
    EOF,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(token_type: TokenType, line: usize, column: usize) -> Self {
        Self {
            token_type,
            line,
            column,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} ({}:{})", self.token_type, self.line, self.column)
    }
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }
    
    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();
        
        while !self.is_at_end() {
            let token = self.next_token()?;
            tokens.push(token);
        }
        
        tokens.push(Token::new(TokenType::EOF, self.line, self.column));
        Ok(tokens)
    }
    
    fn next_token(&mut self) -> Result<Token, LexerError> {
        self.skip_whitespace();
        
        if self.is_at_end() {
            return Ok(Token::new(TokenType::EOF, self.line, self.column));
        }
        
        let c = self.current();
        let line = self.line;
        let column = self.column;
        
        match c {
            // Single character tokens
            '(' => { self.advance(); Ok(Token::new(TokenType::LParen, line, column)) }
            ')' => { self.advance(); Ok(Token::new(TokenType::RParen, line, column)) }
            '{' => { self.advance(); Ok(Token::new(TokenType::LBrace, line, column)) }
            '}' => { self.advance(); Ok(Token::new(TokenType::RBrace, line, column)) }
            '[' => { self.advance(); Ok(Token::new(TokenType::LBracket, line, column)) }
            ']' => { self.advance(); Ok(Token::new(TokenType::RBracket, line, column)) }
            ',' => { self.advance(); Ok(Token::new(TokenType::Comma, line, column)) }
            ';' => { self.advance(); Ok(Token::new(TokenType::Semicolon, line, column)) }
            ':' => { self.advance(); Ok(Token::new(TokenType::Colon, line, column)) }
            '.' => { self.advance(); Ok(Token::new(TokenType::Dot, line, column)) }
            
            // Operators
            '+' => { self.advance(); Ok(Token::new(TokenType::Plus, line, column)) }
            '-' => {
                self.advance();
                if self.current() == '>' {
                    self.advance();
                    Ok(Token::new(TokenType::Arrow, line, column))
                } else {
                    Ok(Token::new(TokenType::Minus, line, column))
                }
            }
            '*' => { self.advance(); Ok(Token::new(TokenType::Star, line, column)) }
            '/' => { self.advance(); Ok(Token::new(TokenType::Slash, line, column)) }
            '%' => { self.advance(); Ok(Token::new(TokenType::Percent, line, column)) }
            
            '!' => {
                self.advance();
                if self.current() == '=' {
                    self.advance();
                    Ok(Token::new(TokenType::BangEqual, line, column))
                } else {
                    Ok(Token::new(TokenType::Bang, line, column))
                }
            }
            
            '=' => {
                self.advance();
                if self.current() == '=' {
                    self.advance();
                    Ok(Token::new(TokenType::EqualEqual, line, column))
                } else {
                    Ok(Token::new(TokenType::Equal, line, column))
                }
            }
            
            '<' => {
                self.advance();
                if self.current() == '=' {
                    self.advance();
                    Ok(Token::new(TokenType::LessEqual, line, column))
                } else {
                    Ok(Token::new(TokenType::Less, line, column))
                }
            }
            
            '>' => {
                self.advance();
                if self.current() == '=' {
                    self.advance();
                    Ok(Token::new(TokenType::GreaterEqual, line, column))
                } else {
                    Ok(Token::new(TokenType::Greater, line, column))
                }
            }
            
            '&' => {
                self.advance();
                if self.current() == '&' {
                    self.advance();
                    Ok(Token::new(TokenType::And, line, column))
                } else {
                    Err(LexerError::UnexpectedChar('&', line, column))
                }
            }
            
            '|' => {
                self.advance();
                if self.current() == '|' {
                    self.advance();
                    Ok(Token::new(TokenType::Or, line, column))
                } else {
                    Err(LexerError::UnexpectedChar('|', line, column))
                }
            }
            
            // String literal
            '"' | '\'' => self.read_string(c, line, column),
            
            // Number literal
            '0'..='9' => self.read_number(line, column),
            
            // Identifier or keyword
            'a'..='z' | 'A'..='Z' | '_' => self.read_identifier(line, column),
            
            _ => Err(LexerError::UnexpectedChar(c, line, column)),
        }
    }
    
    fn read_string(&mut self, quote: char, line: usize, column: usize) -> Result<Token, LexerError> {
        self.advance(); // consume opening quote
        
        let mut s = String::new();
        
        while !self.is_at_end() && self.current() != quote {
            if self.current() == '\n' {
                return Err(LexerError::UnterminatedString(line));
            }
            s.push(self.current());
            self.advance();
        }
        
        if self.is_at_end() {
            return Err(LexerError::UnterminatedString(line));
        }
        
        self.advance(); // consume closing quote
        Ok(Token::new(TokenType::String(s), line, column))
    }
    
    fn read_number(&mut self, line: usize, column: usize) -> Result<Token, LexerError> {
        let mut num_str = String::new();
        let mut has_dot = false;
        
        while !self.is_at_end() {
            let c = self.current();
            if c.is_ascii_digit() {
                num_str.push(c);
                self.advance();
            } else if c == '.' && !has_dot {
                has_dot = true;
                num_str.push(c);
                self.advance();
            } else {
                break;
            }
        }
        
        if has_dot {
            let num = num_str.parse::<f64>()
                .map_err(|_| LexerError::InvalidNumber(num_str, line))?;
            Ok(Token::new(TokenType::Float(num), line, column))
        } else {
            let num = num_str.parse::<i64>()
                .map_err(|_| LexerError::InvalidNumber(num_str, line))?;
            Ok(Token::new(TokenType::Number(num), line, column))
        }
    }
    
    fn read_identifier(&mut self, line: usize, column: usize) -> Result<Token, LexerError> {
        let mut ident = String::new();
        
        while !self.is_at_end() {
            let c = self.current();
            if c.is_ascii_alphanumeric() || c == '_' {
                ident.push(c);
                self.advance();
            } else {
                break;
            }
        }
        
        let token_type = match ident.as_str() {
            "let" => TokenType::Let,
            "const" => TokenType::Const,
            "function" => TokenType::Function,
            "return" => TokenType::Return,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "while" => TokenType::While,
            "for" => TokenType::For,
            "true" => TokenType::Boolean(true),
            "false" => TokenType::Boolean(false),
            "null" => TokenType::Null,
            _ => TokenType::Identifier(ident),
        };
        
        Ok(Token::new(token_type, line, column))
    }
    
    fn skip_whitespace(&mut self) {
        while !self.is_at_end() {
            match self.current() {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                '\n' => {
                    self.line += 1;
                    self.column = 0;
                    self.advance();
                }
                '/' => {
                    if self.peek() == '/' {
                        // Single line comment
                        while !self.is_at_end() && self.current() != '\n' {
                            self.advance();
                        }
                    } else if self.peek() == '*' {
                        // Multi-line comment
                        self.advance();
                        self.advance();
                        while !self.is_at_end() {
                            if self.current() == '*' && self.peek() == '/' {
                                self.advance();
                                self.advance();
                                break;
                            }
                            if self.current() == '\n' {
                                self.line += 1;
                                self.column = 0;
                            }
                            self.advance();
                        }
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
    }
    
    fn current(&self) -> char {
        self.input.get(self.position).copied().unwrap_or('\0')
    }
    
    fn peek(&self) -> char {
        self.input.get(self.position + 1).copied().unwrap_or('\0')
    }
    
    fn advance(&mut self) {
        if !self.is_at_end() {
            self.position += 1;
            self.column += 1;
        }
    }
    
    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_tokens() {
        let mut lexer = Lexer::new("let x = 42;");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens.len(), 5);
        assert!(matches!(tokens[0].token_type, TokenType::Let));
        assert!(matches!(tokens[1].token_type, TokenType::Identifier(_)));
        assert!(matches!(tokens[2].token_type, TokenType::Equal));
        assert!(matches!(tokens[3].token_type, TokenType::Number(42)));
        assert!(matches!(tokens[4].token_type, TokenType::Semicolon));
    }
    
    #[test]
    fn test_string_literal() {
        let mut lexer = Lexer::new(r#""hello world""#);
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens.len(), 2);
        if let TokenType::String(s) = &tokens[0].token_type {
            assert_eq!(s, "hello world");
        } else {
            panic!("Expected string token");
        }
    }
    
    #[test]
    fn test_operators() {
        let mut lexer = Lexer::new("1 + 2 * 3 == 9 && true");
        let tokens = lexer.tokenize().unwrap();
        
        assert!(matches!(tokens[0].token_type, TokenType::Number(1)));
        assert!(matches!(tokens[1].token_type, TokenType::Plus));
        assert!(matches!(tokens[2].token_type, TokenType::Number(2)));
        assert!(matches!(tokens[3].token_type, TokenType::Star));
        assert!(matches!(tokens[4].token_type, TokenType::Number(3)));
        assert!(matches!(tokens[5].token_type, TokenType::EqualEqual));
    }
    
    #[test]
    fn test_comments() {
        let mut lexer = Lexer::new(r#"
            // This is a comment
            let x = 10; // inline comment
            /* Multi
               line */
            let y = 20;
        "#);
        let tokens = lexer.tokenize().unwrap();
        
        // Should skip comments
        assert!(tokens.len() < 20);
    }
    
    #[test]
    fn test_arrow_function() {
        let mut lexer = Lexer::new("(x) => x + 1");
        let tokens = lexer.tokenize().unwrap();
        
        assert!(matches!(tokens[2].token_type, TokenType::Arrow));
    }
    
    #[test]
    fn test_float_number() {
        let mut lexer = Lexer::new("3.14");
        let tokens = lexer.tokenize().unwrap();
        
        if let TokenType::Float(f) = tokens[0].token_type {
            assert!((f - 3.14).abs() < 0.001);
        } else {
            panic!("Expected float token");
        }
    }
    
    #[test]
    fn test_error_handling() {
        let mut lexer = Lexer::new("let x = @;");
        let result = lexer.tokenize();
        assert!(result.is_err());
    }
}
