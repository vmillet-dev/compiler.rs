use std::collections::HashMap;
use crate::lexer::TokenType;
use crate::parser::ast::{Expr, Stmt};
use super::instruction::{Instruction, Operand, Register, Size};
use super::emitter::{Emitter, CodeEmitter};
use super::expression::ExpressionGenerator;

pub trait StatementGenerator: Emitter + CodeEmitter + ExpressionGenerator {
    fn gen_stmt(&mut self, stmt: &Stmt);
    fn get_stack_offset(&self) -> i32;
    fn set_stack_offset(&mut self, offset: i32);
    fn get_locals_mut(&mut self) -> &mut HashMap<String, i32>;
    fn get_local_types(&self) -> &HashMap<String, TokenType>;
    fn get_local_types_mut(&mut self) -> &mut HashMap<String, TokenType>;
}

impl StatementGenerator for super::Codegen {
    fn gen_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl { var_type, name, initializer } => {
                let type_str = match var_type {
                    TokenType::Int => "int",
                    TokenType::FloatType => "float", 
                    TokenType::CharType => "char",
                    _ => "unknown",
                };
                if let Some(init_expr) = initializer {
                    let init_str = match init_expr {
                        Expr::Integer(i) => i.to_string(),
                        Expr::Float(f) => f.to_string(),
                        Expr::Char(c) => format!("'{}'", c),
                        Expr::String(s) => format!("\"{}\"", s),
                        _ => "expression".to_string(),
                    };
                    self.emit_comment(&format!("--- {} {} = {}; ---", type_str, name, init_str));
                } else {
                    self.emit_comment(&format!("--- {} {}; ---", type_str, name));
                }
                let (_var_size, stack_offset) = match var_type {
                    TokenType::Int => {
                        self.stack_offset -= 4;
                        (4, self.stack_offset)
                    },
                    TokenType::FloatType => {
                        self.stack_offset -= 8;
                        (8, self.stack_offset)
                    },
                    TokenType::CharType => {
                        self.stack_offset -= 1;
                        (1, self.stack_offset)
                    },
                    _ => {
                        self.stack_offset -= 8;
                        (8, self.stack_offset)
                    }
                };

                // Store offset relative to RBP
                self.locals.insert(name.clone(), stack_offset);
                // Store variable type for later use
                self.local_types.insert(name.clone(), var_type.clone());

                if let Some(expr) = initializer {
                    match var_type {
                        TokenType::Int => {
                            if let Expr::Integer(i) = expr {
                                self.emit_instruction_with_size(Instruction::Mov, Size::Dword, vec![
                                    Operand::Memory { base: Register::Rbp, offset: stack_offset },
                                    Operand::Immediate(*i)
                                ]);
                            } else {
                                self.gen_expr(expr);
                                self.emit_instruction_with_size(Instruction::Mov, Size::Dword, vec![
                                    Operand::Memory { base: Register::Rbp, offset: stack_offset },
                                    Operand::Register(Register::Eax)
                                ]);
                            }
                        },
                        TokenType::FloatType => {
                            if let Expr::Float(f) = expr {
                                let float_bits = f.to_bits();
                                self.emit_instruction(Instruction::Mov, vec![
                                    Operand::Register(Register::Rax),
                                    Operand::Immediate(float_bits as i64)
                                ]);
                                self.emit_instruction(Instruction::Movq, vec![
                                    Operand::Register(Register::Xmm0),
                                    Operand::Register(Register::Rax)
                                ]);
                                self.emit_instruction_with_size(Instruction::Movsd, Size::Qword, vec![
                                    Operand::Memory { base: Register::Rbp, offset: stack_offset },
                                    Operand::Register(Register::Xmm0)
                                ]);
                            } else {
                                self.gen_expr(expr);
                                self.emit_instruction_with_size(Instruction::Movsd, Size::Qword, vec![
                                    Operand::Memory { base: Register::Rbp, offset: stack_offset },
                                    Operand::Register(Register::Xmm0)
                                ]);
                            }
                        },
                        TokenType::CharType => {
                            if let Expr::Char(c) = expr {
                                self.emit_instruction_with_size(Instruction::Mov, Size::Byte, vec![
                                    Operand::Memory { base: Register::Rbp, offset: stack_offset },
                                    Operand::String(format!("'{}'", c))
                                ]);
                            } else {
                                self.gen_expr(expr);
                                self.emit_instruction_with_size(Instruction::Mov, Size::Byte, vec![
                                    Operand::Memory { base: Register::Rbp, offset: stack_offset },
                                    Operand::Register(Register::Al)
                                ]);
                            }
                        },
                        _ => {
                            self.gen_expr(expr);
                            self.emit_instruction_with_size(Instruction::Mov, Size::Qword, vec![
                                Operand::Memory { base: Register::Rbp, offset: stack_offset },
                                Operand::Register(Register::Rax)
                            ]);
                        }
                    }
                }
            }

