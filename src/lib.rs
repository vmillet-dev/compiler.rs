use crate::error::CompilerError;

pub mod lexer;
pub mod error;

pub type Result<T> = std::result::Result<T, CompilerError>;