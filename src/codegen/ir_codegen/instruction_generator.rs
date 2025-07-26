use crate::ir::{IrInstruction, IrValue, IrType};
use crate::codegen::instruction::{Instruction, Operand, Register, Size};
use crate::codegen::emitter::{Emitter, CodeEmitter, CodeEmitterWithComment};
use super::IrCodegen;

impl IrCodegen {
    /// Generate assembly for a single IR instruction
    pub fn generate_instruction(&mut self, instruction: &IrInstruction) {
        match instruction {
            IrInstruction::Alloca { var_type, name } => {
                // Space already allocated in prologue, just add comment
                let size = self.get_type_size(var_type);
                let offset = self.locals.get(name).copied().unwrap_or(0);
                self.emit_comment(&format!("alloca {} {} at [rbp{}] ({} bytes)", 
                    var_type, name, offset, size));
            }

            IrInstruction::Load { dest, src, var_type } => {
                let src_operand = self.ir_value_to_operand(src);
                let dest_operand = self.ir_value_to_operand(dest);
                let size = self.ir_type_to_size(var_type);
                
                // Use register as intermediate for memory-to-memory moves
                let reg = match size {
                    Size::Byte => Register::Al,
                    Size::Dword => Register::Eax,
                    Size::Qword => Register::Rax,
                    _ => Register::Eax,
                };
                
                self.emit_instruction_with_size_and_comment(Instruction::Mov, size, vec![
                    Operand::Register(reg),
                    src_operand
                ], Some(&format!("load {} {} to register", var_type, self.ir_value_to_string(src))));
                
                self.emit_instruction_with_size_and_comment(Instruction::Mov, size, vec![
                    dest_operand,
                    Operand::Register(reg)
                ], Some("store to destination"));
            }

            IrInstruction::Store { value, dest, var_type } => {
                let dest_operand = self.ir_value_to_operand(dest);
                let size = self.ir_type_to_size(var_type);
                
                // Handle different value types appropriately
                match (value, var_type) {
                    (IrValue::FloatConstant(f), IrType::Float) => {
                        // For float constants, we need to handle them specially
                        self.emit_comment(&format!("store float constant {} to {}", f, self.ir_value_to_string(dest)));
                        // Move the float bits as integer first, then convert
                        let bits = f.to_bits() as i64;
                        self.emit_instruction_with_comment(Instruction::Mov, vec![
                            Operand::Register(Register::Rax),
                            Operand::Immediate(bits)
                        ], Some("load float bits"));
                        self.emit_instruction_with_size_and_comment(Instruction::Mov, Size::Qword, vec![
                            dest_operand,
                            Operand::Register(Register::Rax)
                        ], Some("store float"));
                    }
                    _ => {
                        // For other types, get the value operand and use register as intermediate if needed
                        let value_operand = self.ir_value_to_operand(value);
                        let reg = match size {
                            Size::Byte => Register::Al,
                            Size::Dword => Register::Eax,
                            Size::Qword => Register::Rax,
                            _ => Register::Eax,
                        };
                        
                        // Check if we need an intermediate register
                        let needs_intermediate = matches!(value_operand, Operand::Memory { .. }) && 
                                               matches!(dest_operand, Operand::Memory { .. });
                        
                        if needs_intermediate {
                            self.emit_instruction_with_size_and_comment(Instruction::Mov, size, vec![
                                Operand::Register(reg),
                                value_operand
                            ], Some(&format!("load {} to register", self.ir_value_to_string(value))));
                            
                            self.emit_instruction_with_size_and_comment(Instruction::Mov, size, vec![
                                dest_operand,
                                Operand::Register(reg)
                            ], Some(&format!("store to {}", self.ir_value_to_string(dest))));
                        } else {
                            self.emit_instruction_with_size_and_comment(Instruction::Mov, size, vec![
                                dest_operand,
                                value_operand
                            ], Some(&format!("store {} to {}", self.ir_value_to_string(value), self.ir_value_to_string(dest))));
                        }
                    }
                }
            }

            IrInstruction::BinaryOp { dest, op, left, right, var_type } => {
                self.generate_binary_op(dest, op, left, right, var_type);
            }

            IrInstruction::UnaryOp { dest, op, operand, var_type } => {
                self.generate_unary_op(dest, op, operand, var_type);
            }

            IrInstruction::Call { dest, func, args, return_type } => {
                self.generate_function_call(dest, func, args, return_type);
            }

            IrInstruction::Branch { condition, true_label, false_label } => {
                let condition_operand = self.ir_value_to_operand(condition);
                
                // Load condition to register first, then compare
                self.emit_instruction_with_comment(Instruction::Mov, vec![
                    Operand::Register(Register::Eax),
                    condition_operand
                ], Some("load condition"));
                
                self.emit_instruction_with_comment(Instruction::Cmp, vec![
                    Operand::Register(Register::Eax),
                    Operand::Immediate(0)
                ], Some("test condition"));
                
                self.emit_instruction(Instruction::Je, vec![
                    Operand::Label(false_label.clone())
                ]);
                self.emit_instruction(Instruction::Jmp, vec![
                    Operand::Label(true_label.clone())
                ]);
            }

            IrInstruction::Jump { label } => {
                self.emit_instruction(Instruction::Jmp, vec![
                    Operand::Label(label.clone())
                ]);
            }

            IrInstruction::Label { name } => {
                self.emit_line(&format!("{}:", name));
            }

            IrInstruction::Return { value, var_type } => {
                if let Some(val) = value {
                    let val_operand = self.ir_value_to_operand(val);
                    let register = match var_type {
                        IrType::Float => Register::Xmm0,
                        _ => Register::Eax,
                    };
                    
                    match var_type {
                        IrType::Float => {
                            self.emit_instruction_with_comment(Instruction::Movsd, vec![
                                Operand::Register(register),
                                val_operand
                            ], Some(&format!("return {}", self.ir_value_to_string(val))));
                        }
                        _ => {
                            self.emit_instruction_with_comment(Instruction::Mov, vec![
                                Operand::Register(register),
                                val_operand
                            ], Some(&format!("return {}", self.ir_value_to_string(val))));
                        }
                    }
                } else {
                    self.emit_instruction_with_comment(Instruction::Xor, vec![
                        Operand::Register(Register::Eax),
                        Operand::Register(Register::Eax)
                    ], Some("return 0"));
                }
            }

            IrInstruction::Print { format_string, args } => {
                self.generate_print_call(format_string, args);
            }

            IrInstruction::Move { dest, src, var_type } => {
                let src_operand = self.ir_value_to_operand(src);
                let dest_operand = self.ir_value_to_operand(dest);
                let size = self.ir_type_to_size(var_type);
                
                // Use register as intermediate for memory-to-memory moves
                let needs_intermediate = matches!(src_operand, Operand::Memory { .. }) && 
                                       matches!(dest_operand, Operand::Memory { .. });
                
                if needs_intermediate {
                    let reg = match size {
                        Size::Byte => Register::Al,
                        Size::Dword => Register::Eax,
                        Size::Qword => Register::Rax,
                        _ => Register::Eax,
                    };
                    
                    self.emit_instruction_with_size_and_comment(Instruction::Mov, size, vec![
                        Operand::Register(reg),
                        src_operand
                    ], Some(&format!("load {} to register", self.ir_value_to_string(src))));
                    
                    self.emit_instruction_with_size_and_comment(Instruction::Mov, size, vec![
                        dest_operand,
                        Operand::Register(reg)
                    ], Some(&format!("move to {}", self.ir_value_to_string(dest))));
                } else {
                    self.emit_instruction_with_size_and_comment(Instruction::Mov, size, vec![
                        dest_operand,
                        src_operand
                    ], Some(&format!("move {} to {}", self.ir_value_to_string(src), self.ir_value_to_string(dest))));
                }
            }

            IrInstruction::Convert { dest, dest_type, src, src_type } => {
                // Type conversion - simplified implementation
                let src_operand = self.ir_value_to_operand(src);
                let dest_operand = self.ir_value_to_operand(dest);
                
                self.emit_comment(&format!("convert {} {} to {} {}", 
                    src_type, self.ir_value_to_string(src), dest_type, self.ir_value_to_string(dest)));
                
                // For now, just move (would need proper conversion logic)
                self.emit_instruction(Instruction::Mov, vec![dest_operand, src_operand]);
            }

            IrInstruction::Cast { dest, src, dest_type, src_type } => {
                self.emit_comment(&format!("Cast {} {} to {}", src_type, self.ir_value_to_string(src), dest_type));
                
                // For now, implement basic casting by moving the value
                match (src_type, dest_type) {
                    (IrType::Int, IrType::Float) => {
                        self.emit_instruction(Instruction::Mov, vec![
                            self.ir_value_to_operand(src),
                            self.ir_value_to_operand(dest),
                        ]);
                    }
                    (IrType::Float, IrType::Int) => {
                        // For float to int conversion, use mov for now
                        self.emit_instruction(Instruction::Mov, vec![
                            self.ir_value_to_operand(src),
                            self.ir_value_to_operand(dest),
                        ]);
                    }
                    _ => {
                        // For other cases, just move the value
                        self.emit_instruction(Instruction::Mov, vec![
                            self.ir_value_to_operand(src),
                            self.ir_value_to_operand(dest),
                        ]);
                    }
                }
            }
            IrInstruction::Comment { text } => {
                self.emit_comment(text);
            }
        }
    }
}