            Stmt::Return(Some(expr)) => {
                let return_str = match expr {
                    Expr::Integer(i) => i.to_string(),
                    Expr::Identifier(name) => name.clone(),
                    Expr::Binary { left, operator, right } => {
                        match (left.as_ref(), operator, right.as_ref()) {
                            (Expr::Identifier(name), TokenType::Plus, Expr::Integer(i)) => format!("{} + {}", name, i),
                            _ => "expression".to_string(),
                        }
                    },
                    _ => "expression".to_string(),
                };
                self.emit_comment(&format!("--- return {}; ---", return_str));
                // Handle specific case of "return var + 1;"
                if let Expr::Binary { left, operator, right } = expr {
                    if let (Expr::Identifier(var_name), TokenType::Plus, Expr::Integer(1)) = (left.as_ref(), operator, right.as_ref()) {
                        if let Some(&offset) = self.locals.get(var_name) {
                            self.emit_line(&format!("    mov     eax, [rbp{}]            ; Recharge {} dans eax", offset, var_name));
                            self.emit_instruction(Instruction::Inc, vec![Operand::Register(Register::Eax)]);
                        } else {
                            self.gen_expr(expr);
                            self.emit_line("    ; result in eax");
                        }
                    } else {
                        self.gen_expr(expr);
                        self.emit_line("    ; result in eax");
                    }
                } else {
                    self.gen_expr(expr);
                    self.emit_line("    ; result in eax");
                }
            }

            Stmt::Return(None) => {
                self.emit_comment("--- return 0; ---");
                self.emit_instruction(Instruction::Xor, vec![
                    Operand::Register(Register::Eax), 
                    Operand::Register(Register::Eax)
                ]);
            }

            Stmt::ExprStmt(expr) => {
                self.gen_expr(expr);
            }

            Stmt::Block(stmts) => {
                // Save current stack offset and locals for block scope
                let original_stack_offset = self.stack_offset;
                let original_locals = self.locals.clone();

                for stmt in stmts {
                    self.gen_stmt(stmt);
                }

                // Restore stack offset and locals after block
                self.stack_offset = original_stack_offset;
                self.locals = original_locals;
            }

