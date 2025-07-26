use super::instruction::{Instruction, Operand, Size};

pub trait Emitter {
    fn emit_line(&mut self, line: &str);
    fn emit_comment(&mut self, comment: &str);
    fn emit_line_with_comment(&mut self, line: &str, comment: Option<&str>) {
        if let Some(comment) = comment {
            self.emit_line(&format!("{:40} ; {}", line, comment));
        } else {
            self.emit_line(line);
        }
    }
}

pub trait CodeEmitter: Emitter {
    fn emit_instruction(&mut self, instruction: Instruction, operands: Vec<Operand>);
    fn emit_instruction_with_size(&mut self, instruction: Instruction, size: Size, operands: Vec<Operand>);
}

pub trait CodeEmitterWithComment: Emitter {
    fn emit_instruction_with_comment(&mut self, instruction: Instruction, operands: Vec<Operand>, comment: Option<&str>);
    fn emit_instruction_with_size_and_comment(&mut self, instruction: Instruction, size: Size, operands: Vec<Operand>, comment: Option<&str>);
}

impl<T: Emitter> CodeEmitter for T {
    fn emit_instruction(&mut self, instruction: Instruction, operands: Vec<Operand>) {
        let instr_str = instruction.to_string();
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
        let size_str = match size {
            Size::Byte => "byte",
            Size::Word => "word", 
            Size::Dword => "dword",
            Size::Qword => "qword",
        };
        let instr_str = instruction.to_string();
        let operands_str = operands.iter()
            .enumerate()
            .map(|(i, op)| {
                if i == 0 && matches!(op, Operand::Memory { .. }) {
                    format!("{} {}", size_str, op.to_string())
                } else {
                    op.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        self.emit_line(&format!("    {:8} {}", instr_str, operands_str));
    }
}

impl<T: Emitter> CodeEmitterWithComment for T {
    fn emit_instruction_with_comment(&mut self, instruction: Instruction, operands: Vec<Operand>, comment: Option<&str>) {
        let instr_str = instruction.to_string();
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
        let size_str = match size {
            Size::Byte => "byte",
            Size::Word => "word", 
            Size::Dword => "dword",
            Size::Qword => "qword",
        };
        let instr_str = instruction.to_string();
        let operands_str = operands.iter()
            .enumerate()
            .map(|(i, op)| {
                if i == 0 && matches!(op, Operand::Memory { .. }) {
                    format!("{} {}", size_str, op.to_string())
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