//! Assembly code emission traits and implementations
//! 
//! This module provides traits for emitting assembly code with proper
//! formatting, comments, and instruction handling.

use super::instruction::{Instruction, Operand, Size};

/// Basic trait for emitting lines of assembly code
pub trait Emitter {
    /// Emit a single line of assembly code
    fn emit_line(&mut self, line: &str);
    
    /// Emit a comment line
    fn emit_comment(&mut self, comment: &str);
    
    /// Emit a line with an optional comment
    fn emit_line_with_comment(&mut self, line: &str, comment: Option<&str>) {
        if let Some(comment) = comment {
            self.emit_line(&format!("{:40} ; {}", line, comment));
        } else {
            self.emit_line(line);
        }
    }
    
    /// Emit a section header with decorative formatting
    fn emit_section_header(&mut self, title: &str) {
        let separator = "=".repeat(60);
        self.emit_line(&format!("; {}", separator));
        self.emit_line(&format!("; {}", title));
        self.emit_line(&format!("; {}", separator));
    }
    
    /// Emit a subsection header with lighter formatting
    fn emit_subsection_header(&mut self, title: &str) {
        let separator = "-".repeat(40);
        self.emit_line(&format!("; {}", separator));
        self.emit_line(&format!("; {}", title));
        self.emit_line(&format!("; {}", separator));
    }
}

/// Trait for emitting assembly instructions
pub trait CodeEmitter: Emitter {
    /// Emit an instruction with operands
    fn emit_instruction(&mut self, instruction: Instruction, operands: Vec<Operand>);
    
    /// Emit an instruction with explicit size specification
    fn emit_instruction_with_size(&mut self, instruction: Instruction, size: Size, operands: Vec<Operand>);
    
    /// Emit a label
    fn emit_label(&mut self, label: &str) {
        self.emit_line(&format!("{}:", label));
    }
}

/// Trait for emitting instructions with comments
pub trait CodeEmitterWithComment: Emitter {
    /// Emit an instruction with an optional comment
    fn emit_instruction_with_comment(&mut self, instruction: Instruction, operands: Vec<Operand>, comment: Option<&str>);
    
    /// Emit an instruction with size and optional comment
    fn emit_instruction_with_size_and_comment(&mut self, instruction: Instruction, size: Size, operands: Vec<Operand>, comment: Option<&str>);
}

/// Default implementation of CodeEmitter for any type that implements Emitter
impl<T: Emitter> CodeEmitter for T {
    fn emit_instruction(&mut self, instruction: Instruction, operands: Vec<Operand>) {
        let instr_str = instruction.as_str();
        if operands.is_empty() {
            self.emit_line(&format!("    {:8}", instr_str));
        } else {
            let operands_str = operands.iter()
                .map(|op| op.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            self.emit_line(&format!("    {:8} {}", instr_str, operands_str));
        }
    }

    fn emit_instruction_with_size(&mut self, instruction: Instruction, size: Size, operands: Vec<Operand>) {
        let instr_str = instruction.as_str();
        let operands_str = operands.iter()
            .enumerate()
            .map(|(i, op)| {
                // Add size specifier to the first memory operand
                if i == 0 && matches!(op, Operand::Memory { .. }) {
                    format!("{} {}", size.nasm_specifier(), op)
                } else {
                    op.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        self.emit_line(&format!("    {:8} {}", instr_str, operands_str));
    }
}

/// Default implementation of CodeEmitterWithComment for any type that implements Emitter
impl<T: Emitter> CodeEmitterWithComment for T {
    fn emit_instruction_with_comment(&mut self, instruction: Instruction, operands: Vec<Operand>, comment: Option<&str>) {
        let instr_str = instruction.as_str();
        if operands.is_empty() {
            if let Some(comment) = comment {
                self.emit_line(&format!("    {:8}                    ; {}", instr_str, comment));
            } else {
                self.emit_line(&format!("    {:8}", instr_str));
            }
        } else {
            let operands_str = operands.iter()
                .map(|op| op.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            if let Some(comment) = comment {
                self.emit_line(&format!("    {:8} {:20} ; {}", instr_str, operands_str, comment));
            } else {
                self.emit_line(&format!("    {:8} {}", instr_str, operands_str));
            }
        }
    }

    fn emit_instruction_with_size_and_comment(&mut self, instruction: Instruction, size: Size, operands: Vec<Operand>, comment: Option<&str>) {
        let instr_str = instruction.as_str();
        let operands_str = operands.iter()
            .enumerate()
            .map(|(i, op)| {
                // Add size specifier to the first memory operand
                if i == 0 && matches!(op, Operand::Memory { .. }) {
                    format!("{} {}", size.nasm_specifier(), op)
                } else {
                    op.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        if let Some(comment) = comment {
            self.emit_line(&format!("    {:8} {:20} ; {}", instr_str, operands_str, comment));
        } else {
            self.emit_line(&format!("    {:8} {}", instr_str, operands_str));
        }
    }
}