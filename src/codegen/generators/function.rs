//! Function code generation utilities
//! 
//! This module provides utilities for generating function-specific
//! assembly code including prologues, epilogues, and function calls.

use crate::ir::IrFunction;
use crate::codegen::core::{Emitter, CodeEmitterWithComment, Target};
use crate::codegen::utils::StackManager;

/// Function code generation utilities
pub struct FunctionGenerator;

impl FunctionGenerator {
    /// Calculate the stack space needed for a function
    pub fn calculate_stack_space(function: &IrFunction, _stack_manager: &StackManager) -> usize {
        // Calculate space for all local variables and temporaries
        let mut total_space = 0;
        
        // Add space for each instruction that allocates variables
        for instruction in &function.instructions {
            if let crate::ir::IrInstruction::Alloca { var_type, .. } = instruction {
                let size = match var_type {
                    crate::ir::IrType::Int => 4,
                    crate::ir::IrType::Float => 8,
                    crate::ir::IrType::Char => 1,
                    _ => 8,
                };
                total_space += size;
            }
        }
        
        // Align to 16-byte boundary
        (total_space + 15) & !15
    }
    
    /// Generate function prologue with stack allocation
    pub fn generate_prologue_with_stack<T: Emitter + CodeEmitterWithComment>(
        emitter: &mut T,
        target: &dyn Target,
        stack_space: usize,
    ) {
        emitter.emit_subsection_header("Function Prologue");
        
        let prologue_instructions = target.function_prologue();
        for (i, instr) in prologue_instructions.iter().enumerate() {
            let comment = match i {
                0 => Some("save caller's frame pointer"),
                1 => Some("set up new frame pointer"),
                _ => None,
            };
            emitter.emit_line_with_comment(&format!("    {}", instr), comment);
        }
        
        // Allocate stack space if needed
        if stack_space > 0 {
            emitter.emit_line_with_comment(
                &format!("    sub rsp, {}", stack_space),
                Some(&format!("allocate {} bytes for locals", stack_space))
            );
        }
        
        emitter.emit_line("");
    }
    
    /// Generate function epilogue with stack deallocation
    pub fn generate_epilogue_with_stack<T: Emitter + CodeEmitterWithComment>(
        emitter: &mut T,
        target: &dyn Target,
        stack_space: usize,
    ) {
        emitter.emit_subsection_header("Function Epilogue");
        
        // Deallocate stack space if needed
        if stack_space > 0 {
            emitter.emit_line_with_comment(
                &format!("    add rsp, {}", stack_space),
                Some("deallocate local variables")
            );
        }
        
        let epilogue_instructions = target.function_epilogue();
        for (i, instr) in epilogue_instructions.iter().enumerate() {
            let comment = match i {
                0 => Some("restore stack pointer"),
                1 => Some("restore caller's frame pointer"),
                2 => Some("return to caller"),
                _ => None,
            };
            emitter.emit_line_with_comment(&format!("    {}", instr), comment);
        }
    }
    
    /// Generate function label and setup
    pub fn generate_function_header<T: Emitter>(
        emitter: &mut T,
        function_name: &str,
    ) {
        emitter.emit_subsection_header(&format!("FUNCTION: {}", function_name));
        emitter.emit_line(&format!("{}:", function_name));
    }
}