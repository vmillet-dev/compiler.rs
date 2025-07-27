use std::collections::HashMap;
use crate::codegen::core::Register;

/// Simple register allocator for managing register assignments
pub struct RegisterAllocator {
    available_registers: Vec<Register>,
    allocated_registers: HashMap<String, Register>,
}

impl RegisterAllocator {
    pub fn new() -> Self {
        Self {
            available_registers: vec![
                Register::Rax, Register::Rcx, Register::Rdx, Register::R8, Register::R9,
            ],
            allocated_registers: HashMap::new(),
        }
    }
    
    /// Allocate a register for a variable
    pub fn allocate(&mut self, var_name: String) -> Option<Register> {
        if let Some(reg) = self.available_registers.pop() {
            self.allocated_registers.insert(var_name, reg);
            Some(reg)
        } else {
            None // Need to spill to memory
        }
    }
    
    /// Free a register from a variable
    pub fn free(&mut self, var_name: &str) -> Option<Register> {
        if let Some(reg) = self.allocated_registers.remove(var_name) {
            self.available_registers.push(reg);
            Some(reg)
        } else {
            None
        }
    }
    
    /// Get the register assigned to a variable
    pub fn get_register(&self, var_name: &str) -> Option<Register> {
        self.allocated_registers.get(var_name).copied()
    }
    
    /// Check if a register is available
    pub fn is_available(&self, reg: Register) -> bool {
        self.available_registers.contains(&reg)
    }
    
    /// Get all allocated registers
    pub fn allocated_registers(&self) -> &HashMap<String, Register> {
        &self.allocated_registers
    }
    
    /// Get all available registers
    pub fn available_registers(&self) -> &[Register] {
        &self.available_registers
    }
}

impl Default for RegisterAllocator {
    fn default() -> Self {
        Self::new()
    }
}