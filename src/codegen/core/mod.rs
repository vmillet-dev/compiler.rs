//! Core abstractions and traits for code generation

mod emitter;
mod instruction;
pub mod targets;

pub use emitter::{Emitter, CodeEmitter, CodeEmitterWithComment};
pub use instruction::{Instruction, Register, Operand, Size};
pub use targets::{
    Target, TargetPlatform, CallingConvention, 
    WindowsX64Target, LinuxX64Target, MacOSX64Target, 
    create_target, parse_target_platform
};