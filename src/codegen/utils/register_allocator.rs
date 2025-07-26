//! Register allocation utilities
//! 
//! This module provides utilities for managing register allocation
//! and tracking register usage during code generation.

use crate::codegen::core::Register;
use std::collections::{HashMap, HashSet};

/// Simple register allocator for code generation
pub struct RegisterAllocator {
    /// Available general-purpose registers
    available_registers: Vec<Register>,
    
    /// Currently allocated registers mapped to variable names
    allocated_registers: HashMap<String, Register>,
    
    /// Set of registers currently in use
    used_registers: HashSet<Register>,
    
    /// Available floating-point registers
    available_float_registers: Vec<Register>,
    
    /// Currently allocated floating-point registers
    allocated_float_registers: HashMap<String, Register>,
    
    /// Set of floating-point registers currently in use
    used_float_registers: HashSet<Register>,
}

impl RegisterAllocator {
    /// Create a new register allocator
    pub fn new() -> Self {
        Self {
            available_registers: vec![
                Register::Rax,
                Register::Rcx,
                Register::Rdx,
                Register::R8,
                Register::R9,
            ],
            allocated_registers: HashMap::new(),
            used_registers: HashSet::new(),
            available_float_registers: vec![
                Register::Xmm0,
                Register::Xmm1,
                Register::Xmm2,
                Register::Xmm3,
            ],
            allocated_float_registers: HashMap::new(),
            used_float_registers: HashSet::new(),
        }
    }
    
    /// Reset the allocator for a new function
    pub fn reset(&mut self) {
        self.allocated_registers.clear();
        self.used_registers.clear();
        self.allocated_float_registers.clear();
        self.used_float_registers.clear();
    }
    
    /// Allocate a register for a variable
    pub fn allocate_register(&mut self, var_name: String) -> Option<Register> {
        // Find the first available register
        for &reg in &self.available_registers {
            if !self.used_registers.contains(&reg) {
                self.used_registers.insert(reg);
                self.allocated_registers.insert(var_name, reg);
                return Some(reg);
            }
        }
        None // No available registers
    }
    
    /// Allocate a floating-point register for a variable
    pub fn allocate_float_register(&mut self, var_name: String) -> Option<Register> {
        // Find the first available floating-point register
        for &reg in &self.available_float_registers {
            if !self.used_float_registers.contains(&reg) {
                self.used_float_registers.insert(reg);
                self.allocated_float_registers.insert(var_name, reg);
                return Some(reg);
            }
        }
        None // No available floating-point registers
    }
    
    /// Free a register allocated to a variable
    pub fn free_register(&mut self, var_name: &str) -> Option<Register> {
        if let Some(reg) = self.allocated_registers.remove(var_name) {
            self.used_registers.remove(&reg);
            Some(reg)
        } else {
            None
        }
    }
    
    /// Free a floating-point register allocated to a variable
    pub fn free_float_register(&mut self, var_name: &str) -> Option<Register> {
        if let Some(reg) = self.allocated_float_registers.remove(var_name) {
            self.used_float_registers.remove(&reg);
            Some(reg)
        } else {
            None
        }
    }
    
    /// Get the register allocated to a variable
    pub fn get_register(&self, var_name: &str) -> Option<Register> {
        self.allocated_registers.get(var_name).copied()
    }
    
    /// Get the floating-point register allocated to a variable
    pub fn get_float_register(&self, var_name: &str) -> Option<Register> {
        self.allocated_float_registers.get(var_name).copied()
    }
    
    /// Check if a register is available
    pub fn is_register_available(&self, reg: Register) -> bool {
        !self.used_registers.contains(&reg)
    }
    
    /// Check if a floating-point register is available
    pub fn is_float_register_available(&self, reg: Register) -> bool {
        !self.used_float_registers.contains(&reg)
    }
    
    /// Get a temporary register (doesn't allocate permanently)
    pub fn get_temp_register(&self) -> Option<Register> {
        for &reg in &self.available_registers {
            if !self.used_registers.contains(&reg) {
                return Some(reg);
            }
        }
        None
    }
    
    /// Get a temporary floating-point register (doesn't allocate permanently)
    pub fn get_temp_float_register(&self) -> Option<Register> {
        for &reg in &self.available_float_registers {
            if !self.used_float_registers.contains(&reg) {
                return Some(reg);
            }
        }
        None
    }
    
    /// Mark a register as used (for system registers like RSP, RBP)
    pub fn mark_register_used(&mut self, reg: Register) {
        self.used_registers.insert(reg);
    }
    
    /// Mark a register as available again
    pub fn mark_register_available(&mut self, reg: Register) {
        self.used_registers.remove(&reg);
    }
    
    /// Get the number of available registers
    pub fn available_register_count(&self) -> usize {
        self.available_registers.len() - self.used_registers.len()
    }
    
    /// Get the number of available floating-point registers
    pub fn available_float_register_count(&self) -> usize {
        self.available_float_registers.len() - self.used_float_registers.len()
    }
    
    /// Get all currently allocated registers
    pub fn allocated_registers(&self) -> &HashMap<String, Register> {
        &self.allocated_registers
    }
    
    /// Get all currently allocated floating-point registers
    pub fn allocated_float_registers(&self) -> &HashMap<String, Register> {
        &self.allocated_float_registers
    }
}

impl Default for RegisterAllocator {
    fn default() -> Self {
        Self::new()
    }
}