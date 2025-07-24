use crate::lexer::TokenType;
use crate::parser::ast::Expr;
use super::instruction::{Instruction, Operand, Register};
use super::emitter::{Emitter, CodeEmitter};

pub trait ExpressionGenerator: Emitter + CodeEmitter {
    fn gen_expr(&mut self, expr: &Expr);
    fn get_locals(&self) -> &std::collections::HashMap<String, i32>;
    fn get_data_strings(&self) -> &std::collections::HashMap<String, String>;
}

impl ExpressionGenerator for super::Codegen {
    fn gen_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Integer(i) => {
                self.emit_instruction(Instruction::Mov, vec![
                    Operand::Register(Register::Rax), 
                    Operand::Immediate(*i)
                ]);
            }
            Expr::Float(f) => {
                // Pour les floats, nous devons les gérer correctement pour printf
                // Nous allons créer une approche hybride :
                // - Stocker le float dans la section .data
                // - Charger sa valeur dans un registre XMM pour printf
                let float_bits = f.to_bits();
                self.emit_instruction(Instruction::Mov, vec![
                    Operand::Register(Register::Rax), 
                    Operand::Immediate(float_bits as i64)
                ]);
                // Pour printf avec %f, nous devrions utiliser movq xmm0, rax
                // mais cela nécessiterait de suivre quel registre XMM utiliser
                // Pour l'instant, gardons la représentation en bits
            }
            Expr::Char(c) => {
                self.emit_instruction(Instruction::Mov, vec![
                    Operand::Register(Register::Rax), 
                    Operand::Immediate(*c as i64)
                ]);
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
                    // Load value from stack into RAX
                    // Need to consider variable size (BYTE, WORD, DWORD, QWORD)
                    // For now, assume QWORD for all identifiers for simplicity.
                    self.emit_instruction(Instruction::Mov, vec![
                        Operand::Register(Register::Rax), 
                        Operand::Memory { base: Register::Rbp, offset }
                    ]);
                } else {
                    self.emit_line(&format!("    ; unknown variable '{}'", name));
                    self.emit_instruction(Instruction::Mov, vec![
                        Operand::Register(Register::Rax), 
                        Operand::Immediate(0)
                    ]);
                }
            }
            Expr::Unary { operator, operand } => {
                match operator {
                    TokenType::LogicalNot => { // Unary '!'
                        self.gen_expr(operand); // Evaluate the operand
                        self.emit_instruction(Instruction::Cmp, vec![
                            Operand::Register(Register::Rax), 
                            Operand::Immediate(0)
                        ]);
                        self.emit_instruction(Instruction::Sete, vec![Operand::Register(Register::Al)]);
                        self.emit_instruction(Instruction::Movzx, vec![
                            Operand::Register(Register::Rax), 
                            Operand::Register(Register::Al)
                        ]);
                    }
                    TokenType::Minus => { // Unary '-'
                        self.gen_expr(operand); // Evaluate the operand
                        self.emit_instruction(Instruction::Neg, vec![Operand::Register(Register::Rax)]);
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
                // Binary operators
                self.gen_expr(right);
                self.emit_instruction(Instruction::Push, vec![Operand::Register(Register::Rax)]);
                self.gen_expr(left);
                self.emit_instruction(Instruction::Pop, vec![Operand::Register(Register::R8)]);

                match operator {
                    TokenType::Plus => self.emit_instruction(Instruction::Add, vec![
                        Operand::Register(Register::Rax), 
                        Operand::Register(Register::R8)
                    ]),
                    TokenType::Minus => self.emit_instruction(Instruction::Sub, vec![
                        Operand::Register(Register::Rax), 
                        Operand::Register(Register::R8)
                    ]),
                    TokenType::Multiply => self.emit_instruction(Instruction::Imul, vec![
                        Operand::Register(Register::Rax), 
                        Operand::Register(Register::R8)
                    ]),
                    TokenType::Divide => {
                        self.emit_instruction(Instruction::Cqo, vec![]);
                        self.emit_instruction(Instruction::Idiv, vec![Operand::Register(Register::R8)]);
                    }
                    TokenType::Equal => {
                        self.emit_instruction(Instruction::Cmp, vec![
                            Operand::Register(Register::Rax), 
                            Operand::Register(Register::R8)
                        ]);
                        self.emit_instruction(Instruction::Sete, vec![Operand::Register(Register::Al)]);
                        self.emit_instruction(Instruction::Movzx, vec![
                            Operand::Register(Register::Rax), 
                            Operand::Register(Register::Al)
                        ]);
                    }
                    TokenType::NotEqual => {
                        self.emit_instruction(Instruction::Cmp, vec![
                            Operand::Register(Register::Rax), 
                            Operand::Register(Register::R8)
                        ]);
                        self.emit_instruction(Instruction::Setne, vec![Operand::Register(Register::Al)]);
                        self.emit_instruction(Instruction::Movzx, vec![
                            Operand::Register(Register::Rax), 
                            Operand::Register(Register::Al)
                        ]);
                    }
                    TokenType::LessThan => {
                        self.emit_instruction(Instruction::Cmp, vec![
                            Operand::Register(Register::Rax), 
                            Operand::Register(Register::R8)
                        ]);
                        self.emit_instruction(Instruction::Setl, vec![Operand::Register(Register::Al)]);
                        self.emit_instruction(Instruction::Movzx, vec![
                            Operand::Register(Register::Rax), 
                            Operand::Register(Register::Al)
                        ]);
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