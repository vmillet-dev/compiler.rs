//! Code generation module
//! 
//! This module provides a clean, well-organized code generation pipeline
//! for converting intermediate representation (IR) to assembly code.
//! 
//! ## Architecture
//! 
//! The module is organized into several sub-modules:
//! 
//! - **core/**: Core abstractions including instructions, emitters, and targets
//! - **backends/**: Different backend implementations (IR-based, AST-based)
//! - **generators/**: Specialized generators for functions, instructions, operations, and calls
//! - **utils/**: Utility modules for stack management, register allocation, and formatting
//! - **targets/**: Target-specific implementations for different platforms
//! 
//! ## Usage
//! 
//! ```rust,ignore
//! use compiler_minic::codegen::{IrBackend, TargetPlatform};
//! 
//! let mut backend = IrBackend::new_with_target(TargetPlatform::WindowsX64);
//! backend.set_ir_program(ir_program);
//! let assembly = backend.generate();
//! ```

pub mod core;
pub mod backends;
pub mod generators;
pub mod utils;
pub mod targets;

// Re-export core types for convenience
pub use core::{
    Instruction, Register, Operand, Size,
    Emitter, CodeEmitter, CodeEmitterWithComment,
    Target, TargetPlatform, CallingConvention,
};

// Re-export main backend implementations
pub use backends::IrBackend;

// Re-export utility types
pub use utils::{StackManager, RegisterAllocator, AssemblyFormatter};

// Re-export target factory functions
pub use core::target::{create_target, parse_target_platform};

// Re-export generators
pub use generators::{FunctionGenerator, InstructionGenerator, OperationGenerator, CallGenerator};