//! Assembly formatting utilities
//! 
//! This module provides utilities for formatting assembly code,
//! instructions, and operands in a consistent manner.

use crate::codegen::core::{Instruction, Operand, Size};

/// Utility for formatting assembly code
pub struct AssemblyFormatter;

impl AssemblyFormatter {
    /// Format an instruction with operands
    pub fn format_instruction(instr: &Instruction, operands: &[Operand]) -> String {
        let instr_str = instr.as_str();
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
    
    /// Format an instruction with explicit size specification
    pub fn format_instruction_with_size(instr: &Instruction, size: &Size, operands: &[Operand]) -> String {
        let instr_str = instr.as_str();
        let operands_str = operands.iter()
            .enumerate()
            .map(|(i, op)| {
                // Add size specifier to the first memory operand
                if i == 0 && matches!(op, Operand::Memory { .. }) {
                    format!("{} {}", size.nasm_specifier(), Self::format_operand(op))
                } else {
                    Self::format_operand(op)
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        
        if operands.is_empty() {
            instr_str.to_string()
        } else {
            format!("{} {}", instr_str, operands_str)
        }
    }
    
    /// Format a single operand
    pub fn format_operand(operand: &Operand) -> String {
        match operand {
            Operand::Register(reg) => reg.as_str().to_string(),
            Operand::Immediate(val) => val.to_string(),
            Operand::Memory { base, offset } => {
                if *offset == 0 {
                    format!("[{}]", base.as_str())
                } else if *offset > 0 {
                    format!("[{}+{}]", base.as_str(), offset)
                } else {
                    format!("[{}{}]", base.as_str(), offset)
                }
            },
            Operand::Label(label) => label.clone(),
            Operand::String(s) => s.clone(),
        }
    }
    
    /// Format a function prologue with comments
    pub fn format_prologue_with_comments() -> Vec<(String, String)> {
        vec![
            ("push rbp".to_string(), "save caller's frame pointer".to_string()),
            ("mov rbp, rsp".to_string(), "set up new frame pointer".to_string()),
        ]
    }
    
    /// Format a function epilogue with comments
    pub fn format_epilogue_with_comments() -> Vec<(String, String)> {
        vec![
            ("mov rsp, rbp".to_string(), "restore stack pointer".to_string()),
            ("pop rbp".to_string(), "restore caller's frame pointer".to_string()),
            ("ret".to_string(), "return to caller".to_string()),
        ]
    }
    
    /// Format a section header
    pub fn format_section_header(title: &str) -> String {
        let separator = "=".repeat(60);
        format!("; {}\n; {}\n; {}", separator, title, separator)
    }
    
    /// Format a subsection header
    pub fn format_subsection_header(title: &str) -> String {
        let separator = "-".repeat(40);
        format!("; {}\n; {}\n; {}", separator, title, separator)
    }
    
    /// Format an instruction with proper indentation and optional comment
    pub fn format_instruction_line(instr: &str, comment: Option<&str>) -> String {
        if let Some(comment) = comment {
            format!("    {:40} ; {}", instr, comment)
        } else {
            format!("    {}", instr)
        }
    }
    
    /// Format a label
    pub fn format_label(label: &str) -> String {
        format!("{}:", label)
    }
    
    /// Format a comment line
    pub fn format_comment(comment: &str) -> String {
        format!("; {}", comment)
    }
    
    /// Format a data declaration
    pub fn format_data_declaration(label: &str, data_type: &str, value: &str) -> String {
        format!("    {}: {} {}", label, data_type, value)
    }
    
    /// Format a string literal declaration
    pub fn format_string_literal(label: &str, content: &str) -> String {
        let escaped_content = content
            .replace('\\', "\\\\")
            .replace('\n', "\\n")
            .replace('\t', "\\t")
            .replace('\r', "\\r")
            .replace('\"', "\\\"");
        format!("    {}: db \"{}\", 0", label, escaped_content)
    }
}