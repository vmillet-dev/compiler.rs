pub mod symbol_table;
pub mod lifetime_simple;
pub mod memory_manager;

pub use symbol_table::{SymbolTable, Symbol, Visibility, Mutability};
pub use lifetime_simple::{LifetimeAnalyzer, Lifetime, LifetimeConstraint};
pub use memory_manager::{MemoryLayout, StackFrameManager, MemorySafetyChecker, MemorySafetyWarning, MemorySafetySeverity, AllocationStrategy};
