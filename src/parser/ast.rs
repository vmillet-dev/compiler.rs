use crate::lexer::TokenType;
use crate::types::Type;

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
        type_arguments: Vec<Type>, // For generic function calls like func<T>(args)
    },
    Assignment {
        name: String,
        value: Box<Expr>,
    },
    TypeCast {
        expr: Box<Expr>,
        target_type: Type,
    },
}

#[derive(Debug, PartialEq)]
pub enum Stmt {
    ExprStmt(Expr),
    VarDecl {
        var_type: Type,
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
        return_type: Type,
        name: String,
        type_parameters: Vec<String>, // Generic type parameters like <T, U>
        parameters: Vec<Parameter>,   // Function parameters
        body: Vec<Stmt>,
    },
    PrintStmt {
        format_string: Expr,
        args: Vec<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub param_type: Type,
    pub is_mutable: bool,
}
