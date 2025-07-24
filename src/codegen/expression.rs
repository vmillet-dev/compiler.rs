use crate::lexer::TokenType;
use crate::parser::ast::Expr;
use super::instruction::{Instruction, Operand, Register};
use super::emitter::{Emitter, CodeEmitter, CodeEmitterWithComment};

pub trait ExpressionGenerator: Emitter + CodeEmitter + CodeEmitterWithComment {
    fn gen_expr(&mut self, expr: &Expr);
    fn get_locals(&self) -> &std::collections::HashMap<String, i32>;
    fn get_data_strings(&self) -> &std::collections::HashMap<String, String>;
}

impl ExpressionGenerator for super::Codegen {
    fn gen_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Integer(i) => {
                self.emit_instruction_with_comment(Instruction::Mov, vec![
                    Operand::Register(Register::Rax), 
                    Operand::Immediate(*i)
                ], Some(&format!("load integer {}", i)));
            }
            Expr::Float(f) => {
                let float_bits = f.to_bits();
                self.emit_instruction_with_comment(Instruction::Mov, vec![
                    Operand::Register(Register::Rax), 
                    Operand::Immediate(float_bits as i64)
                ], Some(&format!("load float {} as bits", f)));
            }
            Expr::Char(c) => {
                self.emit_instruction_with_comment(Instruction::Mov, vec![
                    Operand::Register(Register::Rax), 
                    Operand::Immediate(*c as i64)
                ], Some(&format!("load char '{}'", c)));
            }
            Expr::String(s) => {
                // CORRECTION: Utiliser RIP-relative addressing pour les chaînes
                if let Some(label) = self.data_strings.get(s) {
                    self.emit_instruction(Instruction::Lea, vec![
                        Operand::Register(Register::Rax), 
                        Operand::String(format!("[rel {}]", label))
                    ]);
                } else {
                    // This should not happen if collect_format_strings is called correctly
                    self.emit_line(&format!("    ; String literal '{}' not found in data section", s));
                    self.emit_instruction(Instruction::Mov, vec![
                        Operand::Register(Register::Rax), 
                        Operand::Immediate(0)
                    ]);
                }
            }
            Expr::Identifier(name) => {
                if let Some(&offset) = self.locals.get(name) {
                    self.emit_instruction_with_comment(Instruction::Mov, vec![
                        Operand::Register(Register::Rax), 
                        Operand::Memory { base: Register::Rbp, offset }
                    ], Some(&format!("load {}", name)));
                } else {
                    self.emit_instruction_with_comment(Instruction::Mov, vec![
                        Operand::Register(Register::Rax), 
                        Operand::Immediate(0)
                    ], Some(&format!("unknown var {}", name)));
                }
            }
            Expr::Unary { operator, operand } => {
                match operator {
                    TokenType::LogicalNot => { // Unary '!'
                        self.gen_expr(operand);
                        self.emit_instruction_with_comment(Instruction::Cmp, vec![
                            Operand::Register(Register::Rax), 
                            Operand::Immediate(0)
                        ], Some("test for zero"));
                        self.emit_instruction_with_comment(Instruction::Sete, vec![
                            Operand::Register(Register::Al)
                        ], Some("set if zero"));
                        self.emit_instruction_with_comment(Instruction::Movzx, vec![
                            Operand::Register(Register::Rax), 
                            Operand::Register(Register::Al)
                        ], Some("extend to 64-bit"));
                    }
                    TokenType::Minus => { // Unary '-'
                        self.gen_expr(operand);
                        self.emit_instruction_with_comment(Instruction::Neg, vec![
                            Operand::Register(Register::Rax)
                        ], Some("negate"));
                    }
                    _ => {
                        self.emit_line(&format!("    ; unsupported unary operator: {:?}", operator));
                        self.emit_instruction(Instruction::Mov, vec![
                            Operand::Register(Register::Rax), 
                            Operand::Immediate(0)
                        ]);
                    }
                }
            }

