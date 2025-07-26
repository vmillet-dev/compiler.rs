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
        self.emit_instruction_with_comment(Instruction::Push, vec![
            Operand::Register(Register::Rbp)
        ], Some("save caller's frame"));
        self.emit_instruction_with_comment(Instruction::Mov, vec![
            Operand::Register(Register::Rbp), 
            Operand::Register(Register::Rsp)
        ], Some("set up frame"));

        // Calculate stack space needed
        let stack_space = self.calculate_stack_space(function);
        if stack_space > 0 {
            self.emit_instruction_with_comment(Instruction::Sub, vec![
                Operand::Register(Register::Rsp), 
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
                Operand::Register(Register::Rsp), 
                Operand::Immediate(stack_space as i64)
            ], Some("deallocate stack space"));
        }
        
        self.emit_instruction_with_comment(Instruction::Pop, vec![
            Operand::Register(Register::Rbp)
        ], Some("restore frame"));
        self.emit_instruction_with_comment(Instruction::Ret, vec![], Some("return"));
        
        self.emit_line(""); // Add spacing after function
    }
}