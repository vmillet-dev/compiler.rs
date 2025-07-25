use super::backend::{CodegenBackend, BackendUtils, RegisterAllocator};
use super::instruction::{Instruction, Operand, Size};
use crate::lexer::TokenType;
use std::collections::HashMap;

pub struct DirectBackend {
    output: String,
    stack_offset: i32,
    locals: HashMap<String, i32>,
    local_types: HashMap<String, TokenType>,
    _register_allocator: RegisterAllocator,
}

impl DirectBackend {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            stack_offset: 0,
            locals: HashMap::new(),
            local_types: HashMap::new(),
            _register_allocator: RegisterAllocator::new(),
        }
    }
    
    pub fn generate_program(&mut self, functions: &[String]) -> String {
        let mut program = String::new();
        
        program.push_str("section .data\n");
        program.push_str("    format_int db '%d', 0\n");
        program.push_str("    format_float db '%.2f', 0\n");
        program.push_str("    format_char db '%c', 0\n");
        program.push_str("    newline db 10, 0\n\n");
        
        program.push_str("section .text\n");
        program.push_str("    global _start\n");
        program.push_str("    extern printf\n");
        program.push_str("    extern exit\n\n");
        
        for function in functions {
            program.push_str(function);
            program.push('\n');
        }
        
        program.push_str(&self.output);
        
        program
    }
}

impl CodegenBackend for DirectBackend {
    fn emit_instruction(&mut self, instr: Instruction, operands: Vec<Operand>) {
        let formatted = BackendUtils::format_instruction(&instr, &operands);
        self.output.push_str(&format!("    {}\n", formatted));
    }
    
    fn emit_instruction_with_size(&mut self, instr: Instruction, size: Size, operands: Vec<Operand>) {
        let formatted = BackendUtils::format_instruction_with_size(&instr, &size, &operands);
        self.output.push_str(&format!("    {}\n", formatted));
    }
    
    fn emit_instruction_with_size_and_comment(&mut self, instr: Instruction, size: Size, operands: Vec<Operand>, comment: Option<&str>) {
        let formatted = BackendUtils::format_instruction_with_size(&instr, &size, &operands);
        if let Some(comment) = comment {
            self.output.push_str(&format!("    {} ; {}\n", formatted, comment));
        } else {
            self.output.push_str(&format!("    {}\n", formatted));
        }
    }
    
    fn emit_comment(&mut self, comment: &str) {
        self.output.push_str(&format!("    ; {}\n", comment));
    }
    
    fn emit_label(&mut self, label: &str) {
        self.output.push_str(&format!("{}:\n", label));
    }
    
    fn get_stack_offset(&self) -> i32 {
        self.stack_offset
    }
    
    fn set_stack_offset(&mut self, offset: i32) {
        self.stack_offset = offset;
    }
    
    fn get_locals(&self) -> &HashMap<String, i32> {
        &self.locals
    }
    
    fn get_locals_mut(&mut self) -> &mut HashMap<String, i32> {
        &mut self.locals
    }
    
    fn get_local_types(&self) -> &HashMap<String, TokenType> {
        &self.local_types
    }
    
    fn get_local_types_mut(&mut self) -> &mut HashMap<String, TokenType> {
        &mut self.local_types
    }
    
    fn get_output(&self) -> &str {
        &self.output
    }
}

impl Default for DirectBackend {
    fn default() -> Self {
        Self::new()
    }
}
