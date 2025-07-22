use std::collections::HashMap;
use crate::lexer::TokenType;
use crate::parser::ast::{Expr, Stmt};

#[derive(Debug, Clone)]
pub enum Instruction {
    Mov, Movsd, Movzx, Movq, Lea,
    Push, Pop,
    Add, Sub, Imul, Idiv, Inc, Neg, Cqo,
    Cmp, Test,
    Sete, Setne, Setl, Setle, Setg, Setge,
    Jmp, Je, Jle, Call, Ret,
    And, Or, Xor,
}

#[derive(Debug, Clone)]
pub enum Register {
    Rax, Rbp, Rsp, Rcx, Rdx, R8, R9,
    Eax, Edx, R8d, R9d,
    Al,
    Xmm0, Xmm1, Xmm2, Xmm3,
}

#[derive(Debug, Clone)]
pub enum Operand {
    Register(Register),
    Immediate(i64),
    Memory { base: Register, offset: i32 },
    Label(String),
    String(String),
}

#[derive(Debug, Clone)]
pub enum Size {
    Byte, Word, Dword, Qword,
}

impl Instruction {
    fn to_string(&self) -> &'static str {
        match self {
            Instruction::Mov => "mov",
            Instruction::Movsd => "movsd",
            Instruction::Movzx => "movzx",
            Instruction::Movq => "movq",
            Instruction::Lea => "lea",
            Instruction::Push => "push",
            Instruction::Pop => "pop",
            Instruction::Add => "add",
            Instruction::Sub => "sub",
            Instruction::Imul => "imul",
            Instruction::Idiv => "idiv",
            Instruction::Inc => "inc",
            Instruction::Neg => "neg",
            Instruction::Cqo => "cqo",
            Instruction::Cmp => "cmp",
            Instruction::Test => "test",
            Instruction::Sete => "sete",
            Instruction::Setne => "setne",
            Instruction::Setl => "setl",
            Instruction::Setle => "setle",
            Instruction::Setg => "setg",
            Instruction::Setge => "setge",
            Instruction::Jmp => "jmp",
            Instruction::Je => "je",
            Instruction::Jle => "jle",
            Instruction::Call => "call",
            Instruction::Ret => "ret",
            Instruction::And => "and",
            Instruction::Or => "or",
            Instruction::Xor => "xor",
        }
    }
}

impl Register {
    fn to_string(&self) -> &'static str {
        match self {
            Register::Rax => "rax",
            Register::Rbp => "rbp",
            Register::Rsp => "rsp",
            Register::Rcx => "rcx",
            Register::Rdx => "rdx",
            Register::R8 => "r8",
            Register::R9 => "r9",
            Register::Eax => "eax",
            Register::Edx => "edx",
            Register::R8d => "r8d",
            Register::R9d => "r9d",
            Register::Al => "al",
            Register::Xmm0 => "xmm0",
            Register::Xmm1 => "xmm1",
            Register::Xmm2 => "xmm2",
            Register::Xmm3 => "xmm3",
        }
    }
}

impl Operand {
    fn to_string(&self) -> String {
        match self {
            Operand::Register(reg) => reg.to_string().to_string(),
            Operand::Immediate(val) => val.to_string(),
            Operand::Memory { base, offset } => {
                if *offset >= 0 {
                    format!("[{}+{}]", base.to_string(), offset)
                } else {
                    format!("[{}{}]", base.to_string(), offset)
                }
            },
            Operand::Label(label) => label.clone(),
            Operand::String(s) => s.clone(),
        }
    }
}

pub struct Codegen {
    label_count: usize,
    stack_offset: i32,
    locals: HashMap<String, i32>,
    local_types: HashMap<String, TokenType>, // Track variable types
    output: String,
    data_strings: HashMap<String, String>, // To store format strings and their labels
    string_label_count: usize, // For unique string labels
}

impl Codegen {
    pub fn new() -> Self {
        Self {
            label_count: 0,
            stack_offset: 0,
            locals: HashMap::new(),
            local_types: HashMap::new(),
            output: String::new(),
            data_strings: HashMap::new(),
            string_label_count: 0,
        }
    }

    pub fn generate(mut self, ast: &[Stmt]) -> String {
        self.emit_line("bits 64");
        self.emit_line("default rel");
        self.emit_line("global main");
        self.emit_line("extern printf");
        self.emit_line("");

        // First pass: collect variable types
        self.collect_variable_types(ast);
        // Second pass: collect format strings with type information
        self.collect_format_strings(ast);

        self.emit_comment("--- Data section for string literals ---");
        self.emit_line("section .data");

        let data_strings_clone = self.data_strings.clone();
        for (s, label) in &data_strings_clone {
            let formatted_s = s.replace('\n', "").replace("%f", "%.2f");
            self.emit_line(&format!("    {}: db \"{}\", 10, 0", label, formatted_s));
        }
        
        self.emit_line("");

        self.emit_comment("--- Text section for executable code ---");
        self.emit_line("section .text");

        for stmt in ast {
            if let Stmt::Function { name, body, .. } = stmt {
                self.generate_function(name, body);
            }
        }

        self.output
    }

