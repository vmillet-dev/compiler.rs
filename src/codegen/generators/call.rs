//! Function call generation utilities
//! 
//! This module provides utilities for generating function calls
//! with proper argument passing and calling conventions.

use crate::ir::IrValue;
use crate::codegen::core::{Emitter, CodeEmitterWithComment, Target, Instruction, Register, Operand, Size};
use crate::codegen::utils::StackManager;

/// Function call generation utilities
pub struct CallGenerator;

impl CallGenerator {
    /// Generate function call with arguments
    pub fn generate_call<T: Emitter + CodeEmitterWithComment>(
        emitter: &mut T,
        stack_manager: &StackManager,
        target: &dyn Target,
        dest: &Option<IrValue>,
        function_name: &str,
        args: &[IrValue],
    ) {
        emitter.emit_comment(&format!("Call function: {}", function_name));
        
        // Prepare arguments according to calling convention
        Self::prepare_arguments(emitter, stack_manager, target, args);
        
        // Generate the actual call
        let call_instructions = target.format_function_call(function_name);
        for instr in call_instructions {
            emitter.emit_line_with_comment(
                &format!("    {}", instr),
                Some(&format!("call {}", function_name))
            );
        }
        
        // Handle return value if needed
        if let Some(dest) = dest {
            Self::handle_return_value(emitter, stack_manager, target, dest);
        }
        
        // Clean up stack if needed (for cdecl calling convention)
        if !args.is_empty() {
            Self::cleanup_stack_after_call(emitter, target, args.len());
        }
    }
    
    /// Prepare function arguments according to calling convention
    fn prepare_arguments<T: Emitter + CodeEmitterWithComment>(
        emitter: &mut T,
        stack_manager: &StackManager,
        target: &dyn Target,
        args: &[IrValue],
    ) {
        let param_registers = target.parameter_registers();
        
        emitter.emit_comment("Prepare function arguments");
        
        for (i, arg) in args.iter().enumerate() {
            if i < param_registers.len() {
                // Pass argument in register
                let reg = param_registers[i];
                Self::load_argument_to_register(emitter, stack_manager, arg, reg, i);
            } else {
                // Pass argument on stack
                Self::push_argument_to_stack(emitter, stack_manager, arg, i);
            }
        }
    }
    
    /// Load an argument into a register
    fn load_argument_to_register<T: Emitter + CodeEmitterWithComment>(
        emitter: &mut T,
        stack_manager: &StackManager,
        arg: &IrValue,
        register: Register,
        arg_index: usize,
    ) {
        match arg {
            IrValue::IntConstant(val) => {
                emitter.emit_instruction_with_size_and_comment(
                    Instruction::Mov,
                    Size::Qword,
                    vec![Operand::Register(register), Operand::Immediate(*val)],
                    Some(&format!("arg {} = {}", arg_index, val))
                );
            }
            IrValue::Local(var_name) => {
                if let Some(offset) = stack_manager.get_local_offset(var_name) {
                    emitter.emit_instruction_with_size_and_comment(
                        Instruction::Mov,
                        Size::Dword,
                        vec![
                            Operand::Register(register),
                            Operand::Memory { base: Register::Rbp, offset: offset }
                        ],
                        Some(&format!("arg {} = {}", arg_index, var_name))
                    );
                }
            }
            IrValue::StringConstant(content) => {
                emitter.emit_instruction_with_size_and_comment(
                    Instruction::Lea,
                    Size::Qword,
                    vec![Operand::Register(register), Operand::String(content.clone())],
                    Some(&format!("arg {} = string \"{}\"", arg_index, content))
                );
            }
            _ => {
                emitter.emit_comment(&format!("TODO: load arg {} {:?} to register", arg_index, arg));
            }
        }
    }
    
