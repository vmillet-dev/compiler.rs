//! Code generation modules for different IR constructs

pub mod function;
pub mod instruction;
pub mod operation;
pub mod call;
pub mod value;

// These modules contain impl blocks for IrCodegen, not separate structs
// So we don't export specific types, just make the modules public