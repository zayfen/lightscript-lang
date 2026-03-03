use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Number(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Char(char),
    
    // Identifiers and Keywords
    Identifier(String),
    Let,
    Const,
    Function,
    Class,
    If,
    Else,
    While,
    For,
    Return,
    Break,
    Continue,
    True,
    False,
    Null,
    Undefined,
    
    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equal,
    EqualEqual,
    Bang,
    BangEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
    
    // Delimiters
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Semicolon,
    Colon,
    Arrow,
    FatArrow,
    
    // Special
    EOF,
    Unknown(char),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Number(n) => write!(f, "Number({})", n),
            Token::Float(n) => write!(f, "Float({})", n),
            Token::String(s) => write!(f, "String({:?})", s),
            Token::Boolean(b) => write!(f, "Boolean({})", b),
            Token::Char(c) => write!(f, "Char({:?})", c),
            Token::Identifier(s) => write!(f, "Identifier({})", s),
            Token::Unknown(c) => write!(f, "Unknown({:?})", c),
            _ => write!(f, "{:?}", self),
        }
    }
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
        }
    }
    
    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        
        while self.position < self.input.len() {
            let ch = self.current_char();
            
            match ch {
                ' ' | '\t' | '\n' | '\r' => {
                    self.advance();
                }
                _ if ch.is_ascii_digit() => {
                    tokens.push(self.read_number()?);
                }
                _ if ch.is_alphabetic() || ch == '_' || ch == '$' => {
                    tokens.push(self.read_identifier());
                }
                '"' | '\'' => {
                    tokens.push(self.read_string()?);
                }
                '+' => { tokens.push(Token::Plus); self.advance(); }
                '-' => { 
                    self.advance();
                    if self.current_char() == '>' {
                        tokens.push(Token::Arrow);
                        self.advance();
                    } else {
                        tokens.push(Token::Minus);
                    }
                }
                '*' => { tokens.push(Token::Star); self.advance(); }
                '/' => { tokens.push(Token::Slash); self.advance(); }
                '%' => { tokens.push(Token::Percent); self.advance(); }
                '=' => {
                    self.advance();
                    if self.current_char() == '=' {
                        tokens.push(Token::EqualEqual);
                        self.advance();
                    } else if self.current_char() == '>' {
                        tokens.push(Token::FatArrow);
                        self.advance();
                    } else {
                        tokens.push(Token::Equal);
                    }
                }
                '!' => {
                    self.advance();
                    if self.current_char() == '=' {
                        tokens.push(Token::BangEqual);
                        self.advance();
                    } else {
                        tokens.push(Token::Bang);
                    }
                }
                '<' => {
                    self.advance();
                    if self.current_char() == '=' {
                        tokens.push(Token::LessEqual);
                        self.advance();
                    } else {
                        tokens.push(Token::Less);
                    }
                }
                '>' => {
                    self.advance();
                    if self.current_char() == '=' {
                        tokens.push(Token::GreaterEqual);
                        self.advance();
                    } else {
                        tokens.push(Token::Greater);
                    }
                }
                '&' => {
                    self.advance();
                    if self.current_char() == '&' {
                        tokens.push(Token::And);
                        self.advance();
                    }
                }
                '|' => {
                    self.advance();
                    if self.current_char() == '|' {
                        tokens.push(Token::Or);
                        self.advance();
                    }
                }
                '(' => { tokens.push(Token::LeftParen); self.advance(); }
                ')' => { tokens.push(Token::RightParen); self.advance(); }
                '{' => { tokens.push(Token::LeftBrace); self.advance(); }
                '}' => { tokens.push(Token::RightBrace); self.advance(); }
                '[' => { tokens.push(Token::LeftBracket); self.advance(); }
                ']' => { tokens.push(Token::RightBracket); self.advance(); }
                ',' => { tokens.push(Token::Comma); self.advance(); }
                ';' => { tokens.push(Token::Semicolon); self.advance(); }
                ':' => { tokens.push(Token::Colon); self.advance(); }
                _ => {
                    tokens.push(Token::Unknown(ch));
                    self.advance();
                }
            }
        }
        
        tokens.push(Token::EOF);
        Ok(tokens)
    }
    
    fn current_char(&self) -> char {
        *self.input.get(self.position).unwrap_or(&'\0')
    }
    
    fn advance(&mut self) {
        self.position += 1;
    }
    
    fn read_number(&mut self) -> Result<Token, String> {
        let mut num_str = String::new();
        let mut is_float = false;
        
        while self.position < self.input.len() {
            let ch = self.current_char();
            if ch.is_ascii_digit() {
                num_str.push(ch);
                self.advance();
            } else if ch == '.' && !is_float {
                is_float = true;
                num_str.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        if is_float {
            num_str.parse::<f64>()
                .map(Token::Float)
                .map_err(|_| format!("Invalid float: {}", num_str))
        } else {
            num_str.parse::<i64>()
                .map(Token::Number)
                .map_err(|_| format!("Invalid number: {}", num_str))
        }
    }
    
    fn read_identifier(&mut self) -> Token {
        let mut ident = String::new();
        
        while self.position < self.input.len() {
            let ch = self.current_char();
            if ch.is_alphanumeric() || ch == '_' || ch == '$' {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        match ident.as_str() {
            "let" => Token::Let,
            "const" => Token::Const,
            "function" | "fn" => Token::Function,
            "class" => Token::Class,
            "if" => Token::If,
            "else" => Token::Else,
            "while" => Token::While,
            "for" => Token::For,
            "return" => Token::Return,
            "break" => Token::Break,
            "continue" => Token::Continue,
            "true" => Token::Boolean(true),
            "false" => Token::Boolean(false),
            "null" => Token::Null,
            "undefined" => Token::Undefined,
            _ => Token::Identifier(ident),
        }
    }
    
    fn read_string(&mut self) -> Result<Token, String> {
        let quote = self.current_char();
        self.advance(); // Skip opening quote
        
        let mut s = String::new();
        
        while self.position < self.input.len() {
            let ch = self.current_char();
            
            if ch == quote {
                self.advance(); // Skip closing quote
                return Ok(Token::String(s));
            } else if ch == '\\' {
                self.advance();
                if self.position < self.input.len() {
                    let escaped = self.current_char();
                    match escaped {
                        'n' => s.push('\n'),
                        't' => s.push('\t'),
                        'r' => s.push('\r'),
                        '\\' => s.push('\\'),
                        '"' => s.push('"'),
                        '\'' => s.push('\''),
                        _ => s.push(escaped),
                    }
                    self.advance();
                }
            } else {
                s.push(ch);
                self.advance();
            }
        }
        
        Err("Unterminated string".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_tokens() {
        let mut lexer = Lexer::new("let x = 42;");
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(tokens[0], Token::Let));
        assert!(matches!(tokens[1], Token::Identifier(_)));
        assert!(matches!(tokens[2], Token::Equal));
        assert!(matches!(tokens[3], Token::Number(42)));
        assert!(matches!(tokens[4], Token::Semicolon));
    }
    
    #[test]
    fn test_string_literal() {
        let mut lexer = Lexer::new("\"hello world\"");
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(tokens[0], Token::String(_)));
    }
    
    #[test]
    fn test_operators() {
        let mut lexer = Lexer::new("+ - * / % == != < <= > >=");
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(tokens[0], Token::Plus));
        assert!(matches!(tokens[1], Token::Minus));
        assert!(matches!(tokens[2], Token::Star));
        assert!(matches!(tokens[3], Token::Slash));
        assert!(matches!(tokens[4], Token::Percent));
    }
    
    #[test]
    fn test_keywords() {
        let mut lexer = Lexer::new("let const function if else while for");
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(tokens[0], Token::Let));
        assert!(matches!(tokens[1], Token::Const));
        assert!(matches!(tokens[2], Token::Function));
    }
}
