mod instruction;
mod emitter;
mod ir_codegen;
mod backend;
mod ir_backend;

pub use ir_codegen::IrCodegen;
pub use instruction::{Instruction, Register, Operand, Size};
pub use emitter::{Emitter, CodeEmitter, CodeEmitterWithComment};
pub use backend::{CodegenBackend, BackendUtils, RegisterAllocator};
pub use ir_backend::IrBackend;
