mod instruction;
mod emitter;
mod ir_codegen;
mod backend;

pub use ir_codegen::IrCodegen;
pub use instruction::{Instruction, Register, Operand, Size};
pub use emitter::{Emitter, CodeEmitter, CodeEmitterWithComment};
pub use backend::{BackendUtils, RegisterAllocator, IrBackend};
