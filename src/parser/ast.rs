use crate::lexer::TokenType;

// AST definitions
#[derive(Debug, Clone, PartialEq)]
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
    Unary {
        operator: TokenType,
        operand: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        arguments: Vec<Expr>,
    },
    Assignment {
        name: String,
        value: Box<Expr>,
    },
}

#[derive(Debug, PartialEq)]
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
    PrintStmt {
        format_string: Expr,
        args: Vec<Expr>,
    },
}