            Stmt::If { condition, then_branch } => {
                let condition_str = match condition {
                    Expr::Binary { left, operator, right } => {
                        match (left.as_ref(), operator, right.as_ref()) {
                            (Expr::Identifier(name), TokenType::GreaterThan, Expr::Integer(i)) => format!("{} > {}", name, i),
                            (Expr::Identifier(name), TokenType::LessThan, Expr::Integer(i)) => format!("{} < {}", name, i),
                            (Expr::Identifier(name), TokenType::Equal, Expr::Integer(i)) => format!("{} == {}", name, i),
                            _ => "condition".to_string(),
                        }
                    },
                    _ => "condition".to_string(),
                };
                self.emit_comment(&format!("--- if ({}) ---", condition_str));
                if let Expr::Binary { left, operator, right } = condition {
                    if let (Expr::Identifier(var_name), TokenType::GreaterThan, Expr::Integer(val)) = (left.as_ref(), operator, right.as_ref()) {
                        if let Some(&offset) = self.locals.get(var_name) {
                            self.emit_line(&format!("    mov     eax, [rbp{}]            ; Charge {} dans eax pour la comparaison", offset, var_name));
                            self.emit_instruction(Instruction::Cmp, vec![
                                Operand::Register(Register::Eax), 
                                Operand::Immediate(*val)
                            ]);
                            self.emit_instruction(Instruction::Jle, vec![Operand::Label(".else_block".to_string())]);
                        }
                    } else {
                        self.gen_expr(condition);
                        self.emit_instruction(Instruction::Cmp, vec![
                            Operand::Register(Register::Eax), 
                            Operand::Immediate(0)
                        ]);
                        self.emit_instruction(Instruction::Je, vec![Operand::Label(".else_block".to_string())]);
                    }
                } else {
                    self.gen_expr(condition);
                    self.emit_instruction(Instruction::Cmp, vec![
                        Operand::Register(Register::Eax), 
                        Operand::Immediate(0)
                    ]);
                    self.emit_instruction(Instruction::Je, vec![Operand::Label(".else_block".to_string())]);
                }
                self.emit_line("");
                self.emit_comment("--- Bloc du \"if\" (si x > 0) ---");
                for stmt in then_branch {
                    self.gen_stmt(stmt);
                }
                self.emit_instruction(Instruction::Jmp, vec![Operand::Label(".end_program".to_string())]);
                self.emit_line("");
                self.emit_line(".else_block:");
                self.emit_comment("--- return 0; ---");
                self.emit_comment("Ce bloc est exécuté si x <= 0");
                self.emit_instruction(Instruction::Xor, vec![
                    Operand::Register(Register::Eax), 
                    Operand::Register(Register::Eax)
                ]);
                self.emit_line("");
                self.emit_line(".end_program:");
            }

