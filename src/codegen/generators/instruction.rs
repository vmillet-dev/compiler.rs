//! Instruction generation utilities
//! 
//! This module provides utilities for generating specific types of
//! assembly instructions from IR instructions.

use crate::ir::{IrValue, IrType};
use crate::codegen::core::{Emitter, CodeEmitterWithComment, Instruction, Register, Operand, Size};
use crate::codegen::utils::StackManager;
use crate::lexer::TokenType;

/// Instruction generation utilities
pub struct InstructionGenerator;

impl InstructionGenerator {
    /// Generate alloca instruction
    pub fn generate_alloca<T: Emitter>(
        emitter: &mut T,
        stack_manager: &mut StackManager,
        name: &str,
        var_type: &IrType,
    ) {
        let token_type = Self::ir_type_to_token_type(var_type);
        let offset = stack_manager.allocate_local(name.to_string(), token_type);
        let size = Self::type_size(var_type);
        emitter.emit_comment(&format!("alloca {} ({} bytes) at offset {}", name, size, offset));
    }
    
    /// Generate store instruction
    pub fn generate_store<T: Emitter + CodeEmitterWithComment>(
        emitter: &mut T,
        stack_manager: &StackManager,
        value: &IrValue,
        dest: &IrValue,
    ) {
        if let IrValue::Local(dest_name) = dest {
            if let Some(dest_offset) = stack_manager.get_local_offset(dest_name) {
                match value {
                    IrValue::IntConstant(val) => {
                        emitter.emit_instruction_with_size_and_comment(
                            Instruction::Mov,
                            Size::Dword,
                            vec![
                                Operand::Memory { base: Register::Rbp, offset: dest_offset },
                                Operand::Immediate(*val)
                            ],
                            Some(&format!("store {} -> {}", val, dest_name))
                        );
                    }
                    IrValue::FloatConstant(val) => {
                        // For floating point, we need to use SSE instructions
                        emitter.emit_comment(&format!("TODO: store float {} -> {}", val, dest_name));
                    }
                    IrValue::Local(src_name) => {
                        if let Some(src_offset) = stack_manager.get_local_offset(src_name) {
                            // Load from source to register, then store to destination
                            emitter.emit_instruction_with_size_and_comment(
                                Instruction::Mov,
                                Size::Dword,
                                vec![
                                    Operand::Register(Register::Eax),
                                    Operand::Memory { base: Register::Rbp, offset: src_offset }
                                ],
                                Some(&format!("load {}", src_name))
                            );
                            emitter.emit_instruction_with_size_and_comment(
                                Instruction::Mov,
                                Size::Dword,
                                vec![
                                    Operand::Memory { base: Register::Rbp, offset: dest_offset },
                                    Operand::Register(Register::Eax)
                                ],
                                Some(&format!("store to {}", dest_name))
                            );
                        }
                    }
                    _ => {
                        emitter.emit_comment(&format!("TODO: store {:?} -> {:?}", value, dest));
                    }
                }
            }
        }
    }
    
    /// Generate load instruction
    pub fn generate_load<T: Emitter + CodeEmitterWithComment>(
        emitter: &mut T,
        stack_manager: &StackManager,
        dest: &IrValue,
        src: &IrValue,
    ) {
        if let (IrValue::Local(dest_name), IrValue::Local(src_name)) = (dest, src) {
            if let Some(src_offset) = stack_manager.get_local_offset(src_name) {
                emitter.emit_instruction_with_size_and_comment(
                    Instruction::Mov,
                    Size::Dword,
                    vec![
                        Operand::Register(Register::Eax),
                        Operand::Memory { base: Register::Rbp, offset: src_offset }
                    ],
                    Some(&format!("load {} from {}", dest_name, src_name))
                );
            }
        }
    }
    
    /// Generate return instruction
    pub fn generate_return<T: Emitter + CodeEmitterWithComment>(
        emitter: &mut T,
        stack_manager: &StackManager,
        value: &Option<IrValue>,
    ) {
        if let Some(value) = value {
            match value {
                IrValue::IntConstant(val) => {
                    emitter.emit_instruction_with_size_and_comment(
                        Instruction::Mov,
                        Size::Dword,
                        vec![Operand::Register(Register::Eax), Operand::Immediate(*val)],
                        Some(&format!("return {}", val))
                    );
                }
                IrValue::FloatConstant(val) => {
                    emitter.emit_comment(&format!("TODO: return float {}", val));
                }
                IrValue::Local(var_name) => {
                    if let Some(offset) = stack_manager.get_local_offset(var_name) {
                        emitter.emit_instruction_with_size_and_comment(
                            Instruction::Mov,
                            Size::Dword,
                            vec![
                                Operand::Register(Register::Eax),
                                Operand::Memory { base: Register::Rbp, offset: offset }
                            ],
                            Some(&format!("return {}", var_name))
                        );
                    }
                }
                _ => {
                    emitter.emit_comment(&format!("TODO: return {:?}", value));
                }
            }
        }
    }
    
    /// Convert IR type to token type
    fn ir_type_to_token_type(ir_type: &IrType) -> TokenType {
        match ir_type {
            IrType::Int => TokenType::Int,
            IrType::Float => TokenType::FloatType,
            IrType::Char => TokenType::CharType,
            IrType::Void => TokenType::Void,
            _ => TokenType::Int, // Default fallback
        }
    }
    
    /// Get the size of a type in bytes
    fn type_size(ir_type: &IrType) -> usize {
        match ir_type {
            IrType::Int => 4,
            IrType::Float => 8,
            IrType::Char => 1,
            IrType::Void => 0,
            _ => 8, // Default to pointer size
        }
    }
}