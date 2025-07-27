use std::collections::HashMap;
use crate::lexer::TokenType;

/// Manages stack layout and variable offsets
pub struct StackManager {
    stack_offset: i32,
    locals: HashMap<String, i32>,
    local_types: HashMap<String, TokenType>,
}

impl StackManager {
    pub fn new() -> Self {
        Self {
            stack_offset: 0,
            locals: HashMap::new(),
            local_types: HashMap::new(),
        }
    }
    
    /// Calculate stack offset for a variable type
    pub fn calculate_stack_offset(var_type: &TokenType, current_offset: i32) -> (usize, i32) {
        match var_type {
            TokenType::Int => {
                let new_offset = current_offset - 4;
                (4, new_offset)
            },
            TokenType::FloatType => {
                let new_offset = current_offset - 8;
                (8, new_offset)
            },
            TokenType::CharType => {
                let new_offset = current_offset - 1;
                (1, new_offset)
            },
            _ => {
                let new_offset = current_offset - 8;
                (8, new_offset)
            }
        }
    }
    
    /// Allocate space for a variable on the stack
    pub fn allocate_variable(&mut self, name: String, var_type: TokenType) -> i32 {
        let (_, new_offset) = Self::calculate_stack_offset(&var_type, self.stack_offset);
        self.stack_offset = new_offset;
        self.locals.insert(name.clone(), new_offset);
        self.local_types.insert(name, var_type);
        new_offset
    }
    
    /// Get the stack offset for a variable
    pub fn get_variable_offset(&self, name: &str) -> Option<i32> {
        self.locals.get(name).copied()
    }
    
    /// Get the type of a variable
    pub fn get_variable_type(&self, name: &str) -> Option<&TokenType> {
        self.local_types.get(name)
    }
    
    /// Get current stack offset
    pub fn current_offset(&self) -> i32 {
        self.stack_offset
    }
    
    /// Set stack offset
    pub fn set_offset(&mut self, offset: i32) {
        self.stack_offset = offset;
    }
    
    /// Get all locals
    pub fn locals(&self) -> &HashMap<String, i32> {
        &self.locals
    }
    
    /// Get all local types
    pub fn local_types(&self) -> &HashMap<String, TokenType> {
        &self.local_types
    }
    
    /// Clear all variables (for new function)
    pub fn clear(&mut self) {
        self.stack_offset = 0;
        self.locals.clear();
        self.local_types.clear();
    }
}

impl Default for StackManager {
    fn default() -> Self {
        Self::new()
    }
}