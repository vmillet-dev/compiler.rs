use crate::ir::{IrValue, IrType, IrBinaryOp, IrUnaryOp};
use crate::codegen::instruction::{Instruction, Operand, Register};
use crate::codegen::emitter::{Emitter, CodeEmitter, CodeEmitterWithComment};
use super::IrCodegen;

impl IrCodegen {
    /// Generate binary operation
    pub fn generate_binary_op(&mut self, dest: &IrValue, op: &IrBinaryOp, left: &IrValue, right: &IrValue, var_type: &IrType) {
        let dest_operand = self.ir_value_to_operand(dest);
        
        match var_type {
            IrType::Float => {
                // Floating point operations - handle float constants specially
                match left {
                    IrValue::FloatConstant(f) => {
                        let float_bits = f.to_bits() as i64;
                        self.emit_instruction_with_comment(Instruction::Mov, vec![
                            Operand::Register(Register::Rax),
                            Operand::Immediate(float_bits)
                        ], Some("load float bits"));
                        self.emit_instruction_with_comment(Instruction::Mov, vec![
                            Operand::Memory { base: Register::Rsp, offset: -8 },
                            Operand::Register(Register::Rax)
                        ], Some("store float to temp memory"));
                        self.emit_instruction_with_comment(Instruction::Movsd, vec![
                            Operand::Register(Register::Xmm0),
                            Operand::Memory { base: Register::Rsp, offset: -8 }
                        ], Some("load left operand"));
                    }
                    _ => {
                        let left_operand = self.ir_value_to_operand(left);
                        self.emit_instruction_with_comment(Instruction::Movsd, vec![
                            Operand::Register(Register::Xmm0),
                            left_operand
                        ], Some("load left operand"));
                    }
                }
                
                let asm_op = match op {
                    IrBinaryOp::Add => Instruction::Addsd,
                    IrBinaryOp::Sub => Instruction::Subsd,
                    IrBinaryOp::Mul => Instruction::Mulsd,
                    IrBinaryOp::Div => Instruction::Divsd,
                    _ => {
                        self.emit_comment(&format!("Unsupported float operation: {}", op));
                        return;
                    }
                };
                
                match right {
                    IrValue::FloatConstant(f) => {
                        let float_bits = f.to_bits() as i64;
                        self.emit_instruction_with_comment(Instruction::Mov, vec![
                            Operand::Register(Register::Rax),
                            Operand::Immediate(float_bits)
                        ], Some("load float bits"));
                        self.emit_instruction_with_comment(Instruction::Mov, vec![
                            Operand::Memory { base: Register::Rsp, offset: -16 },
                            Operand::Register(Register::Rax)
                        ], Some("store float to temp memory"));
                        self.emit_instruction_with_comment(asm_op, vec![
                            Operand::Register(Register::Xmm0),
                            Operand::Memory { base: Register::Rsp, offset: -16 }
                        ], Some(&format!("{} operation", op)));
                    }
                    _ => {
                        let right_operand = self.ir_value_to_operand(right);
                        self.emit_instruction_with_comment(asm_op, vec![
                            Operand::Register(Register::Xmm0),
                            right_operand
                        ], Some(&format!("{} operation", op)));
                    }
                }
                
                self.emit_instruction_with_comment(Instruction::Movsd, vec![
                    dest_operand,
                    Operand::Register(Register::Xmm0)
                ], Some("store result"));
            }
            _ => {
                // Integer operations
                let left_operand = self.ir_value_to_operand(left);
                self.emit_instruction_with_comment(Instruction::Mov, vec![
                    Operand::Register(Register::Eax),
                    left_operand
                ], Some("load left operand"));
                
                let asm_op = match op {
                    IrBinaryOp::Add => Instruction::Add,
                    IrBinaryOp::Sub => Instruction::Sub,
                    IrBinaryOp::Mul => Instruction::Imul,
                    IrBinaryOp::Div => {
                        // Division requires special handling
                        let right_operand = self.ir_value_to_operand(right);
                        self.emit_instruction(Instruction::Cdq, vec![]);
                        self.emit_instruction(Instruction::Idiv, vec![right_operand]);
                        self.emit_instruction(Instruction::Mov, vec![dest_operand, Operand::Register(Register::Eax)]);
                        return;
                    }
                    IrBinaryOp::Eq | IrBinaryOp::Ne | IrBinaryOp::Lt | 
                    IrBinaryOp::Le | IrBinaryOp::Gt | IrBinaryOp::Ge => {
                        // Comparison operations - handle float constants specially
                        match right {
                            IrValue::FloatConstant(f) => {
                                let float_bits = f.to_bits() as i64;
                                self.emit_instruction_with_comment(Instruction::Mov, vec![
                                    Operand::Register(Register::Edx),
                                    Operand::Immediate(float_bits as i32 as i64) // Truncate to 32-bit to avoid overflow
                                ], Some("load float bits for comparison"));
                                self.emit_instruction(Instruction::Cmp, vec![
                                    Operand::Register(Register::Eax),
                                    Operand::Register(Register::Edx)
                                ]);
                            }
                            _ => {
                                let right_operand = self.ir_value_to_operand(right);
                                self.emit_instruction(Instruction::Cmp, vec![
                                    Operand::Register(Register::Eax),
                                    right_operand
                                ]);
                            }
                        }
                        
                        let set_op = match op {
                            IrBinaryOp::Eq => Instruction::Sete,
                            IrBinaryOp::Ne => Instruction::Setne,
                            IrBinaryOp::Lt => Instruction::Setl,
                            IrBinaryOp::Le => Instruction::Setle,
                            IrBinaryOp::Gt => Instruction::Setg,
                            IrBinaryOp::Ge => Instruction::Setge,
                            _ => unreachable!(),
                        };
                        
                        self.emit_instruction(set_op, vec![Operand::Register(Register::Al)]);
                        self.emit_instruction(Instruction::Movzx, vec![
                            Operand::Register(Register::Eax),
                            Operand::Register(Register::Al)
                        ]);
                        self.emit_instruction(Instruction::Mov, vec![dest_operand, Operand::Register(Register::Eax)]);
                        return;
                    }
                    _ => {
                        self.emit_comment(&format!("Unsupported operation: {}", op));
                        return;
                    }
                };
                
                let right_operand = self.ir_value_to_operand(right);
                self.emit_instruction_with_comment(asm_op, vec![
                    Operand::Register(Register::Eax),
                    right_operand
                ], Some(&format!("{} operation", op)));
                
                self.emit_instruction_with_comment(Instruction::Mov, vec![
                    dest_operand,
                    Operand::Register(Register::Eax)
                ], Some("store result"));
            }
        }
    }

