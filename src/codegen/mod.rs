mod instruction;
mod emitter;
mod analyzer;
mod expression;
mod statement;
mod codegen;
mod ir_codegen;
mod backend;
mod direct_backend;
mod ir_backend;
mod target;
mod calling_convention;

pub use codegen::Codegen;
pub use ir_codegen::IrCodegen;
pub use instruction::{Instruction, Register, Operand, Size};
pub use emitter::{Emitter, CodeEmitter, CodeEmitterWithComment};
pub use analyzer::AstAnalyzer;
pub use expression::ExpressionGenerator;
pub use statement::StatementGenerator;
pub use backend::{CodegenBackend, BackendUtils, RegisterAllocator};
pub use direct_backend::DirectBackend;
pub use ir_backend::IrBackend;
pub use target::{TargetArchitecture, RegisterAllocator as TargetRegisterAllocator, CallingConvention, CodeGenerator};
pub use target::x86_64_windows::{X86_64Windows, X86RegisterAllocator, WindowsX64CallingConvention};
pub use calling_convention::{FunctionCallGenerator, CallingConvention as CallConv};
