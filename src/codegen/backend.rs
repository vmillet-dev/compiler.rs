use crate::codegen::instruction::{Instruction, Operand, Register, Size};
use std::collections::HashMap;

pub trait CodegenBackend {
    fn emit_instruction(&mut self, instr: Instruction, operands: Vec<Operand>);
    
    fn emit_instruction_with_size(&mut self, instr: Instruction, size: Size, operands: Vec<Operand>);
    
    fn emit_instruction_with_size_and_comment(&mut self, instr: Instruction, size: Size, operands: Vec<Operand>, comment: Option<&str>);
    
    fn emit_comment(&mut self, comment: &str);
    
    fn emit_label(&mut self, label: &str);
    
    fn get_stack_offset(&self) -> i32;
    
    fn set_stack_offset(&mut self, offset: i32);
    
    fn get_locals(&self) -> &HashMap<String, i32>;
    
    fn get_locals_mut(&mut self) -> &mut HashMap<String, i32>;
    
    fn get_local_types(&self) -> &HashMap<String, crate::lexer::TokenType>;
    
    fn get_local_types_mut(&mut self) -> &mut HashMap<String, crate::lexer::TokenType>;
    
    fn get_output(&self) -> &str;
}

pub struct BackendUtils;

impl BackendUtils {
    pub fn calculate_stack_offset(var_type: &crate::lexer::TokenType, current_offset: i32) -> (usize, i32) {
        match var_type {
            crate::lexer::TokenType::Int => {
                let new_offset = current_offset - 4;
                (4, new_offset)
            },
            crate::lexer::TokenType::FloatType => {
                let new_offset = current_offset - 8;
                (8, new_offset)
            },
            crate::lexer::TokenType::CharType => {
                let new_offset = current_offset - 1;
                (1, new_offset)
            },
            _ => {
                let new_offset = current_offset - 8;
                (8, new_offset)
            }
        }
    }
    
    pub fn format_instruction(instr: &Instruction, operands: &[Operand]) -> String {
        let instr_str = format!("{:?}", instr).to_lowercase();
        if operands.is_empty() {
            instr_str
        } else {
            let operands_str = operands.iter()
                .map(|op| Self::format_operand(op))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{} {}", instr_str, operands_str)
        }
    }
    
    pub fn format_instruction_with_size(instr: &Instruction, size: &Size, operands: &[Operand]) -> String {
        let instr_str = format!("{:?}", instr).to_lowercase();
        let size_suffix = match size {
            Size::Byte => "b",
            Size::Word => "w", 
            Size::Dword => "d",
            Size::Qword => "q",
        };
        
        if operands.is_empty() {
            format!("{}{}", instr_str, size_suffix)
        } else {
            let operands_str = operands.iter()
                .map(|op| Self::format_operand(op))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}{} {}", instr_str, size_suffix, operands_str)
        }
    }
    
    pub fn format_operand(operand: &Operand) -> String {
        match operand {
            Operand::Register(reg) => format!("{:?}", reg).to_lowercase(),
            Operand::Immediate(val) => val.to_string(),
            Operand::Memory { base, offset } => {
                if *offset == 0 {
                    format!("[{}]", format!("{:?}", base).to_lowercase())
                } else if *offset > 0 {
                    format!("[{}+{}]", format!("{:?}", base).to_lowercase(), offset)
                } else {
                    format!("[{}{}]", format!("{:?}", base).to_lowercase(), offset)
                }
            },
            Operand::String(s) => s.clone(),
            Operand::Label(label) => label.clone(),
        }
    }
    
    pub fn generate_prologue() -> Vec<String> {
        vec![
            "push rbp".to_string(),
            "mov rbp, rsp".to_string(),
        ]
    }
    
    pub fn generate_epilogue() -> Vec<String> {
        vec![
            "mov rsp, rbp".to_string(),
            "pop rbp".to_string(),
            "ret".to_string(),
        ]
    }
}

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
    
    pub fn allocate(&mut self, var_name: String) -> Option<Register> {
        if let Some(reg) = self.available_registers.pop() {
            self.allocated_registers.insert(var_name, reg);
            Some(reg)
        } else {
            None // Need to spill to memory
        }
    }
    
    pub fn free(&mut self, var_name: &str) -> Option<Register> {
        if let Some(reg) = self.allocated_registers.remove(var_name) {
            self.available_registers.push(reg);
            Some(reg)
        } else {
            None
        }
    }
    
    pub fn get_register(&self, var_name: &str) -> Option<Register> {
        self.allocated_registers.get(var_name).copied()
    }
    
    pub fn is_available(&self, reg: Register) -> bool {
        self.available_registers.contains(&reg)
    }
}

impl Default for RegisterAllocator {
    fn default() -> Self {
        Self::new()
    }
}
