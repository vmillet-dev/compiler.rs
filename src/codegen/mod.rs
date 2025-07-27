// Core abstractions and traits
pub mod core;

// Shared utilities
pub mod utils;

// Code generation modules
pub mod generators;
mod codegen;
pub mod targets;

// Re-export commonly used items
pub use core::{CodeEmitter, CodeEmitterWithComment, Emitter, Instruction, Operand, Register, Size};

pub use utils::{InstructionFormatter, RegisterAllocator, StackManager};

pub use codegen::Codegen;
