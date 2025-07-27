// Core abstractions and traits
pub mod core;

// Shared utilities
pub mod utils;

// Code generation modules
pub mod generators;
mod codegen;

// Re-export commonly used items
pub use core::{
    create_target, parse_target_platform, CallingConvention,
    CodeEmitter, CodeEmitterWithComment, Emitter, Instruction,
    LinuxX64Target, MacOSX64Target, Operand,
    Register, Size, Target,
    TargetPlatform, WindowsX64Target
};

pub use utils::{InstructionFormatter, RegisterAllocator, StackManager};

pub use codegen::Codegen;
