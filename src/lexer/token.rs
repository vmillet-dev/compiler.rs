use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Integer(i64),
    Float(f64),
    String(String),
    Char(char),

    // Identifiers and keywords
    Identifier(String),

    Int,
    FloatType,
    CharType,
    Void,
    If,
    Else,
    While,
    For,
    Return,
    Break,
    Continue,
    Println,

    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,

    Equal,
    NotEqual,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,

    LogicalAnd,
    LogicalOr,
    LogicalNot,

    Assign,

    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Semicolon,
    Comma,

    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, line: usize, column: usize) -> Self {
        Token {
            token_type,
            lexeme,
            line,
            column,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} '{}' at {}:{}", self.token_type, self.lexeme, self.line, self.column)
    }
}
