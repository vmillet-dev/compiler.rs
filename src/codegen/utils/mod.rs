//! Utility modules for code generation
//! 
//! This module contains utility functions and helpers used throughout
//! the code generation pipeline.

pub mod stack_manager;
pub mod register_allocator;
pub mod formatter;

pub use stack_manager::StackManager;
pub use register_allocator::RegisterAllocator;
pub use formatter::AssemblyFormatter;