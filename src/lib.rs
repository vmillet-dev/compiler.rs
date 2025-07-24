use crate::error::CompilerError;

pub mod lexer;
pub mod error;
pub mod types;
pub mod parser;
pub mod semantic;

pub mod ir;

pub mod codegen;

pub type Result<T> = std::result::Result<T, CompilerError>;
