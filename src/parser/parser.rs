use crate::lexer::{Token, TokenType};
use crate::parser::ast::{Expr, Stmt};
use crate::types::Type;
use crate::error::error::CompilerError;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    errors: Vec<CompilerError>,
}

impl Parser {
    pub fn new(mut tokens: Vec<Token>) -> Self {
        // Ensure the token vector always ends with EOF to prevent bounds issues
        if tokens.is_empty() || tokens.last().unwrap().token_type != TokenType::Eof {
            tokens.push(Token::new(TokenType::Eof, String::new(), 1, 1));
        }
        Parser { tokens, current: 0, errors: Vec::new() }
    }
    
    pub fn get_errors(&self) -> &[CompilerError] {
        &self.errors
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        while !self.is_at_end() {
            if let Some(func) = self.function() {
                stmts.push(func);
            } else {
                // Report error for unparseable top-level constructs
                let token = self.peek();
                self.report_error(
                    "Unrecognized top-level construct",
                    Some("Expected function declaration"),
                    token.line,
                    token.column
                );
                self.synchronize();
            }
        }
        stmts
    }

    fn function(&mut self) -> Option<Stmt> {
        let return_type = self.consume_type()?;
        let name = self.consume_identifier()?;
        self.consume(TokenType::LeftParen)?;
        self.consume(TokenType::RightParen)?;
        self.consume(TokenType::LeftBrace)?;

        let mut body = Vec::new();
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            if let Some(stmt) = self.statement() {
                body.push(stmt);
            } else {
                self.synchronize();
            }
        }

        self.consume(TokenType::RightBrace)?;

