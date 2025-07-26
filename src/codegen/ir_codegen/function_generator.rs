use crate::ir::{IrFunction};
use crate::codegen::instruction::{Instruction, Operand, Register};
use crate::codegen::emitter::{Emitter, CodeEmitterWithComment};
use super::IrCodegen;

impl IrCodegen {
    /// Generate assembly for a single function
    pub fn generate_function(&mut self, function: &IrFunction) {
        self.emit_subsection_header(&format!("FUNCTION: {}", function.name));
        self.emit_line(&format!("{}:", function.name));
        
        // Reset state for new function
        self.stack_offset = 0;
        self.locals.clear();
        self.temp_locations.clear();

        // Function prologue
        self.emit_subsection_header("Function Prologue");
        let prologue_instructions = self.target.function_prologue();
        for (i, instr) in prologue_instructions.iter().enumerate() {
            let comment = match i {
                0 => Some("save caller's frame"),
                1 => Some("set up frame"),
                _ => None,
            };
            self.emit_line_with_comment(&format!("    {}", instr), comment);
        }

        // Calculate stack space needed
        let stack_space = self.calculate_stack_space(function);
        if stack_space > 0 {
            self.emit_instruction_with_comment(Instruction::Sub, vec![
                Operand::Register(self.target.stack_pointer()), 
                Operand::Immediate(stack_space as i64)
            ], Some(&format!("allocate {} bytes for locals and temps", stack_space)));
        }

        // Generate function body
        self.emit_subsection_header("Function Body");
        for instruction in &function.instructions {
            self.generate_instruction(instruction);
        }

        // Function epilogue
        self.emit_subsection_header("Function Epilogue");
        self.emit_stack_layout_summary();
        
        if stack_space > 0 {
            self.emit_instruction_with_comment(Instruction::Add, vec![
                Operand::Register(self.target.stack_pointer()), 
                Operand::Immediate(stack_space as i64)
            ], Some("deallocate stack space"));
        }
        
        let epilogue_instructions = self.target.function_epilogue();
        for (i, instr) in epilogue_instructions.iter().enumerate() {
            let comment = match i {
                0 => Some("restore stack pointer"),
                1 => Some("restore frame"),
                2 => Some("return"),
                _ => None,
            };
            self.emit_line_with_comment(&format!("    {}", instr), comment);
        }
        
        self.emit_line(""); // Add spacing after function
    }
}