    /// Generate unary operation
    pub fn generate_unary_op(&mut self, dest: &IrValue, op: &IrUnaryOp, operand: &IrValue, _var_type: &IrType) {
        let operand_op = self.ir_value_to_operand(operand);
        let dest_operand = self.ir_value_to_operand(dest);
        
        match op {
            IrUnaryOp::Neg => {
                self.emit_instruction_with_comment(Instruction::Mov, vec![
                    Operand::Register(Register::Eax),
                    operand_op
                ], Some("load operand"));
                
                self.emit_instruction_with_comment(Instruction::Neg, vec![
                    Operand::Register(Register::Eax)
                ], Some("negate"));
                
                self.emit_instruction_with_comment(Instruction::Mov, vec![
                    dest_operand,
                    Operand::Register(Register::Eax)
                ], Some("store result"));
            }
            IrUnaryOp::Not => {
                self.emit_instruction_with_comment(Instruction::Mov, vec![
                    Operand::Register(Register::Eax),
                    operand_op
                ], Some("load operand"));
                
                self.emit_instruction_with_comment(Instruction::Cmp, vec![
                    Operand::Register(Register::Eax),
                    Operand::Immediate(0)
                ], Some("test for zero"));
                
                self.emit_instruction(Instruction::Sete, vec![Operand::Register(Register::Al)]);
                self.emit_instruction(Instruction::Movzx, vec![
                    Operand::Register(Register::Eax),
                    Operand::Register(Register::Al)
                ]);
                
                self.emit_instruction_with_comment(Instruction::Mov, vec![
                    dest_operand,
                    Operand::Register(Register::Eax)
                ], Some("store result"));
            }
        }
    }
}