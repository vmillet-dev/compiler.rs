use crate::lexer::TokenType;

// AST definitions
#[derive(Debug)]
pub enum Expr {
    Integer(i64),
    Float(f64),
    Char(char),
    String(String),
    Identifier(String),
    Binary {
        left: Box<Expr>,
        operator: TokenType,
        right: Box<Expr>,
    },
}

#[derive(Debug)]
pub enum Stmt {
    ExprStmt(Expr),
    VarDecl {
        var_type: TokenType,
        name: String,
        initializer: Option<Expr>,
    },
    Return(Option<Expr>),
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
    },
    Block(Vec<Stmt>),
    Function {
        return_type: TokenType,
        name: String,
        body: Vec<Stmt>,
    },
}