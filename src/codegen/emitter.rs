use super::instruction::{Instruction, Operand, Size};

pub trait Emitter {
    fn emit_line(&mut self, line: &str);
    fn emit_comment(&mut self, comment: &str);
}

pub trait CodeEmitter: Emitter {
    fn emit_instruction(&mut self, instruction: Instruction, operands: Vec<Operand>);
    fn emit_instruction_with_size(&mut self, instruction: Instruction, size: Size, operands: Vec<Operand>);
}

impl<T: Emitter> CodeEmitter for T {
    fn emit_instruction(&mut self, instruction: Instruction, operands: Vec<Operand>) {
        let instr_str = instruction.to_string();
        if operands.is_empty() {
            self.emit_line(&format!("    {}", instr_str));
        } else {
            let operands_str = operands.iter()
                .map(|op| op.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            self.emit_line(&format!("    {} {}", instr_str, operands_str));
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
        self.emit_line(&format!("    {} {}", instr_str, operands_str));
    }
}