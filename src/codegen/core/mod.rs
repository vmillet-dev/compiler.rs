//! Core abstractions and traits for code generation
//! 
//! This module contains the fundamental building blocks used throughout
//! the code generation pipeline, including instruction definitions,
//! emitter traits, and target abstractions.

pub mod instruction;
pub mod emitter;
pub mod target;

// Re-export core types for convenience
pub use instruction::{Instruction, Register, Operand, Size};
pub use emitter::{Emitter, CodeEmitter, CodeEmitterWithComment};
pub use target::{Target, TargetPlatform, CallingConvention, create_target, parse_target_platform};