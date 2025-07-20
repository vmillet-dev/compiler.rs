use super::token::{Token, TokenType};
use crate::error::CompilerError;
use crate::Result;

/// Analyseur lexical (lexer) pour le langage
pub struct Lexer {
    input: Vec<char>,
    current: usize,
    line: usize,
    column: usize,
    start: usize,
}

impl Lexer {
    /// Crée un nouveau lexer pour l'entrée donnée
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            current: 0,
            line: 1,
            column: 1,
            start: 0,
        }
    }

    /// Tokenise l'entrée complète et retourne la liste des tokens
    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            self.skip_whitespace();

            if self.is_at_end() {
                break;
            }

            self.start = self.current;
            let start_line = self.line;
            let start_column = self.column;

            match self.scan_token() {
                Ok(token_type) => {
                    let lexeme = self.get_lexeme();
                    tokens.push(Token::new(token_type, lexeme, start_line, start_column));
                }
                Err(message) => {
                    return Err(CompilerError::LexError {
                        message,
                        line: start_line,
                        column: start_column,
                    });
                }
            }
        }

        tokens.push(Token::new(
            TokenType::Eof,
            String::new(),
            self.line,
            self.column,
        ));

        Ok(tokens)
    }

    fn scan_token(&mut self) -> std::result::Result<TokenType, String> {
        let c = self.advance();

        match c {
            // Délimiteurs simples
            '(' => Ok(TokenType::LeftParen),
            ')' => Ok(TokenType::RightParen),
            '{' => Ok(TokenType::LeftBrace),
            '}' => Ok(TokenType::RightBrace),
            '[' => Ok(TokenType::LeftBracket),
            ']' => Ok(TokenType::RightBracket),
            ';' => Ok(TokenType::Semicolon),
            ',' => Ok(TokenType::Comma),
            '+' => Ok(TokenType::Plus),
            '-' => Ok(TokenType::Minus),
            '*' => Ok(TokenType::Multiply),
            '/' => {
                if self.match_char('/') {
                    self.skip_line_comment();
                    self.scan_token()
                } else if self.match_char('*') {
                    self.skip_block_comment()?;
                    self.scan_token()
                } else {
                    Ok(TokenType::Divide)
                }
            }
            '%' => Ok(TokenType::Modulo),

            // Opérateurs avec potentiel double caractère
            '=' => {
                if self.match_char('=') {
                    Ok(TokenType::Equal)
                } else {
                    Ok(TokenType::Assign)
                }
            }
            '!' => {
                if self.match_char('=') {
                    Ok(TokenType::NotEqual)
                } else {
                    Ok(TokenType::LogicalNot)
                }
            }
            '<' => {
                if self.match_char('=') {
                    Ok(TokenType::LessEqual)
                } else {
                    Ok(TokenType::LessThan)
                }
            }
            '>' => {
                if self.match_char('=') {
                    Ok(TokenType::GreaterEqual)
                } else {
                    Ok(TokenType::GreaterThan)
                }
            }
            '&' => {
                if self.match_char('&') {
                    Ok(TokenType::LogicalAnd)
                } else {
                    Err("Caractère '&' inattendu".to_string())
                }
            }
            '|' => {
                if self.match_char('|') {
                    Ok(TokenType::LogicalOr)
                } else {
                    Err("Caractère '|' inattendu".to_string())
                }
            }

            // Chaînes de caractères
            '"' => self.string(),

            // Caractères
            '\'' => self.char_literal(),

            // Nombres
            c if c.is_ascii_digit() => self.number(),

            // Identificateurs et mots-clés
            c if c.is_ascii_alphabetic() || c == '_' => self.identifier(),

            _ => Err(format!("Caractère inattendu: '{}'", c)),
        }
    }

    fn string(&mut self) -> std::result::Result<TokenType, String> {
        let mut value = String::new();

        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
                self.column = 0;
            }

            if self.peek() == '\\' {
                self.advance(); // Consommer le '\'
                match self.advance() {
                    'n' => value.push('\n'),
                    't' => value.push('\t'),
                    'r' => value.push('\r'),
                    '\\' => value.push('\\'),
                    '"' => value.push('"'),
                    '\'' => value.push('\''),
                    c => return Err(format!("Séquence d'échappement invalide: \\{}", c)),
                }
            } else {
                value.push(self.advance());
            }
        }

        if self.is_at_end() {
            return Err("Chaîne de caractères non terminée".to_string());
        }

        // Consommer le '"' fermant
        self.advance();

        Ok(TokenType::String(value))
    }

    fn char_literal(&mut self) -> std::result::Result<TokenType, String> {
        if self.is_at_end() {
            return Err("Caractère littéral non terminé".to_string());
        }

        let c = if self.peek() == '\\' {
            self.advance(); // Consommer le '\'
            match self.advance() {
                'n' => '\n',
                't' => '\t',
                'r' => '\r',
                '\\' => '\\',
                '\'' => '\'',
                '"' => '"',
                c => return Err(format!("Séquence d'échappement invalide dans un caractère: \\{}", c)),
            }
        } else {
            self.advance()
        };

        if self.peek() != '\'' {
            return Err("Caractère littéral non terminé".to_string());
        }

        self.advance(); // Consommer le '\'' fermant

        Ok(TokenType::Char(c))
    }

    fn number(&mut self) -> std::result::Result<TokenType, String> {
        // On a déjà consommé le premier chiffre dans scan_token
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        // Vérifier s'il y a une partie décimale
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance(); // Consommer le '.'

            while self.peek().is_ascii_digit() {
                self.advance();
            }

            let value: f64 = self.get_lexeme().parse()
                .map_err(|_| "Nombre flottant invalide".to_string())?;
            Ok(TokenType::Float(value))
        } else {
            let value: i64 = self.get_lexeme().parse()
                .map_err(|_| "Nombre entier invalide".to_string())?;
            Ok(TokenType::Integer(value))
        }
    }

    fn identifier(&mut self) -> std::result::Result<TokenType, String> {
        // On a déjà consommé le premier caractère dans scan_token
        while self.peek().is_ascii_alphanumeric() || self.peek() == '_' {
            self.advance();
        }

        let text = self.get_lexeme();

        let token_type = match text.as_str() {
            "int" => TokenType::Int,
            "float" => TokenType::FloatType,
            "char" => TokenType::CharType,
            "void" => TokenType::Void,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "while" => TokenType::While,
            "for" => TokenType::For,
            "return" => TokenType::Return,
            "break" => TokenType::Break,
            "continue" => TokenType::Continue,
            _ => TokenType::Identifier(text),
        };

        Ok(token_type)
    }

    fn skip_line_comment(&mut self) {
        while self.peek() != '\n' && !self.is_at_end() {
            self.advance();
        }
    }

    fn skip_block_comment(&mut self) -> std::result::Result<(), String> {
        while !self.is_at_end() {
            if self.peek() == '*' && self.peek_next() == '/' {
                self.advance(); // Consommer '*'
                self.advance(); // Consommer '/'
                return Ok(());
            }

            if self.peek() == '\n' {
                self.line += 1;
                self.column = 0;
            }

            self.advance();
        }

        Err("Commentaire bloc non terminé".to_string())
    }

    fn skip_whitespace(&mut self) {
        while !self.is_at_end() {
            match self.peek() {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                '\n' => {
                    self.line += 1;
                    self.column = 0;
                    self.advance();
                }
                _ => break,
            }
        }
    }

    fn advance(&mut self) -> char {
        let c = self.input[self.current];
        self.current += 1;
        self.column += 1;
        c
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.input[self.current] != expected {
            false
        } else {
            self.current += 1;
            self.column += 1;
            true
        }
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.input[self.current]
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.input.len() {
            '\0'
        } else {
            self.input[self.current + 1]
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.input.len()
    }

    fn get_lexeme(&self) -> String {
        self.input[self.start..self.current].iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tokens() {
        let mut lexer = Lexer::new("+ - * / ( ) { } [ ] ; ,");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].token_type, TokenType::Plus);
        assert_eq!(tokens[1].token_type, TokenType::Minus);
        assert_eq!(tokens[2].token_type, TokenType::Multiply);
        assert_eq!(tokens[3].token_type, TokenType::Divide);
    }

    #[test]
    fn test_keywords() {
        let mut lexer = Lexer::new("int float if else while return");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].token_type, TokenType::Int);
        assert_eq!(tokens[1].token_type, TokenType::FloatType);
        assert_eq!(tokens[2].token_type, TokenType::If);
        assert_eq!(tokens[3].token_type, TokenType::Else);
        assert_eq!(tokens[4].token_type, TokenType::While);
        assert_eq!(tokens[5].token_type, TokenType::Return);
    }

    #[test]
    fn test_numbers() {
        let mut lexer = Lexer::new("123 45.67");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].token_type, TokenType::Integer(123));
        assert_eq!(tokens[1].token_type, TokenType::Float(45.67));
    }
}