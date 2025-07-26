mod instruction;
mod emitter;
mod ir_codegen;
mod backend;
mod target;

pub use ir_codegen::IrCodegen;
pub use instruction::{Instruction, Register, Operand, Size};
pub use emitter::{Emitter, CodeEmitter, CodeEmitterWithComment};
pub use backend::{BackendUtils, RegisterAllocator, IrBackend};
pub use target::{Target, TargetPlatform, CallingConvention, WindowsX64Target, LinuxX64Target, MacOSX64Target, create_target, parse_target_platform};
