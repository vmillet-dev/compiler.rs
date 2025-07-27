use crate::codegen::core::{Instruction, Operand, Size};

/// Utility for formatting assembly instructions
pub struct InstructionFormatter;

impl InstructionFormatter {
    /// Format an instruction with operands
    pub fn format_instruction(instr: &Instruction, operands: &[Operand]) -> String {
        let instr_str = instr.to_string();
        if operands.is_empty() {
            instr_str.to_string()
        } else {
            let operands_str = operands.iter()
                .map(|op| Self::format_operand(op))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{} {}", instr_str, operands_str)
        }
    }
    
    /// Format an instruction with size and operands
    pub fn format_instruction_with_size(instr: &Instruction, size: &Size, operands: &[Operand]) -> String {
        let instr_str = instr.to_string();
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
    
    /// Format a single operand
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
    
    /// Generate function prologue
    pub fn generate_prologue() -> Vec<String> {
        vec![
            "push rbp".to_string(),
            "mov rbp, rsp".to_string(),
        ]
    }
    
    /// Generate function epilogue
    pub fn generate_epilogue() -> Vec<String> {
        vec![
            "mov rsp, rbp".to_string(),
            "pop rbp".to_string(),
            "ret".to_string(),
        ]
    }
}