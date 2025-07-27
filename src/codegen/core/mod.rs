//! Core abstractions and traits for code generation

mod emitter;
mod instruction;

pub use emitter::{CodeEmitter, CodeEmitterWithComment, Emitter};
pub use instruction::{Instruction, Operand, Register, Size};
// pub use crate::codegen::targets::{
//     create_target, parse_target_platform, CallingConvention,
//     LinuxX64Target, MacOSX64Target, Target,
//     TargetPlatform, WindowsX64Target
// };