    /// Push an argument onto the stack
    fn push_argument_to_stack<T: Emitter + CodeEmitterWithComment>(
        emitter: &mut T,
        stack_manager: &StackManager,
        arg: &IrValue,
        arg_index: usize,
    ) {
        match arg {
            IrValue::IntConstant(val) => {
                emitter.emit_instruction_with_comment(
                    Instruction::Push,
                    vec![Operand::Immediate(*val)],
                    Some(&format!("push arg {} = {}", arg_index, val))
                );
            }
            IrValue::Local(var_name) => {
                if let Some(offset) = stack_manager.get_local_offset(var_name) {
                    // Load to temporary register first, then push
                    emitter.emit_instruction_with_size_and_comment(
                        Instruction::Mov,
                        Size::Dword,
                        vec![
                            Operand::Register(Register::Eax),
                            Operand::Memory { base: Register::Rbp, offset: offset }
                        ],
                        Some(&format!("load arg {} = {}", arg_index, var_name))
                    );
                    emitter.emit_instruction_with_comment(
                        Instruction::Push,
                        vec![Operand::Register(Register::Rax)],
                        Some(&format!("push arg {}", arg_index))
                    );
                }
            }
            _ => {
                emitter.emit_comment(&format!("TODO: push arg {} {:?} to stack", arg_index, arg));
            }
        }
    }
    
    /// Handle return value from function call
    fn handle_return_value<T: Emitter + CodeEmitterWithComment>(
        emitter: &mut T,
        stack_manager: &StackManager,
        target: &dyn Target,
        dest: &IrValue,
    ) {
        let return_reg = target.return_register();
        
        if let IrValue::Local(dest_name) = dest {
            if let Some(dest_offset) = stack_manager.get_local_offset(dest_name) {
                emitter.emit_instruction_with_size_and_comment(
                    Instruction::Mov,
                    Size::Dword,
                    vec![
                        Operand::Memory { base: Register::Rbp, offset: dest_offset },
                        Operand::Register(return_reg)
                    ],
                    Some(&format!("store return value to {}", dest_name))
                );
            }
        }
    }
    
    /// Clean up stack after function call (for cdecl calling convention)
    fn cleanup_stack_after_call<T: Emitter + CodeEmitterWithComment>(
        emitter: &mut T,
        target: &dyn Target,
        arg_count: usize,
    ) {
        let param_registers = target.parameter_registers();
        
        // Only need to clean up stack arguments (those beyond register parameters)
        if arg_count > param_registers.len() {
            let stack_args = arg_count - param_registers.len();
            let cleanup_bytes = stack_args * 8; // 8 bytes per argument on x64
            
            if cleanup_bytes > 0 {
                emitter.emit_instruction_with_size_and_comment(
                    Instruction::Add,
                    Size::Qword,
                    vec![Operand::Register(Register::Rsp), Operand::Immediate(cleanup_bytes as i64)],
                    Some(&format!("clean up {} bytes of stack arguments", cleanup_bytes))
                );
            }
        }
    }
    
    /// Generate built-in function call (like println)
    pub fn generate_builtin_call<T: Emitter + CodeEmitterWithComment>(
        emitter: &mut T,
        stack_manager: &StackManager,
        target: &dyn Target,
        function_name: &str,
        args: &[IrValue],
    ) {
        match function_name {
            "println" => {
                Self::generate_println_call(emitter, stack_manager, target, args);
            }
            _ => {
                emitter.emit_comment(&format!("TODO: builtin function {}", function_name));
            }
        }
    }
    
    /// Generate println function call
    fn generate_println_call<T: Emitter + CodeEmitterWithComment>(
        emitter: &mut T,
        stack_manager: &StackManager,
        target: &dyn Target,
        args: &[IrValue],
    ) {
        emitter.emit_comment("Built-in println function call");
        
        if !args.is_empty() {
            // For now, assume first argument is a format string
            Self::prepare_arguments(emitter, stack_manager, target, args);
            
            // Call printf
            let call_instructions = target.format_function_call("printf");
            for instr in call_instructions {
                emitter.emit_line_with_comment(
                    &format!("    {}", instr),
                    Some("call printf")
                );
            }
        }
    }
}