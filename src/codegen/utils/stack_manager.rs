//! Stack management utilities
//! 
//! This module provides utilities for managing stack layout,
//! variable allocation, and stack frame calculations.

use crate::lexer::TokenType;
use std::collections::HashMap;

/// Manages stack layout and variable allocation
pub struct StackManager {
    /// Current stack offset from base pointer
    stack_offset: i32,
    
    /// Map of local variable names to their stack offsets
    locals: HashMap<String, i32>,
    
    /// Map of local variable names to their types
    local_types: HashMap<String, TokenType>,
    
    /// Map of temporary variables to their stack locations
    temp_locations: HashMap<usize, i32>,
    
    /// Stack alignment requirement
    alignment: usize,
}

impl StackManager {
    /// Create a new stack manager with default alignment
    pub fn new() -> Self {
        Self::new_with_alignment(16) // Default to 16-byte alignment
    }
    
    /// Create a new stack manager with specified alignment
    pub fn new_with_alignment(alignment: usize) -> Self {
        Self {
            stack_offset: 0,
            locals: HashMap::new(),
            local_types: HashMap::new(),
            temp_locations: HashMap::new(),
            alignment,
        }
    }
    
    /// Reset the stack manager for a new function
    pub fn reset(&mut self) {
        self.stack_offset = 0;
        self.locals.clear();
        self.local_types.clear();
        self.temp_locations.clear();
    }
    
    /// Allocate space for a local variable
    pub fn allocate_local(&mut self, name: String, var_type: TokenType) -> i32 {
        let (_size, new_offset) = self.calculate_stack_offset(&var_type, self.stack_offset);
        self.stack_offset = new_offset;
        self.locals.insert(name.clone(), new_offset);
        self.local_types.insert(name, var_type);
        new_offset
    }
    
    /// Allocate space for a temporary variable
    pub fn allocate_temp(&mut self, temp_id: usize, var_type: TokenType) -> i32 {
        let (_, new_offset) = self.calculate_stack_offset(&var_type, self.stack_offset);
        self.stack_offset = new_offset;
        self.temp_locations.insert(temp_id, new_offset);
        new_offset
    }
    
    /// Get the stack offset for a local variable
    pub fn get_local_offset(&self, name: &str) -> Option<i32> {
        self.locals.get(name).copied()
    }
    
    /// Get the stack offset for a temporary variable
    pub fn get_temp_offset(&self, temp_id: usize) -> Option<i32> {
        self.temp_locations.get(&temp_id).copied()
    }
    
    /// Get the type of a local variable
    pub fn get_local_type(&self, name: &str) -> Option<&TokenType> {
        self.local_types.get(name)
    }
    
    /// Get the current stack offset
    pub fn current_offset(&self) -> i32 {
        self.stack_offset
    }
    
    /// Calculate the total stack space needed
    pub fn total_stack_space(&self) -> usize {
        let space = (-self.stack_offset) as usize;
        // Align to the required boundary
        (space + self.alignment - 1) & !(self.alignment - 1)
    }
    
    /// Calculate stack offset and size for a given type
    fn calculate_stack_offset(&self, var_type: &TokenType, current_offset: i32) -> (usize, i32) {
        let size = self.type_size(var_type);
        let aligned_size = self.align_size(size);
        let new_offset = current_offset - aligned_size as i32;
        (size, new_offset)
    }
    
    /// Get the size of a type in bytes
    fn type_size(&self, var_type: &TokenType) -> usize {
        match var_type {
            TokenType::Int => 4,
            TokenType::FloatType => 8,
            TokenType::CharType => 1,
            TokenType::Void => 0,
            _ => 8, // Default to pointer size
        }
    }
    
    /// Align a size to the stack alignment boundary
    fn align_size(&self, size: usize) -> usize {
        match size {
            1 => 1, // char doesn't need alignment
            2 => 2, // short aligns to 2 bytes
            4 => 4, // int aligns to 4 bytes
            8 => 8, // long/double aligns to 8 bytes
            _ => (size + self.alignment - 1) & !(self.alignment - 1),
        }
    }
    
    /// Get all local variables and their offsets
    pub fn locals(&self) -> &HashMap<String, i32> {
        &self.locals
    }
    
    /// Get all local variable types
    pub fn local_types(&self) -> &HashMap<String, TokenType> {
        &self.local_types
    }
    
    /// Get all temporary variable locations
    pub fn temp_locations(&self) -> &HashMap<usize, i32> {
        &self.temp_locations
    }
}

impl Default for StackManager {
    fn default() -> Self {
        Self::new()
    }
}