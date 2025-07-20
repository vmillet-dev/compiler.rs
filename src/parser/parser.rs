use crate::lexer::{Token, TokenType};
use crate::parser::ast::{Expr, Stmt};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        while !self.is_at_end() {
            if let Some(func) = self.function() {
                stmts.push(func);
            } else {
                self.advance();
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
                self.advance();
            }
        }

        self.consume(TokenType::RightBrace)?;

        Some(Stmt::Function {
            return_type,
            name,
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

            // Le premier argument doit être la chaîne de format
            let format_string_expr = self.expression()?;
            let format_string = match format_string_expr {
                Expr::String(_) => format_string_expr,
                _ => {
                    eprintln!("Erreur d'analyse: Le premier argument de printf doit être une chaîne de caractères à {}:{}", self.peek().line, self.peek().column);
                    return None; // Ou renvoyer une erreur plus spécifique
                }
            };

            let mut args = Vec::new();
            // Si le prochain token n'est pas ')' (fin des arguments), on s'attend à une virgule et d'autres arguments
            while !self.check(&TokenType::RightParen) && !self.is_at_end() {
                if self.check(&TokenType::Comma) { // Si on a une virgule, on la consomme
                    self.advance();
                } else if !args.is_empty() { // Si ce n'est pas la première itération et pas de virgule, c'est une erreur
                    eprintln!("Erreur d'analyse: Virgule attendue entre les arguments de printf à {}:{}", self.peek().line, self.peek().column);
                    return None;
                }
                args.push(self.expression()?);
            }

            self.consume(TokenType::RightParen)?;
            self.consume(TokenType::Semicolon)?;
            return Some(Stmt::PrintStmt { format_string, args });
        }

        if let Some(var_type) = self.match_any_type() {
            let name = self.consume_identifier()?;
            let initializer = if self.match_token(&TokenType::Assign) {
                Some(self.expression()?)
            } else {
                None
            };
            self.consume(TokenType::Semicolon)?;
            return Some(Stmt::VarDecl { var_type, name, initializer });
        }

        let expr = self.expression()?;
        self.consume(TokenType::Semicolon)?;
        Some(Stmt::ExprStmt(expr))
    }

    fn expression(&mut self) -> Option<Expr> {
        self.equality()
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
        // Ajout de l'opérateur unaire '!' pour la négation logique
        if let Some(op) = self.match_any(&[TokenType::LogicalNot, TokenType::Minus]) {
            let right = self.unary()?; // Récursif pour gérer !!x ou -(-x)
            return Some(Expr::Binary {
                left: Box::new(Expr::Integer(0)), // Un placeholder, car l'opérateur unaire n'a pas de 'gauche'
                operator: op,
                right: Box::new(right),
            });
        }
        self.call() // Passer à la gestion des appels de fonction
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
        &self.tokens[self.current]
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }
}
