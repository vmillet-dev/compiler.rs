//! Specialized code generators
//! 
//! This module contains specialized generators for different aspects
//! of code generation like functions, instructions, operations, and calls.

pub mod function;
pub mod instruction;
pub mod operation;
pub mod call;

pub use function::FunctionGenerator;
pub use instruction::InstructionGenerator;
pub use operation::OperationGenerator;
pub use call::CallGenerator;