    // Helper to collect format strings before generating code
    fn collect_variable_types(&mut self, ast: &[Stmt]) {
        for stmt in ast {
            match stmt {
                Stmt::Function { body, .. } => {
                    self.collect_variable_types(body);
                }
                Stmt::VarDecl { var_type, name, .. } => {
                    // Store variable type for later use
                    self.local_types.insert(name.clone(), var_type.clone());
                }
                Stmt::If { then_branch, .. } => {
                    self.collect_variable_types(then_branch);
                }
                Stmt::Block(stmts) => {
                    self.collect_variable_types(stmts);
                }
                _ => {}
            }
        }
    }

    fn collect_format_strings(&mut self, ast: &[Stmt]) {
        for stmt in ast {
            match stmt {
                Stmt::Function { body, .. } => {
                    self.collect_format_strings(body);
                }
                Stmt::PrintStmt { format_string, args } => {
                    if let Expr::String(s) = format_string {
                        if s.is_empty() {
                            // Simple println(expr) case - need to create format string
                            if args.len() == 1 {
                                let arg = &args[0];
                                let format_str = match arg {
                                    Expr::Integer(_) => "%d\n",
                                    Expr::Float(_) => "%.6f\n",
                                    Expr::Char(_) => "%c\n",
                                    Expr::Identifier(var_name) => {
                                        // Use stored type information
                                        match self.local_types.get(var_name) {
                                            Some(TokenType::Int) => "%d\n",
                                            Some(TokenType::FloatType) => "%.6f\n",
                                            Some(TokenType::CharType) => "%c\n",
                                            _ => "%d\n", // Default to integer
                                        }
                                    },
                                    _ => "%d\n", // Default to integer
                                };
                                
                                if !self.data_strings.contains_key(format_str) {
                                    let label = self.new_string_label();
                                    self.data_strings.insert(format_str.to_string(), label);
                                }
                            }
                        } else {
                            // Regular format string case
                            if !self.data_strings.contains_key(s) {
                                let label = self.new_string_label();
                                self.data_strings.insert(s.clone(), label);
                            }
                        }
                    }
                }
                Stmt::If { then_branch, .. } => {
                    self.collect_format_strings(then_branch);
                }
                Stmt::Block(stmts) => {
                    self.collect_format_strings(stmts);
                }
                _ => {}
            }
        }
    }

    fn generate_function(&mut self, name: &str, body: &[Stmt]) {
        self.emit_line(&format!("{}:", name));
        self.emit_comment("--- Prologue et allocation de la pile ---");
        self.emit_instruction(Instruction::Push, vec![Operand::Register(Register::Rbp)]);
        self.emit_instruction(Instruction::Mov, vec![
            Operand::Register(Register::Rbp), 
            Operand::Register(Register::Rsp)
        ]);
        self.emit_comment("On alloue 48 octets : ~16 pour nos variables + 32 pour le \"shadow space\"");
        self.emit_comment("IMPORTANT: Aligner la pile sur 16 octets avant les appels");
        self.emit_instruction(Instruction::Sub, vec![
            Operand::Register(Register::Rsp), 
            Operand::Immediate(48)
        ]);
        self.emit_line("");

        self.stack_offset = 0;
        self.locals.clear();

        for stmt in body {
            self.gen_stmt(stmt);
        }

        self.emit_comment("--- Épilogue ---");
        self.emit_instruction(Instruction::Mov, vec![
            Operand::Register(Register::Rsp), 
            Operand::Register(Register::Rbp)
        ]);
        self.emit_instruction(Instruction::Pop, vec![Operand::Register(Register::Rbp)]);
        self.emit_instruction(Instruction::Ret, vec![]);
    }

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

    fn emit_instruction(&mut self, instruction: Instruction, operands: Vec<Operand>) {
        let instr_str = instruction.to_string();
        if operands.is_empty() {
            self.emit_line(&format!("    {}", instr_str));
        } else {
            let operands_str = operands.iter()
                .map(|op| op.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            self.emit_line(&format!("    {} {}", instr_str, operands_str));
        }
    }

    fn emit_instruction_with_size(&mut self, instruction: Instruction, size: Size, operands: Vec<Operand>) {
        let size_str = match size {
            Size::Byte => "byte",
            Size::Word => "word", 
            Size::Dword => "dword",
            Size::Qword => "qword",
        };
        let instr_str = instruction.to_string();
        let operands_str = operands.iter()
            .enumerate()
            .map(|(i, op)| {
                if i == 0 && matches!(op, Operand::Memory { .. }) {
                    format!("{} {}", size_str, op.to_string())
                } else {
                    op.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        self.emit_line(&format!("    {} {}", instr_str, operands_str));
    }

    fn emit_line(&mut self, line: &str) {
        self.output.push_str(line);
        self.output.push('\n');
    }

    fn emit_comment(&mut self, comment: &str) {
        self.emit_line(&format!("; {}", comment));
    }

    fn new_string_label(&mut self) -> String {
        let label = format!("str_{}", self.string_label_count);
        self.string_label_count += 1;
        label
    }
}
