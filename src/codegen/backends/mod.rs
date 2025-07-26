//! Code generation backends
//! 
//! This module contains different backend implementations for code generation,
//! including IR-based and AST-based backends.

pub mod ir_backend;

pub use ir_backend::IrBackend;