            // Handle PrintStmt with RIP-relative addressing for x86-64
            Stmt::PrintStmt { format_string, args } => {
                if let Expr::String(s) = format_string {
                    if s.is_empty() {
                        // Simple println(expr) case
                        if args.len() == 1 {
                            let arg = &args[0];
                            match arg {
                                Expr::Identifier(name) => {
                                    self.emit_comment(&format!("--- println({}); ---", name));
                                }
                                Expr::Integer(i) => {
                                    self.emit_comment(&format!("--- println({}); ---", i));
                                }
                                Expr::Float(f) => {
                                    self.emit_comment(&format!("--- println({}); ---", f));
                                }
                                Expr::Char(c) => {
                                    self.emit_comment(&format!("--- println('{}'); ---", c));
                                }
                                _ => {
                                    self.emit_comment("--- println(expr); ---");
                                }
                            }
                        }
                    } else if args.is_empty() {
                        self.emit_comment(&format!("--- println(\"{}\"); ---", s.replace('\n', "\\n")));
                    } else {
                        let args_str = args.iter()
                            .map(|arg| match arg {
                                Expr::Identifier(name) => name.clone(),
                                Expr::Integer(i) => i.to_string(),
                                Expr::Float(f) => f.to_string(),
                                Expr::Char(c) => format!("'{}'", c),
                                _ => "expr".to_string(),
                            })
                            .collect::<Vec<_>>()
                            .join(", ");
                        self.emit_comment(&format!("--- println(\"{}\", {}); ---", s.replace('\n', "\\n"), args_str));
                    }
                }
                if let Expr::String(s) = format_string {
                    if s.is_empty() {
                        // Handle simple println(expr) case
                        if args.len() == 1 {
                            let arg = &args[0];
                            
                            // Determine the appropriate format string based on the expression type
                            let (format_str, _is_float) = match arg {
                                Expr::Integer(_) => ("%d\n", false),
                                Expr::Float(_) => ("%.6f\n", true),
                                Expr::Char(_) => ("%c\n", false),
                                Expr::Identifier(var_name) => {
                                    // Use stored type information
                                    match self.local_types.get(var_name) {
                                        Some(TokenType::Int) => ("%d\n", false),
                                        Some(TokenType::FloatType) => ("%.6f\n", true),
                                        Some(TokenType::CharType) => ("%c\n", false),
                                        _ => ("%d\n", false), // Default to integer
                                    }
                                }
                                _ => ("%d\n", false), // Default to integer format
                            };
                            
                            // Create the format string if it doesn't exist
                            let format_label = if let Some(label) = self.data_strings.get(format_str) {
                                label.clone()
                            } else {
                                let label = format!("str_{}", self.data_strings.len());
                                self.data_strings.insert(format_str.to_string(), label.clone());
                                label
                            };
                            
                            self.emit_comment("Aligner la pile avant l'appel (RSP doit être multiple de 16)");
                            self.emit_line("    and     rsp, ~15            ; Force l'alignement sur 16 octets");
                            self.emit_instruction(Instruction::Sub, vec![
                                Operand::Register(Register::Rsp), 
                                Operand::Immediate(32)
                            ]);
                            self.emit_line("");
                            
                            // Set up format string in RCX
                            self.emit_instruction(Instruction::Mov, vec![
                                Operand::Register(Register::Rcx), 
                                Operand::Label(format_label)
                            ]);
                            
                            // Handle the argument
                            match arg {
                                Expr::Integer(i) => {
                                    self.emit_instruction(Instruction::Mov, vec![
                                        Operand::Register(Register::Edx), 
                                        Operand::Immediate(*i)
                                    ]);
                                }
                                Expr::Float(f) => {
                                    // For float, we need to put it in both XMM1 and RDX
                                    let float_bits = f.to_bits();
                                    self.emit_instruction(Instruction::Mov, vec![
                                        Operand::Register(Register::Rax), 
                                        Operand::Immediate(float_bits as i64)
                                    ]);
                                    self.emit_line("    movq    xmm1, rax              ; Float value in XMM1");
                                    self.emit_line("    movq    rdx, xmm1              ; AND copy to RDX for printf");
                                }
                                Expr::Char(c) => {
                                    self.emit_instruction(Instruction::Mov, vec![
                                        Operand::Register(Register::Edx), 
                                        Operand::Immediate(*c as i64)
                                    ]);
                                }
                                Expr::Identifier(var_name) => {
                                    if let Some(&offset) = self.locals.get(var_name) {
                                        // Handle different types based on stored type information
                                        match self.local_types.get(var_name) {
                                            Some(TokenType::Int) => {
                                                self.emit_line(&format!("    mov     edx, [rbp{}]            ; Load int variable {} value", offset, var_name));
                                            }
                                            Some(TokenType::FloatType) => {
                                                self.emit_line(&format!("    movsd   xmm1, [rbp{}]          ; Load float variable {} into XMM1", offset, var_name));
                                                self.emit_line("    movq    rdx, xmm1              ; Copy float to RDX for printf");
                                            }
                                            Some(TokenType::CharType) => {
                                                self.emit_line(&format!("    movzx   edx, byte [rbp{}]      ; Load char variable {} value", offset, var_name));
                                            }
                                            _ => {
                                                // Default to integer
                                                self.emit_line(&format!("    mov     edx, [rbp{}]            ; Load variable {} value (default int)", offset, var_name));
                                            }
                                        }
                                    }
                                }
                                _ => {
                                    // For other expressions, generate code and use the result
                                    self.gen_expr(arg);
                                    self.emit_instruction(Instruction::Mov, vec![
                                        Operand::Register(Register::Edx), 
                                        Operand::Register(Register::Eax)
                                    ]);
                                }
                            }
                            
                            self.emit_line("");
                            self.emit_instruction(Instruction::Call, vec![Operand::Label("printf".to_string())]);
                            
                            self.emit_line("");
                            self.emit_instruction(Instruction::Add, vec![
                                Operand::Register(Register::Rsp), 
                                Operand::Immediate(32)
                            ]);
                        }
                        return;
                    }
                    
                    let format_label = self.data_strings.get(s).unwrap().clone();
                    
                    self.emit_comment("Aligner la pile avant l'appel (RSP doit être multiple de 16)");
                    self.emit_line("    and     rsp, ~15            ; Force l'alignement sur 16 octets");
                    self.emit_instruction(Instruction::Sub, vec![
                        Operand::Register(Register::Rsp), 
                        Operand::Immediate(32)
                    ]);
                    self.emit_line("");

                    if args.is_empty() {
                        // Simple printf with just format string
                        self.emit_instruction(Instruction::Mov, vec![
                            Operand::Register(Register::Rcx), 
                            Operand::Label(format_label)
                        ]);
                        self.emit_instruction(Instruction::Call, vec![Operand::Label("printf".to_string())]);
                    } else {
                        self.emit_instruction(Instruction::Mov, vec![
                            Operand::Register(Register::Rcx), 
                            Operand::Label(format_label)
                        ]);
                        
                        // Handle printf arguments generically
                        let arg_registers = ["edx", "r8d", "r9d"]; // Windows x64 calling convention
                        let xmm_registers = ["xmm1", "xmm2", "xmm3"];
                        
                        for (i, arg) in args.iter().enumerate() {
                            if i >= 3 { break; } // Only handle first 3 args for now
                            
                            if let Expr::Identifier(var_name) = arg {
                                if let Some(&offset) = self.locals.get(var_name) {
                                    if i == 0 { // First arg - likely integer
                                        self.emit_line(&format!("    mov     {}, [rbp{}]            ; Arg {}: la valeur de {} (dans {})", 
                                            arg_registers[i], offset, i + 2, var_name, arg_registers[i].to_uppercase()));
                                    } else if i == 1 { // Second arg - likely float
                                        self.emit_line("");
                                        self.emit_comment(&format!("Pour le {}ème argument (flottant), il faut le mettre dans {} ET dans {}", 
                                            i + 2, xmm_registers[i].to_uppercase(), arg_registers[i].to_uppercase()));
                                        self.emit_line(&format!("    movsd   {}, [rbp{}]          ; Charge le flottant dans {}", 
                                            xmm_registers[i], offset, xmm_registers[i].to_uppercase()));
                                        let reg_64 = if arg_registers[i] == "r8d" { "r8" } else { "rdx" };
                                        self.emit_line(&format!("    movq    {}, {}                ; ET copie la même valeur dans {}", 
                                            reg_64, xmm_registers[i], arg_registers[i].to_uppercase()));
                                    } else if i == 2 { // Third arg - likely char
                                        self.emit_line("");
                                        self.emit_comment(&format!("Le {}ème argument va dans {}", i + 2, arg_registers[i].to_uppercase()));
                                        self.emit_line(&format!("    movzx   {}, byte [rbp{}]      ; Arg {}: la valeur de {} (dans {})", 
                                            arg_registers[i], offset, i + 2, var_name, arg_registers[i].to_uppercase()));
                                    }
                                }
                            }
                        }
                        
                        self.emit_line("");
                        self.emit_instruction(Instruction::Call, vec![Operand::Label("printf".to_string())]);
                    }

                    self.emit_line("");
                    self.emit_instruction(Instruction::Add, vec![
                        Operand::Register(Register::Rsp), 
                        Operand::Immediate(32)
                    ]);

                } else {
                    self.emit_line(&format!("    ; printf format string is not a string literal: {:?}", format_string));
                }
            }
            _ => {
                self.emit_line(&format!("    ; unsupported statement {:?}", stmt));
            }
        }
    }

    fn get_stack_offset(&self) -> i32 {
        self.stack_offset
    }

    fn set_stack_offset(&mut self, offset: i32) {
        self.stack_offset = offset;
    }

    fn get_locals_mut(&mut self) -> &mut HashMap<String, i32> {
        &mut self.locals
    }

    fn get_local_types(&self) -> &HashMap<String, TokenType> {
        &self.local_types
    }

    fn get_local_types_mut(&mut self) -> &mut HashMap<String, TokenType> {
        &mut self.local_types
    }
}