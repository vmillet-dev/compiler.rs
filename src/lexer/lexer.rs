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

            // Essayer de scanner un token
            let token_result = self.scan_token();

            match token_result {
                Ok(Some(token_type)) => {
                    let lexeme = self.get_lexeme();
                    tokens.push(Token::new(token_type, lexeme, start_line, start_column));
                }
                Ok(None) => {
                    // Token ignoré (comme les commentaires), continuer
                    continue;
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

    fn scan_token(&mut self) -> std::result::Result<Option<TokenType>, String> {
        let c = self.advance();

        match c {
            // Délimiteurs simples
            '(' => Ok(Some(TokenType::LeftParen)),
            ')' => Ok(Some(TokenType::RightParen)),
            '{' => Ok(Some(TokenType::LeftBrace)),
            '}' => Ok(Some(TokenType::RightBrace)),
            '[' => Ok(Some(TokenType::LeftBracket)),
            ']' => Ok(Some(TokenType::RightBracket)),
            ';' => Ok(Some(TokenType::Semicolon)),
            ',' => Ok(Some(TokenType::Comma)),
            '+' => Ok(Some(TokenType::Plus)),
            '-' => Ok(Some(TokenType::Minus)),
            '*' => Ok(Some(TokenType::Multiply)),
            '/' => {
                if self.match_char('/') {
                    self.skip_line_comment();
                    Ok(None) // Retourne None pour ignorer le commentaire
                } else if self.match_char('*') {
                    self.skip_block_comment()?;
                    Ok(None) // Retourne None pour ignorer le commentaire
                } else {
                    Ok(Some(TokenType::Divide))
                }
            }
            '%' => Ok(Some(TokenType::Modulo)),

            // Opérateurs avec potentiel double caractère
            '=' => {
                if self.match_char('=') {
                    Ok(Some(TokenType::Equal))
                } else {
                    Ok(Some(TokenType::Assign))
                }
            }
            '!' => {
                if self.match_char('=') {
                    Ok(Some(TokenType::NotEqual))
                } else {
                    Ok(Some(TokenType::LogicalNot))
                }
            }
            '<' => {
                if self.match_char('=') {
                    Ok(Some(TokenType::LessEqual))
                } else {
                    Ok(Some(TokenType::LessThan))
                }
            }
            '>' => {
                if self.match_char('=') {
                    Ok(Some(TokenType::GreaterEqual))
                } else {
                    Ok(Some(TokenType::GreaterThan))
                }
            }
            '&' => {
                if self.match_char('&') {
                    Ok(Some(TokenType::LogicalAnd))
                } else {
                    Err("Caractère '&' inattendu".to_string())
                }
            }
            '|' => {
                if self.match_char('|') {
                    Ok(Some(TokenType::LogicalOr))
                } else {
                    Err("Caractère '|' inattendu".to_string())
                }
            }

            // Chaînes de caractères
            '"' => Ok(Some(self.string()?)),

            // Caractères
            '\'' => Ok(Some(self.char_literal()?)),

            // Nombres
            c if c.is_ascii_digit() => Ok(Some(self.number()?)),

            // Identificateurs et mots-clés
            c if c.is_ascii_alphabetic() || c == '_' => Ok(Some(self.identifier()?)),

            _ => Err(format!("Caractère inattendu: '{}'", c)),
        }
    }

    fn string(&mut self) -> std::result::Result<TokenType, String> {
        // Le code reste identique
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
        // Le code reste identique
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
        // Le code reste identique
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
        // Le code reste identique
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
            "println" => TokenType::Println,
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
    fn test_delimiters() {
        let mut lexer = Lexer::new("( ) { } [ ] ; ,");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].token_type, TokenType::LeftParen);
        assert_eq!(tokens[1].token_type, TokenType::RightParen);
        assert_eq!(tokens[2].token_type, TokenType::LeftBrace);
        assert_eq!(tokens[3].token_type, TokenType::RightBrace);
        assert_eq!(tokens[4].token_type, TokenType::LeftBracket);
        assert_eq!(tokens[5].token_type, TokenType::RightBracket);
        assert_eq!(tokens[6].token_type, TokenType::Semicolon);
        assert_eq!(tokens[7].token_type, TokenType::Comma);
        assert_eq!(tokens[8].token_type, TokenType::Eof);
    }

    #[test]
    fn test_arithmetic_operators() {
        let mut lexer = Lexer::new("+ - * / %");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].token_type, TokenType::Plus);
        assert_eq!(tokens[1].token_type, TokenType::Minus);
        assert_eq!(tokens[2].token_type, TokenType::Multiply);
        assert_eq!(tokens[3].token_type, TokenType::Divide);
        assert_eq!(tokens[4].token_type, TokenType::Modulo);
        assert_eq!(tokens[5].token_type, TokenType::Eof);
    }

    #[test]
    fn test_comparison_operators() {
        let mut lexer = Lexer::new("== != < <= > >=");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].token_type, TokenType::Equal);
        assert_eq!(tokens[1].token_type, TokenType::NotEqual);
        assert_eq!(tokens[2].token_type, TokenType::LessThan);
        assert_eq!(tokens[3].token_type, TokenType::LessEqual);
        assert_eq!(tokens[4].token_type, TokenType::GreaterThan);
        assert_eq!(tokens[5].token_type, TokenType::GreaterEqual);
        assert_eq!(tokens[6].token_type, TokenType::Eof);
    }

    #[test]
    fn test_logical_operators() {
        let mut lexer = Lexer::new("&& || !");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].token_type, TokenType::LogicalAnd);
        assert_eq!(tokens[1].token_type, TokenType::LogicalOr);
        assert_eq!(tokens[2].token_type, TokenType::LogicalNot);
        assert_eq!(tokens[3].token_type, TokenType::Eof);
    }

    #[test]
    fn test_assignment_operator() {
        let mut lexer = Lexer::new("= x = 5");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].token_type, TokenType::Assign);
        assert_eq!(tokens[1].token_type, TokenType::Identifier("x".to_string()));
        assert_eq!(tokens[2].token_type, TokenType::Assign);
        assert_eq!(tokens[3].token_type, TokenType::Integer(5));
        assert_eq!(tokens[4].token_type, TokenType::Eof);
    }

    #[test]
    fn test_keywords() {
        let mut lexer = Lexer::new("int float char void if else while for return break continue");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].token_type, TokenType::Int);
        assert_eq!(tokens[1].token_type, TokenType::FloatType);
        assert_eq!(tokens[2].token_type, TokenType::CharType);
        assert_eq!(tokens[3].token_type, TokenType::Void);
        assert_eq!(tokens[4].token_type, TokenType::If);
        assert_eq!(tokens[5].token_type, TokenType::Else);
        assert_eq!(tokens[6].token_type, TokenType::While);
        assert_eq!(tokens[7].token_type, TokenType::For);
        assert_eq!(tokens[8].token_type, TokenType::Return);
        assert_eq!(tokens[9].token_type, TokenType::Break);
        assert_eq!(tokens[10].token_type, TokenType::Continue);
        assert_eq!(tokens[11].token_type, TokenType::Eof);
    }

    #[test]
    fn test_identifiers() {
        let mut lexer = Lexer::new("variable_name _private myVar test123 _");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].token_type, TokenType::Identifier("variable_name".to_string()));
        assert_eq!(tokens[1].token_type, TokenType::Identifier("_private".to_string()));
        assert_eq!(tokens[2].token_type, TokenType::Identifier("myVar".to_string()));
        assert_eq!(tokens[3].token_type, TokenType::Identifier("test123".to_string()));
        assert_eq!(tokens[4].token_type, TokenType::Identifier("_".to_string()));
        assert_eq!(tokens[5].token_type, TokenType::Eof);
    }

    #[test]
    fn test_integers() {
        let mut lexer = Lexer::new("0 123 456789");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].token_type, TokenType::Integer(0));
        assert_eq!(tokens[1].token_type, TokenType::Integer(123));
        assert_eq!(tokens[2].token_type, TokenType::Integer(456789));
        assert_eq!(tokens[3].token_type, TokenType::Eof);
    }

    #[test]
    fn test_floats() {
        let mut lexer = Lexer::new("0.0 3.14 123.456");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].token_type, TokenType::Float(0.0));
        assert_eq!(tokens[1].token_type, TokenType::Float(3.14));
        assert_eq!(tokens[2].token_type, TokenType::Float(123.456));
        assert_eq!(tokens[3].token_type, TokenType::Eof);
    }

    #[test]
    fn test_strings() {
        let mut lexer = Lexer::new(r#""hello" "world with spaces" "" "with\nnewline""#);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].token_type, TokenType::String("hello".to_string()));
        assert_eq!(tokens[1].token_type, TokenType::String("world with spaces".to_string()));
        assert_eq!(tokens[2].token_type, TokenType::String("".to_string()));
        assert_eq!(tokens[3].token_type, TokenType::String("with\nnewline".to_string()));
        assert_eq!(tokens[4].token_type, TokenType::Eof);
    }

    #[test]
    fn test_string_escapes() {
        let mut lexer = Lexer::new(r#""line1\nline2" "tab\ttab" "quote\"quote" "backslash\\""#);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].token_type, TokenType::String("line1\nline2".to_string()));
        assert_eq!(tokens[1].token_type, TokenType::String("tab\ttab".to_string()));
        assert_eq!(tokens[2].token_type, TokenType::String("quote\"quote".to_string()));
        assert_eq!(tokens[3].token_type, TokenType::String("backslash\\".to_string()));
        assert_eq!(tokens[4].token_type, TokenType::Eof);
    }

    #[test]
    fn test_characters() {
        let mut lexer = Lexer::new(r"'a' 'Z' '1' ' '");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].token_type, TokenType::Char('a'));
        assert_eq!(tokens[1].token_type, TokenType::Char('Z'));
        assert_eq!(tokens[2].token_type, TokenType::Char('1'));
        assert_eq!(tokens[3].token_type, TokenType::Char(' '));
        assert_eq!(tokens[4].token_type, TokenType::Eof);
    }

    #[test]
    fn test_line_comments() {
        let mut lexer = Lexer::new("int x; // This is a comment\nfloat y;");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].token_type, TokenType::Int);
        assert_eq!(tokens[1].token_type, TokenType::Identifier("x".to_string()));
        assert_eq!(tokens[2].token_type, TokenType::Semicolon);
        assert_eq!(tokens[3].token_type, TokenType::FloatType);
        assert_eq!(tokens[4].token_type, TokenType::Identifier("y".to_string()));
        assert_eq!(tokens[5].token_type, TokenType::Semicolon);
        assert_eq!(tokens[6].token_type, TokenType::Eof);
    }

    #[test]
    fn test_block_comments() {
        let mut lexer = Lexer::new("int /* this is a block comment */ x;");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].token_type, TokenType::Int);
        assert_eq!(tokens[1].token_type, TokenType::Identifier("x".to_string()));
        assert_eq!(tokens[2].token_type, TokenType::Semicolon);
        assert_eq!(tokens[3].token_type, TokenType::Eof);
    }

    #[test]
    fn test_multiline_block_comment() {
        let mut lexer = Lexer::new("int /*\n  multiline\n  comment\n*/ x;");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].token_type, TokenType::Int);
        assert_eq!(tokens[1].token_type, TokenType::Identifier("x".to_string()));
        assert_eq!(tokens[2].token_type, TokenType::Semicolon);
        assert_eq!(tokens[3].token_type, TokenType::Eof);
    }

    #[test]
    fn test_comment_at_end_of_file() {
        let mut lexer = Lexer::new("int x; // comment at end");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].token_type, TokenType::Int);
        assert_eq!(tokens[1].token_type, TokenType::Identifier("x".to_string()));
        assert_eq!(tokens[2].token_type, TokenType::Semicolon);
        assert_eq!(tokens[3].token_type, TokenType::Eof);
    }

    #[test]
    fn test_nested_comments() {
        // Test que les commentaires de ligne dans les commentaires de bloc sont ignorés
        let mut lexer = Lexer::new("int /* block // line comment inside */ x;");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].token_type, TokenType::Int);
        assert_eq!(tokens[1].token_type, TokenType::Identifier("x".to_string()));
        assert_eq!(tokens[2].token_type, TokenType::Semicolon);
        assert_eq!(tokens[3].token_type, TokenType::Eof);
    }

    #[test]
    fn test_whitespace_handling() {
        let mut lexer = Lexer::new("  \t\n  int\n\tx\r\n  ;  ");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].token_type, TokenType::Int);
        assert_eq!(tokens[1].token_type, TokenType::Identifier("x".to_string()));
        assert_eq!(tokens[2].token_type, TokenType::Semicolon);
        assert_eq!(tokens[3].token_type, TokenType::Eof);
    }

    #[test]
    fn test_line_and_column_tracking() {
        let mut lexer = Lexer::new("int\nx\n;");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].line, 1);
        assert_eq!(tokens[0].column, 1);
        assert_eq!(tokens[1].line, 2);
        assert_eq!(tokens[1].column, 1);
        assert_eq!(tokens[2].line, 3);
        assert_eq!(tokens[2].column, 1);
    }

    #[test]
    fn test_complex_expression() {
        let mut lexer = Lexer::new("if (x >= 10 && y <= 20) { return x + y * 2; }");
        let tokens = lexer.tokenize().unwrap();

        let expected_types = vec![
            TokenType::If,
            TokenType::LeftParen,
            TokenType::Identifier("x".to_string()),
            TokenType::GreaterEqual,
            TokenType::Integer(10),
            TokenType::LogicalAnd,
            TokenType::Identifier("y".to_string()),
            TokenType::LessEqual,
            TokenType::Integer(20),
            TokenType::RightParen,
            TokenType::LeftBrace,
            TokenType::Return,
            TokenType::Identifier("x".to_string()),
            TokenType::Plus,
            TokenType::Identifier("y".to_string()),
            TokenType::Multiply,
            TokenType::Integer(2),
            TokenType::Semicolon,
            TokenType::RightBrace,
            TokenType::Eof,
        ];

        for (i, expected) in expected_types.iter().enumerate() {
            assert_eq!(tokens[i].token_type, *expected, "Token {} mismatch", i);
        }
    }

    // Tests d'erreurs
    #[test]
    fn test_unterminated_string() {
        let mut lexer = Lexer::new(r#""unterminated string"#);
        let result = lexer.tokenize();
        assert!(result.is_err());
    }

    #[test]
    fn test_unterminated_char() {
        let mut lexer = Lexer::new("'a");
        let result = lexer.tokenize();
        assert!(result.is_err());
    }

    #[test]
    fn test_unterminated_block_comment() {
        let mut lexer = Lexer::new("/* unterminated comment");
        let result = lexer.tokenize();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_escape_sequence_string() {
        let mut lexer = Lexer::new(r#""\x""#);
        let result = lexer.tokenize();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_escape_sequence_char() {
        let mut lexer = Lexer::new(r"'\x'");
        let result = lexer.tokenize();
        assert!(result.is_err());
    }

    #[test]
    fn test_unexpected_character() {
        let mut lexer = Lexer::new("@");
        let result = lexer.tokenize();
        assert!(result.is_err());
    }

    #[test]
    fn test_single_ampersand_error() {
        let mut lexer = Lexer::new("&");
        let result = lexer.tokenize();
        assert!(result.is_err());
    }

    #[test]
    fn test_single_pipe_error() {
        let mut lexer = Lexer::new("|");
        let result = lexer.tokenize();
        assert!(result.is_err());
    }

    #[test]
    fn test_lexeme_extraction() {
        let mut lexer = Lexer::new("hello 123 3.14");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].lexeme, "hello");
        assert_eq!(tokens[1].lexeme, "123");
        assert_eq!(tokens[2].lexeme, "3.14");
    }

    #[test]
    fn test_edge_cases() {
        // Test avec juste des espaces
        let mut lexer = Lexer::new("   ");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token_type, TokenType::Eof);

        // Test avec string vide
        let mut lexer = Lexer::new("");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token_type, TokenType::Eof);
    }

    // #[test]
    // fn test_number_followed_by_dot() {
    //     // Test pour s'assurer qu'on ne parse pas "123." comme un float
    //     let mut lexer = Lexer::new("123.x");
    //     let tokens = lexer.tokenize().unwrap();
    //
    //     assert_eq!(tokens[0].token_type, TokenType::Integer(123));
    //     // Le point devrait être traité séparément (ce qui causera une erreur dans ce cas)
    //     // car il n'est pas suivi d'un chiffre
    // }

    #[test]
    fn test_keyword_vs_identifier() {
        let mut lexer = Lexer::new("int integer inte");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].token_type, TokenType::Int);
        assert_eq!(tokens[1].token_type, TokenType::Identifier("integer".to_string()));
        assert_eq!(tokens[2].token_type, TokenType::Identifier("inte".to_string()));
    }

    #[test]
    fn test_multiline_string() {
        let mut lexer = Lexer::new("\"line1\nline2\"");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].token_type, TokenType::String("line1\nline2".to_string()));
        assert_eq!(tokens[0].line, 1); // Commence à la ligne 1
    }
}