            Expr::Binary { left, operator, right } => {
                self.gen_expr(right);
                self.emit_instruction_with_comment(Instruction::Push, vec![
                    Operand::Register(Register::Rax)
                ], Some("save right"));
                self.gen_expr(left);
                self.emit_instruction_with_comment(Instruction::Pop, vec![
                    Operand::Register(Register::R8)
                ], Some("restore right"));

                match operator {
                    TokenType::Plus => {
                        self.emit_instruction_with_comment(Instruction::Add, vec![
                            Operand::Register(Register::Rax), 
                            Operand::Register(Register::R8)
                        ], Some("add"));
                    },
                    TokenType::Minus => {
                        self.emit_instruction_with_comment(Instruction::Sub, vec![
                            Operand::Register(Register::Rax), 
                            Operand::Register(Register::R8)
                        ], Some("subtract"));
                    },
                    TokenType::Multiply => {
                        self.emit_instruction_with_comment(Instruction::Imul, vec![
                            Operand::Register(Register::Rax), 
                            Operand::Register(Register::R8)
                        ], Some("multiply"));
                    },
                    TokenType::Divide => {
                        self.emit_instruction_with_comment(Instruction::Cqo, vec![], Some("sign extend"));
                        self.emit_instruction_with_comment(Instruction::Idiv, vec![
                            Operand::Register(Register::R8)
                        ], Some("divide"));
                    }
                    TokenType::Equal => {
                        self.emit_instruction_with_comment(Instruction::Cmp, vec![
                            Operand::Register(Register::Rax), 
                            Operand::Register(Register::R8)
                        ], Some("compare"));
                        self.emit_instruction_with_comment(Instruction::Sete, vec![
                            Operand::Register(Register::Al)
                        ], Some("set if equal"));
                        self.emit_instruction_with_comment(Instruction::Movzx, vec![
                            Operand::Register(Register::Rax), 
                            Operand::Register(Register::Al)
                        ], Some("extend to 64-bit"));
                    }
                    TokenType::NotEqual => {
                        self.emit_instruction_with_comment(Instruction::Cmp, vec![
                            Operand::Register(Register::Rax), 
                            Operand::Register(Register::R8)
                        ], Some("compare"));
                        self.emit_instruction_with_comment(Instruction::Setne, vec![
                            Operand::Register(Register::Al)
                        ], Some("set if not equal"));
                        self.emit_instruction_with_comment(Instruction::Movzx, vec![
                            Operand::Register(Register::Rax), 
                            Operand::Register(Register::Al)
                        ], Some("extend to 64-bit"));
                    }
                    TokenType::LessThan => {
                        self.emit_instruction_with_comment(Instruction::Cmp, vec![
                            Operand::Register(Register::Rax), 
                            Operand::Register(Register::R8)
                        ], Some("compare"));
                        self.emit_instruction_with_comment(Instruction::Setl, vec![
                            Operand::Register(Register::Al)
                        ], Some("set if less"));
                        self.emit_instruction_with_comment(Instruction::Movzx, vec![
                            Operand::Register(Register::Rax), 
                            Operand::Register(Register::Al)
                        ], Some("extend to 64-bit"));
                    }
                    TokenType::LessEqual => {
                        self.emit_instruction(Instruction::Cmp, vec![
                            Operand::Register(Register::Rax), 
                            Operand::Register(Register::R8)
                        ]);
                        self.emit_instruction(Instruction::Setle, vec![Operand::Register(Register::Al)]);
                        self.emit_instruction(Instruction::Movzx, vec![
                            Operand::Register(Register::Rax), 
                            Operand::Register(Register::Al)
                        ]);
                    }
                    TokenType::GreaterThan => {
                        self.emit_instruction(Instruction::Cmp, vec![
                            Operand::Register(Register::Rax), 
                            Operand::Register(Register::R8)
                        ]);
                        self.emit_instruction(Instruction::Setg, vec![Operand::Register(Register::Al)]);
                        self.emit_instruction(Instruction::Movzx, vec![
                            Operand::Register(Register::Rax), 
                            Operand::Register(Register::Al)
                        ]);
                    }
                    TokenType::GreaterEqual => {
                        self.emit_instruction(Instruction::Cmp, vec![
                            Operand::Register(Register::Rax), 
                            Operand::Register(Register::R8)
                        ]);
                        self.emit_instruction(Instruction::Setge, vec![Operand::Register(Register::Al)]);
                        self.emit_instruction(Instruction::Movzx, vec![
                            Operand::Register(Register::Rax), 
                            Operand::Register(Register::Al)
                        ]);
                    }
                    TokenType::LogicalAnd => {
                        self.emit_instruction(Instruction::And, vec![
                            Operand::Register(Register::Rax), 
                            Operand::Register(Register::R8)
                        ]);
                        self.emit_instruction(Instruction::Cmp, vec![
                            Operand::Register(Register::Rax), 
                            Operand::Immediate(0)
                        ]);
                        self.emit_instruction(Instruction::Setne, vec![Operand::Register(Register::Al)]);
                        self.emit_instruction(Instruction::Movzx, vec![
                            Operand::Register(Register::Rax), 
                            Operand::Register(Register::Al)
                        ]);
                    }
                    TokenType::LogicalOr => {
                        self.emit_instruction(Instruction::Or, vec![
                            Operand::Register(Register::Rax), 
                            Operand::Register(Register::R8)
                        ]);
                        self.emit_instruction(Instruction::Cmp, vec![
                            Operand::Register(Register::Rax), 
                            Operand::Immediate(0)
                        ]);
                        self.emit_instruction(Instruction::Setne, vec![Operand::Register(Register::Al)]);
                        self.emit_instruction(Instruction::Movzx, vec![
                            Operand::Register(Register::Rax), 
                            Operand::Register(Register::Al)
                        ]);
                    }
                    _ => {
                        self.emit_line("    ; unsupported binary op");
                        self.emit_instruction(Instruction::Mov, vec![
                            Operand::Register(Register::Rax), 
                            Operand::Immediate(0)
                        ]);
                    }
                }
            }
            Expr::Call { callee, arguments: _ } => {
                // This is a generic function call.
                // For now, we'll treat it as unsupported as printf is handled by Stmt::PrintStmt.
                // A full compiler would need to resolve `callee` and pass `arguments`.
                self.emit_line(&format!("    ; unsupported general function call expression: {:?}", callee));
                self.emit_instruction(Instruction::Mov, vec![
                    Operand::Register(Register::Rax), 
                    Operand::Immediate(0)
                ]);
            }
            Expr::Assignment { name, value } => {
                // Generate code for the value expression
                self.gen_expr(value);
                
                // Store the result in the variable
                if let Some(&offset) = self.locals.get(name) {
                    self.emit_instruction(Instruction::Mov, vec![
                        Operand::Memory { base: Register::Rbp, offset },
                        Operand::Register(Register::Rax)
                    ]);
                } else {
                    self.emit_line(&format!("    ; assignment to unknown variable '{}'", name));
                }
                // Assignment expression returns the assigned value (in RAX)
            }
        }
    }

    fn get_locals(&self) -> &std::collections::HashMap<String, i32> {
        &self.locals
    }

    fn get_data_strings(&self) -> &std::collections::HashMap<String, String> {
        &self.data_strings
    }
}