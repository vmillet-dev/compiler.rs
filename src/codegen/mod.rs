// Core abstractions and traits
pub mod core;

// Backend implementations
pub mod backend;

// Shared utilities
pub mod utils;

// Code generation modules
pub mod generators;

// Re-export commonly used items
pub use core::{
    Emitter, CodeEmitter, CodeEmitterWithComment,
    Instruction, Register, Operand, Size,
    Target, TargetPlatform, CallingConvention,
    WindowsX64Target, LinuxX64Target, MacOSX64Target,
    create_target, parse_target_platform
};

pub use backend::IrBackend;
pub use utils::{RegisterAllocator, StackManager, InstructionFormatter};

// For backward compatibility, re-export IrBackend as IrCodegen
pub use backend::IrBackend as IrCodegen;
