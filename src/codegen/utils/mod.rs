//! Shared utilities for code generation

mod register_allocator;
mod stack_manager;
mod formatter;

pub use register_allocator::RegisterAllocator;
pub use stack_manager::StackManager;
pub use formatter::InstructionFormatter;