        Some(Stmt::Function {
            return_type: Type::from(return_type),
            name,
            type_parameters: Vec::new(), // TODO: Parse generic type parameters
            parameters: Vec::new(),      // TODO: Parse function parameters
            body,
        })
    }

    fn statement(&mut self) -> Option<Stmt> {
        if self.match_token(&TokenType::Return) {
            let expr = if !self.check(&TokenType::Semicolon) {
                Some(self.expression()?)
            } else {
                None
            };
            self.consume(TokenType::Semicolon)?;
            return Some(Stmt::Return(expr));
        }

        if self.match_token(&TokenType::LeftBrace) {
            let mut statements = Vec::new();
            while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
                if let Some(stmt) = self.statement() {
                    statements.push(stmt);
                } else {
                    self.synchronize();
                }
            }
            self.consume(TokenType::RightBrace)?;
            return Some(Stmt::Block(statements));
        }

        if self.match_token(&TokenType::If) {
            self.consume(TokenType::LeftParen)?;
            let condition = self.expression()?;
            self.consume(TokenType::RightParen)?;
            self.consume(TokenType::LeftBrace)?;
            let mut then_branch = Vec::new();
            while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
                then_branch.push(self.statement()?);
            }
            self.consume(TokenType::RightBrace)?;
            return Some(Stmt::If { condition, then_branch });
        }

        if self.match_token(&TokenType::Println) {
            self.consume(TokenType::LeftParen)?;

            // Parse the first expression
            let first_expr = self.expression()?;
            
            // Check if this is a format string (string literal) or a simple expression
            match &first_expr {
                Expr::String(_) => {
                    // Format string case: println("format", args...)
                    let mut args = Vec::new();
                    // Parse additional arguments after the format string
                    while !self.check(&TokenType::RightParen) && !self.is_at_end() {
                        // Expect a comma before each additional argument
                        if !self.match_token(&TokenType::Comma) {
                            let token = self.peek();
                            self.report_error(
                                "Expected comma between printf arguments",
                                Some("Add ',' between arguments"),
                                token.line,
                                token.column
                            );
                            return None;
                        }
                        
                        // Parse the argument expression
                        if let Some(expr) = self.expression() {
                            args.push(expr);
                        } else {
                            let token = self.peek();
                            self.report_error(
                                "Expected expression after comma",
                                Some("Provide a valid expression as argument"),
                                token.line,
                                token.column
                            );
                            self.synchronize();
                            return None;
                        }
                    }

                    self.consume(TokenType::RightParen)?;
                    self.consume(TokenType::Semicolon)?;
                    return Some(Stmt::PrintStmt { format_string: first_expr, args });
                }
                _ => {
                    // Simple expression case: println(expr)
                    // Check that there are no additional arguments
                    if self.check(&TokenType::Comma) {
                        let token = self.peek();
                        self.report_error(
                            "Simple println cannot have additional arguments",
                            Some("Use format string for multiple arguments"),
                            token.line,
                            token.column
                        );
                        return None;
                    }
                    
                    self.consume(TokenType::RightParen)?;
                    self.consume(TokenType::Semicolon)?;
                    
                    // Create a simple print statement with the expression as a single argument
                    // We'll use an empty string as format_string to indicate this is a simple print
                    return Some(Stmt::PrintStmt { 
                        format_string: Expr::String(String::new()), 
                        args: vec![first_expr] 
                    });
                }
            }
        }

        if let Some(var_type) = self.match_any_type() {
            let name = self.consume_identifier()?;
            let initializer = if self.match_token(&TokenType::Assign) {
                Some(self.expression()?)
            } else {
                None
            };
            self.consume(TokenType::Semicolon)?;
            return Some(Stmt::VarDecl { var_type: Type::from(var_type), name, initializer });
        }

        let expr = self.expression()?;
        self.consume(TokenType::Semicolon)?;
        Some(Stmt::ExprStmt(expr))
    }

    fn expression(&mut self) -> Option<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> Option<Expr> {
        let expr = self.equality()?;

        // Check if this is an assignment (identifier = expression)
        if let Expr::Identifier(name) = expr {
            if self.match_token(&TokenType::Assign) {
                let value = self.assignment()?; // Right-associative
                return Some(Expr::Assignment {
                    name,
                    value: Box::new(value),
                });
            }
            // If not an assignment, return the identifier as-is
            return Some(Expr::Identifier(name));
        }

        Some(expr)
    }

    fn equality(&mut self) -> Option<Expr> {
        let mut expr = self.comparison()?;
        while let Some(op) = self.match_any(&[TokenType::Equal, TokenType::NotEqual]) {
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }
        Some(expr)
    }

    fn comparison(&mut self) -> Option<Expr> {
        let mut expr = self.term()?;
        while let Some(op) = self.match_any(&[
            TokenType::LessThan,
            TokenType::LessEqual,
            TokenType::GreaterThan,
            TokenType::GreaterEqual,
        ]) {
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }
        Some(expr)
    }

    fn term(&mut self) -> Option<Expr> {
        let mut expr = self.factor()?;
        while let Some(op) = self.match_any(&[TokenType::Plus, TokenType::Minus]) {
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }
        Some(expr)
    }

    fn factor(&mut self) -> Option<Expr> {
        let mut expr = self.unary()?;
        while let Some(op) = self.match_any(&[TokenType::Multiply, TokenType::Divide, TokenType::Modulo]) {
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }
        Some(expr)
    }

    fn unary(&mut self) -> Option<Expr> {
        // Handle unary operators: '!' for logical negation and '-' for arithmetic negation
        if let Some(op) = self.match_any(&[TokenType::LogicalNot, TokenType::Minus]) {
            let operand = self.unary()?; // Recursive to handle !!x or -(-x)
            return Some(Expr::Unary {
                operator: op,
                operand: Box::new(operand),
            });
        }
        self.call() // Move to function call handling
    }

    fn call(&mut self) -> Option<Expr> {
        let mut expr = self.primary()?;

        loop {
            if self.check(&TokenType::LeftParen) {
                self.advance();
                let mut arguments = Vec::new();
                if !self.check(&TokenType::RightParen) {
                    loop {
                        arguments.push(self.expression()?);
                        if !self.match_token(&TokenType::Comma) {
                            break;
                        }
                    }
                }
                self.consume(TokenType::RightParen)?;
                expr = Expr::Call {
                    callee: Box::new(expr),
                    arguments,
                    type_arguments: Vec::new(), // TODO: Parse generic type arguments
                };
            } else {
                break;
            }
        }
        Some(expr)
    }

    fn primary(&mut self) -> Option<Expr> {
        let token = self.advance();
        match &token.token_type {
            TokenType::Integer(i) => Some(Expr::Integer(*i)),
            TokenType::Float(f) => Some(Expr::Float(*f)),
            TokenType::Char(c) => Some(Expr::Char(*c)),
            TokenType::String(s) => Some(Expr::String(s.clone())),
            TokenType::Identifier(name) => Some(Expr::Identifier(name.clone())),
            TokenType::LeftParen => {
                let expr = self.expression()?;
                self.consume(TokenType::RightParen)?;
                Some(expr)
            }
            _ => None,
        }
    }

    fn consume(&mut self, expected: TokenType) -> Option<Token> {
        if self.check(&expected) {
            Some(self.advance())
        } else {
            let token = self.peek();
            let expected_str = format!("{:?}", expected);
            let found_str = format!("{:?}", token.token_type);
            self.report_error(
                &format!("Expected {}, found {}", expected_str, found_str),
                Some(&self.suggest_fix_for_token(&expected)),
                token.line,
                token.column
            );
            None
        }
    }

    fn consume_type(&mut self) -> Option<TokenType> {
        self.match_any(&[TokenType::Int, TokenType::FloatType, TokenType::CharType, TokenType::Void])
    }

    fn consume_identifier(&mut self) -> Option<String> {
        let token = self.peek();
        if let TokenType::Identifier(name) = &token.token_type {
            let name = name.clone();
            self.advance();
            Some(name)
        } else {
            None
        }
    }

    fn match_token(&mut self, expected: &TokenType) -> bool {
        if self.check(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn match_any(&mut self, types: &[TokenType]) -> Option<TokenType> {
        for t in types {
            if self.check(t) {
                return Some(self.advance().token_type.clone());
            }
        }
        None
    }

    fn match_any_type(&mut self) -> Option<TokenType> {
        self.match_any(&[TokenType::Int, TokenType::FloatType, TokenType::CharType])
    }

    fn check(&self, token_type: &TokenType) -> bool {
        !self.is_at_end() && &self.peek().token_type == token_type
    }

    fn advance(&mut self) -> Token {
        let token = self.peek().clone();
        self.current += 1;
        token
    }

    fn peek(&self) -> &Token {
        if self.current >= self.tokens.len() {
            // Return the last token (should be EOF) if we're past the end
            &self.tokens[self.tokens.len() - 1]
        } else {
            &self.tokens[self.current]
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || self.peek().token_type == TokenType::Eof
    }
    
    fn synchronize(&mut self) {
        self.advance();
        
        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }
            
            match self.peek().token_type {
                TokenType::If | TokenType::Return | TokenType::Int | 
                TokenType::FloatType | TokenType::CharType | TokenType::Void |
                TokenType::Println => return,
                _ => {
                    self.advance();
                }
            }
        }
    }
    
    fn previous(&self) -> &Token {
        if self.current == 0 {
            &self.tokens[0]
        } else {
            &self.tokens[self.current - 1]
        }
    }
    
    fn report_error(&mut self, message: &str, suggestion: Option<&str>, line: usize, column: usize) {
        let error = CompilerError::parse_error(message.to_string(), line, column);
        self.errors.push(error);
        eprintln!("Parse Error at {}:{}: {}", line, column, message);
        if let Some(suggestion) = suggestion {
            eprintln!("  Suggestion: {}", suggestion);
        }
    }
    
    fn suggest_fix_for_token(&self, expected: &TokenType) -> String {
        match expected {
            TokenType::Semicolon => "Add ';' at the end of the statement".to_string(),
            TokenType::LeftBrace => "Add '{' to start a block".to_string(),
            TokenType::RightBrace => "Add '}' to close the block".to_string(),
            TokenType::LeftParen => "Add '(' to start parameter list".to_string(),
            TokenType::RightParen => "Add ')' to close parameter list".to_string(),
            TokenType::Comma => "Add ',' to separate items".to_string(),
            _ => format!("Add the expected token: {:?}", expected),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::{Token, TokenType};
    use crate::parser::ast::{Expr, Stmt};

    fn create_token(token_type: TokenType, lexeme: &str) -> Token {
        Token::new(token_type, lexeme.to_string(), 1, 1)
    }

    #[test]
    fn test_parse_simple_function_declaration() {
        // Test parsing: "int main() { }"
        let tokens = vec![
            create_token(TokenType::Int, "int"),
            create_token(TokenType::Identifier("main".to_string()), "main"),
            create_token(TokenType::LeftParen, "("),
            create_token(TokenType::RightParen, ")"),
            create_token(TokenType::LeftBrace, "{"),
            create_token(TokenType::RightBrace, "}"),
            create_token(TokenType::Eof, ""),
        ];

        let mut parser = Parser::new(tokens);
        let result = parser.parse();
        
        assert_eq!(result.len(), 1);
        match &result[0] {
            Stmt::Function { return_type, name, body } => {
                assert_eq!(*return_type, Type::from(TokenType::Int));
                assert_eq!(*name, "main");
                assert!(body.is_empty());
            }
            _ => panic!("Expected function statement"),
        }
    }

    #[test]
    fn test_parse_function_with_statements() {
        // Test parsing: "int test() { return 42; }"
        let tokens = vec![
            create_token(TokenType::Int, "int"),
            create_token(TokenType::Identifier("test".to_string()), "test"),
            create_token(TokenType::LeftParen, "("),
            create_token(TokenType::RightParen, ")"),
            create_token(TokenType::LeftBrace, "{"),
            create_token(TokenType::Return, "return"),
            create_token(TokenType::Integer(42), "42"),
            create_token(TokenType::Semicolon, ";"),
            create_token(TokenType::RightBrace, "}"),
            create_token(TokenType::Eof, ""),
        ];

        let mut parser = Parser::new(tokens);
        let result = parser.parse();
        
        assert_eq!(result.len(), 1);
        match &result[0] {
            Stmt::Function { return_type, name, body } => {
                assert_eq!(*return_type, Type::from(TokenType::Int));
                assert_eq!(*name, "test");
                assert_eq!(body.len(), 1);
                match &body[0] {
                    Stmt::Return(Some(expr)) => {
                        assert_eq!(*expr, Expr::Integer(42));
                    }
                    _ => panic!("Expected return statement"),
                }
            }
            _ => panic!("Expected function statement"),
        }
    }

    #[test]
    fn test_parse_variable_declaration_statement() {
        // Test parsing: "int x = 10;"
        let tokens = vec![
            create_token(TokenType::Int, "int"),
            create_token(TokenType::Identifier("x".to_string()), "x"),
            create_token(TokenType::Assign, "="),
            create_token(TokenType::Integer(10), "10"),
            create_token(TokenType::Semicolon, ";"),
            create_token(TokenType::Eof, ""),
        ];

        let mut parser = Parser::new(tokens);
        let _result = parser.parse();
        
        // Since this is not inside a function, the parser won't recognize it as a top-level statement
        // Let's test the statement parsing directly
        let tokens = vec![
            create_token(TokenType::Int, "int"),
            create_token(TokenType::Identifier("x".to_string()), "x"),
            create_token(TokenType::Assign, "="),
            create_token(TokenType::Integer(10), "10"),
            create_token(TokenType::Semicolon, ";"),
            create_token(TokenType::Eof, ""),
        ];

        let mut parser = Parser::new(tokens);
        if let Some(stmt) = parser.statement() {
            match stmt {
                Stmt::VarDecl { var_type, name, initializer } => {
                    assert_eq!(var_type, Type::from(TokenType::Int));
                    assert_eq!(name, "x");
                    assert_eq!(initializer, Some(Expr::Integer(10)));
                }
                _ => panic!("Expected variable declaration statement"),
            }
        } else {
            panic!("Failed to parse variable declaration");
        }
    }

    #[test]
    fn test_parse_return_statement() {
        // Test parsing: "return 5;"
        let tokens = vec![
            create_token(TokenType::Return, "return"),
            create_token(TokenType::Integer(5), "5"),
            create_token(TokenType::Semicolon, ";"),
            create_token(TokenType::Eof, ""),
        ];

        let mut parser = Parser::new(tokens);
        if let Some(stmt) = parser.statement() {
            match stmt {
                Stmt::Return(Some(expr)) => {
                    assert_eq!(expr, Expr::Integer(5));
                }
                _ => panic!("Expected return statement with value"),
            }
        } else {
            panic!("Failed to parse return statement");
        }
    }

    #[test]
    fn test_parse_if_statement() {
        // Test parsing: "if (x == 5) { return 1; }"
        let tokens = vec![
            create_token(TokenType::If, "if"),
            create_token(TokenType::LeftParen, "("),
            create_token(TokenType::Identifier("x".to_string()), "x"),
            create_token(TokenType::Equal, "=="),
            create_token(TokenType::Integer(5), "5"),
            create_token(TokenType::RightParen, ")"),
            create_token(TokenType::LeftBrace, "{"),
            create_token(TokenType::Return, "return"),
            create_token(TokenType::Integer(1), "1"),
            create_token(TokenType::Semicolon, ";"),
            create_token(TokenType::RightBrace, "}"),
            create_token(TokenType::Eof, ""),
        ];

        let mut parser = Parser::new(tokens);
        if let Some(stmt) = parser.statement() {
            match stmt {
                Stmt::If { condition, then_branch } => {
                    // Check condition: x == 5
                    match condition {
                        Expr::Binary { left, operator, right } => {
                            assert_eq!(*left, Expr::Identifier("x".to_string()));
                            assert_eq!(operator, TokenType::Equal);
                            assert_eq!(*right, Expr::Integer(5));
                        }
                        _ => panic!("Expected binary expression in condition"),
                    }
                    // Check then branch
                    assert_eq!(then_branch.len(), 1);
                    match &then_branch[0] {
                        Stmt::Return(Some(expr)) => {
                            assert_eq!(*expr, Expr::Integer(1));
                        }
                        _ => panic!("Expected return statement in then branch"),
                    }
                }
                _ => panic!("Expected if statement"),
            }
        } else {
            panic!("Failed to parse if statement");
        }
    }

    #[test]
    fn test_parse_print_statement() {
        // Test parsing: "println("Hello %d", 42);"
        let tokens = vec![
            create_token(TokenType::Println, "println"),
            create_token(TokenType::LeftParen, "("),
            create_token(TokenType::String("Hello %d".to_string()), "\"Hello %d\""),
            create_token(TokenType::Comma, ","),
            create_token(TokenType::Integer(42), "42"),
            create_token(TokenType::RightParen, ")"),
            create_token(TokenType::Semicolon, ";"),
            create_token(TokenType::Eof, ""),
        ];

        let mut parser = Parser::new(tokens);
        if let Some(stmt) = parser.statement() {
            match stmt {
                Stmt::PrintStmt { format_string, args } => {
                    assert_eq!(format_string, Expr::String("Hello %d".to_string()));
                    assert_eq!(args.len(), 1);
                    assert_eq!(args[0], Expr::Integer(42));
                }
                _ => panic!("Expected print statement"),
            }
        } else {
            panic!("Failed to parse print statement");
        }
    }

    #[test]
    fn test_parse_binary_expressions() {
        // Test parsing: "5 + 3 * 2"
        let tokens = vec![
            create_token(TokenType::Integer(5), "5"),
            create_token(TokenType::Plus, "+"),
            create_token(TokenType::Integer(3), "3"),
            create_token(TokenType::Multiply, "*"),
            create_token(TokenType::Integer(2), "2"),
            create_token(TokenType::Eof, ""),
        ];

        let mut parser = Parser::new(tokens);
        if let Some(expr) = parser.expression() {
            // Should be: 5 + (3 * 2) due to operator precedence
            match expr {
                Expr::Binary { left, operator, right } => {
                    assert_eq!(*left, Expr::Integer(5));
                    assert_eq!(operator, TokenType::Plus);
                    match *right {
                        Expr::Binary { ref left, operator: TokenType::Multiply, ref right } => {
                            assert_eq!(**left, Expr::Integer(3));
                            assert_eq!(**right, Expr::Integer(2));
                        }
                        _ => panic!("Expected multiplication in right side"),
                    }
                }
                _ => panic!("Expected binary expression"),
            }
        } else {
            panic!("Failed to parse binary expression");
        }
    }

    #[test]
    fn test_parse_function_calls() {
        // Test parsing: "func(42, 3.14)"
        let tokens = vec![
            create_token(TokenType::Identifier("func".to_string()), "func"),
            create_token(TokenType::LeftParen, "("),
            create_token(TokenType::Integer(42), "42"),
            create_token(TokenType::Comma, ","),
            create_token(TokenType::Float(3.14), "3.14"),
            create_token(TokenType::RightParen, ")"),
            create_token(TokenType::Eof, ""),
        ];

        let mut parser = Parser::new(tokens);
        if let Some(expr) = parser.expression() {
            match expr {
                Expr::Call { callee, arguments } => {
                    assert_eq!(*callee, Expr::Identifier("func".to_string()));
                    assert_eq!(arguments.len(), 2);
                    assert_eq!(arguments[0], Expr::Integer(42));
                    assert_eq!(arguments[1], Expr::Float(3.14));
                }
                _ => panic!("Expected function call expression"),
            }
        } else {
            panic!("Failed to parse function call");
        }
    }

    #[test]
    fn test_handle_invalid_syntax() {
        // Test parsing invalid syntax: missing semicolon
        let tokens = vec![
            create_token(TokenType::Return, "return"),
            create_token(TokenType::Integer(5), "5"),
            create_token(TokenType::Eof, ""),
        ];

        let mut parser = Parser::new(tokens);
        let result = parser.statement();
        assert!(result.is_none(), "Should fail to parse invalid syntax");
    }

    #[test]
    fn test_parse_unary_expressions() {
        // Test parsing: "-42"
        let tokens = vec![
            create_token(TokenType::Minus, "-"),
            create_token(TokenType::Integer(42), "42"),
            create_token(TokenType::Eof, ""),
        ];

        let mut parser = Parser::new(tokens);
        if let Some(expr) = parser.expression() {
            match expr {
                Expr::Unary { operator, operand } => {
                    assert_eq!(operator, TokenType::Minus);
                    assert_eq!(*operand, Expr::Integer(42));
                }
                _ => panic!("Expected unary expression"),
            }
        } else {
            panic!("Failed to parse unary expression");
        }
    }

    #[test]
    fn test_parse_assignment_expressions() {
        // Test parsing: "x = 42"
        let tokens = vec![
            create_token(TokenType::Identifier("x".to_string()), "x"),
            create_token(TokenType::Assign, "="),
            create_token(TokenType::Integer(42), "42"),
            create_token(TokenType::Eof, ""),
        ];

        let mut parser = Parser::new(tokens);
        if let Some(expr) = parser.expression() {
            match expr {
                Expr::Assignment { name, value } => {
                    assert_eq!(name, "x");
                    assert_eq!(*value, Expr::Integer(42));
                }
                _ => panic!("Expected assignment expression"),
            }
        } else {
            panic!("Failed to parse assignment expression");
        }
    }

    #[test]
    fn test_handle_empty_token_stream() {
        let tokens = vec![
            create_token(TokenType::Eof, ""),
        ];

        let mut parser = Parser::new(tokens);
        let result = parser.parse();
        assert!(result.is_empty(), "Should return empty vector for empty token stream");
    }
}
