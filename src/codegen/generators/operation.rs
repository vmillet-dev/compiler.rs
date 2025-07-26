//! Operation generation utilities
//! 
//! This module provides utilities for generating arithmetic and
//! logical operations from IR instructions.

use crate::ir::IrValue;
use crate::codegen::core::{Emitter, CodeEmitterWithComment, Instruction, Register, Operand, Size};
use crate::codegen::utils::StackManager;

/// Operation generation utilities
pub struct OperationGenerator;

impl OperationGenerator {
    /// Generate binary arithmetic operation
    pub fn generate_binary_op<T: Emitter + CodeEmitterWithComment>(
        emitter: &mut T,
        stack_manager: &StackManager,
        dest: &IrValue,
        left: &IrValue,
        right: &IrValue,
        op: Instruction,
        op_name: &str,
    ) {
        emitter.emit_comment(&format!("Binary operation: {}", op_name));
        
        // Load left operand into EAX
        Self::load_value_to_register(emitter, stack_manager, left, Register::Eax);
        
        // Perform operation with right operand
        match right {
            IrValue::IntConstant(val) => {
                emitter.emit_instruction_with_size_and_comment(
                    op,
                    Size::Dword,
                    vec![Operand::Register(Register::Eax), Operand::Immediate(*val)],
                    Some(&format!("{} with immediate {}", op_name, val))
                );
            }
            IrValue::Local(var_name) => {
                if let Some(offset) = stack_manager.get_local_offset(var_name) {
                    emitter.emit_instruction_with_size_and_comment(
                        op,
                        Size::Dword,
                        vec![
                            Operand::Register(Register::Eax),
                            Operand::Memory { base: Register::Rbp, offset: offset }
                        ],
                        Some(&format!("{} with {}", op_name, var_name))
                    );
                }
            }
            _ => {
                emitter.emit_comment(&format!("TODO: {} with {:?}", op_name, right));
            }
        }
        
        // Store result to destination
        Self::store_register_to_value(emitter, stack_manager, Register::Eax, dest);
    }
    
    /// Generate comparison operation
    pub fn generate_comparison<T: Emitter + CodeEmitterWithComment>(
        emitter: &mut T,
        stack_manager: &StackManager,
        dest: &IrValue,
        left: &IrValue,
        right: &IrValue,
        set_instruction: Instruction,
        op_name: &str,
    ) {
        emitter.emit_comment(&format!("Comparison: {}", op_name));
        
        // Load left operand into EAX
        Self::load_value_to_register(emitter, stack_manager, left, Register::Eax);
        
        // Compare with right operand
        match right {
            IrValue::IntConstant(val) => {
                emitter.emit_instruction_with_size_and_comment(
                    Instruction::Cmp,
                    Size::Dword,
                    vec![Operand::Register(Register::Eax), Operand::Immediate(*val)],
                    Some(&format!("compare with {}", val))
                );
            }
            IrValue::Local(var_name) => {
                if let Some(offset) = stack_manager.get_local_offset(var_name) {
                    emitter.emit_instruction_with_size_and_comment(
                        Instruction::Cmp,
                        Size::Dword,
                        vec![
                            Operand::Register(Register::Eax),
                            Operand::Memory { base: Register::Rbp, offset: offset }
                        ],
                        Some(&format!("compare with {}", var_name))
                    );
                }
            }
            _ => {
                emitter.emit_comment(&format!("TODO: compare with {:?}", right));
            }
        }
        
        // Set result based on comparison
        emitter.emit_instruction_with_comment(
            set_instruction,
            vec![Operand::Register(Register::Al)],
            Some(&format!("set result for {}", op_name))
        );
        
        // Zero-extend AL to EAX
        emitter.emit_instruction_with_size_and_comment(
            Instruction::Movzx,
            Size::Dword,
            vec![Operand::Register(Register::Eax), Operand::Register(Register::Al)],
            Some("zero-extend result to 32-bit")
        );
        
        // Store result to destination
        Self::store_register_to_value(emitter, stack_manager, Register::Eax, dest);
    }
    
    /// Generate unary operation
    pub fn generate_unary_op<T: Emitter + CodeEmitterWithComment>(
        emitter: &mut T,
        stack_manager: &StackManager,
        dest: &IrValue,
        operand: &IrValue,
        op: Instruction,
        op_name: &str,
    ) {
        emitter.emit_comment(&format!("Unary operation: {}", op_name));
        
        // Load operand into EAX
        Self::load_value_to_register(emitter, stack_manager, operand, Register::Eax);
        
        // Perform unary operation
        emitter.emit_instruction_with_size_and_comment(
            op,
            Size::Dword,
            vec![Operand::Register(Register::Eax)],
            Some(&format!("{} operation", op_name))
        );
        
        // Store result to destination
        Self::store_register_to_value(emitter, stack_manager, Register::Eax, dest);
    }
    
    /// Load a value into a register
    fn load_value_to_register<T: Emitter + CodeEmitterWithComment>(
        emitter: &mut T,
        stack_manager: &StackManager,
        value: &IrValue,
        register: Register,
    ) {
        match value {
            IrValue::IntConstant(val) => {
                emitter.emit_instruction_with_size_and_comment(
                    Instruction::Mov,
                    Size::Dword,
                    vec![Operand::Register(register), Operand::Immediate(*val)],
                    Some(&format!("load immediate {}", val))
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
                        Some(&format!("load {}", var_name))
                    );
                }
            }
            _ => {
                emitter.emit_comment(&format!("TODO: load {:?} to register", value));
            }
        }
    }
    
    /// Store a register value to a destination
    fn store_register_to_value<T: Emitter + CodeEmitterWithComment>(
        emitter: &mut T,
        stack_manager: &StackManager,
        register: Register,
        dest: &IrValue,
    ) {
        if let IrValue::Local(dest_name) = dest {
            if let Some(dest_offset) = stack_manager.get_local_offset(dest_name) {
                emitter.emit_instruction_with_size_and_comment(
                    Instruction::Mov,
                    Size::Dword,
                    vec![
                        Operand::Memory { base: Register::Rbp, offset: dest_offset },
                        Operand::Register(register)
                    ],
                    Some(&format!("store result to {}", dest_name))
                );
            }
        }
    }
}