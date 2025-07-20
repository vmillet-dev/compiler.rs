use std::collections::HashMap;
use crate::lexer::TokenType;
use crate::parser::ast::{Expr, Stmt};

pub struct Codegen {
    label_count: usize,
    stack_offset: i32,
    locals: HashMap<String, i32>,
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
            output: String::new(),
            data_strings: HashMap::new(),
            string_label_count: 0,
        }
    }

    pub fn generate(mut self, ast: &[Stmt]) -> String {
        self.emit_line("global main");
        self.emit_line("extern printf");

        self.collect_format_strings(ast);

        self.emit_line("section .data");

        // Cloner les données pour éviter le conflit d'emprunt
        let data_strings_clone = self.data_strings.clone();
        for (s, label) in &data_strings_clone {
            let escaped_s = s.chars()
                .map(|c| match c {
                    '\n' => "\\n".to_string(),
                    '\t' => "\\t".to_string(),
                    '\r' => "\\r".to_string(),
                    '"' => "\\\"".to_string(),
                    '\\' => "\\\\".to_string(),
                    _ => c.to_string(),
                })
                .collect::<String>();
            self.emit_line(&format!("{}: db \"{}\", 0", label, escaped_s));
        }

        self.emit_line("section .text");

        for stmt in ast {
            if let Stmt::Function { name, body, .. } = stmt {
                self.generate_function(name, body);
            }
        }

        self.output
    }

    // Helper to collect format strings before generating code
    fn collect_format_strings(&mut self, ast: &[Stmt]) {
        for stmt in ast {
            match stmt {
                Stmt::Function { body, .. } => {
                    self.collect_format_strings(body);
                }
                Stmt::PrintStmt { format_string, .. } => {
                    if let Expr::String(s) = format_string {
                        if !self.data_strings.contains_key(s) {
                            let label = self.new_string_label();
                            self.data_strings.insert(s.clone(), label);
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
        self.emit_line("    push rbp");
        self.emit_line("    mov rbp, rsp");

        self.stack_offset = 0;
        self.locals.clear();

        for stmt in body {
            self.gen_stmt(stmt);
        }

        self.emit_line("    mov rsp, rbp");
        self.emit_line("    pop rbp");
        self.emit_line("    ret");
    }

    fn gen_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl { var_type, name, initializer } => {
                // Determine size based on type (for simplicity, assume 8 bytes for all for now)
                let var_size = match var_type {
                    TokenType::Int | TokenType::FloatType => 8, // 64-bit
                    TokenType::CharType => 1, // 8-bit
                    _ => 8, // Default to 8 bytes for unknown types
                };
                self.stack_offset -= var_size; // Adjust stack offset

                // Store offset relative to RBP
                self.locals.insert(name.clone(), self.stack_offset);

                if let Some(expr) = initializer {
                    self.gen_expr(expr);
                    // Store the value in the allocated stack space
                    match var_size {
                        8 => self.emit_line(&format!("    mov QWORD [rbp{}], rax", self.stack_offset)),
                        1 => self.emit_line(&format!("    mov BYTE [rbp{}], al", self.stack_offset)),
                        _ => self.emit_line(&format!("    ; unsupported variable size for assignment")),
                    }
                }
            }

            Stmt::Return(Some(expr)) => {
                self.gen_expr(expr);
                self.emit_line("    mov rsp, rbp");
                self.emit_line("    pop rbp");
                self.emit_line("    ret");
            }

            Stmt::Return(None) => {
                self.emit_line("    mov rax, 0");
                self.emit_line("    mov rsp, rbp");
                self.emit_line("    pop rbp");
                self.emit_line("    ret");
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
                let label = self.new_label("endif");
                self.gen_expr(condition);
                self.emit_line("    cmp rax, 0");
                self.emit_line(&format!("    je {}", label));
                for stmt in then_branch {
                    self.gen_stmt(stmt);
                }
                self.emit_line(&format!("{}:", label));
            }

            // Handle PrintStmt with RIP-relative addressing for x86-64
            Stmt::PrintStmt { format_string, args } => {
                if let Expr::String(s) = format_string {
                    let format_label = self.data_strings.get(s).unwrap().clone();
                    let args_len = args.len();

                    let arg_regs = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
                    let total_args = args_len + 1; // +1 pour la format string

                    // 1. Charger les arguments dans les registres (maximum 6)
                    let reg_args_count = std::cmp::min(total_args, 6);

                    // CORRECTION: Utiliser RIP-relative addressing au lieu de LEA direct
                    // Charger la format string dans RDI (toujours le premier argument)
                    self.emit_line(&format!("    lea rdi, [rel {}]", format_label));

                    // Charger les autres arguments dans les registres RSI, RDX, RCX, R8, R9
                    for i in 0..(reg_args_count - 1) { // -1 car RDI est déjà utilisé pour format_string
                        if i < args_len {
                            self.gen_expr(&args[i]);
                            self.emit_line(&format!("    mov {}, rax", arg_regs[i + 1]));
                        }
                    }

                    // Pousser seulement les arguments excédentaires sur la pile
                    if total_args > 6 {
                        // Pousser les arguments excédentaires en ordre inverse
                        for i in (6..total_args).rev() {
                            let arg_index = i - 1; // -1 car la format string n'est pas dans args[]
                            self.gen_expr(&args[arg_index]);
                            self.emit_line("    push rax");
                        }
                    }

                    // Calculer l'alignement de pile correctement
                    let stack_args = if total_args > 6 { total_args - 6 } else { 0 };
                    let alignment_needed = (stack_args % 2) * 8;
                    if alignment_needed > 0 {
                        self.emit_line("    sub rsp, 8");
                    }

                    // RAX = 0 (pas de registres XMM utilisés)
                    self.emit_line("    mov rax, 0");

                    // Appeler printf
                    self.emit_line("    call printf");

                    // Nettoyer la pile correctement
                    // Restaurer l'alignement si nécessaire
                    if alignment_needed > 0 {
                        self.emit_line("    add rsp, 8");
                    }

                    // Nettoyer les arguments poussés sur la pile
                    if stack_args > 0 {
                        self.emit_line(&format!("    add rsp, {}", stack_args * 8));
                    }

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
                self.emit_line(&format!("    mov rax, {}", i));
            }
            Expr::Float(f) => {
                // Pour les floats, nous devons les gérer correctement pour printf
                // Nous allons créer une approche hybride :
                // - Stocker le float dans la section .data
                // - Charger sa valeur dans un registre XMM pour printf
                let float_bits = f.to_bits();
                self.emit_line(&format!("    mov rax, {}", float_bits));
                // Pour printf avec %f, nous devrions utiliser movq xmm0, rax
                // mais cela nécessiterait de suivre quel registre XMM utiliser
                // Pour l'instant, gardons la représentation en bits
            }
            Expr::Char(c) => {
                self.emit_line(&format!("    mov rax, {}", *c as u8)); // Move ASCII value
            }
            Expr::String(s) => {
                // CORRECTION: Utiliser RIP-relative addressing pour les chaînes
                if let Some(label) = self.data_strings.get(s) {
                    self.emit_line(&format!("    lea rax, [rel {}]", label));
                } else {
                    // This should not happen if collect_format_strings is called correctly
                    self.emit_line(&format!("    ; String literal '{}' not found in data section", s));
                    self.emit_line("    mov rax, 0"); // Default to null pointer
                }
            }
            Expr::Identifier(name) => {
                if let Some(offset) = self.locals.get(name) {
                    // Load value from stack into RAX
                    // Need to consider variable size (BYTE, WORD, DWORD, QWORD)
                    // For now, assume QWORD for all identifiers for simplicity.
                    self.emit_line(&format!("    mov rax, [rbp{}]", offset));
                } else {
                    self.emit_line(&format!("    ; unknown variable '{}'", name));
                    self.emit_line("    mov rax, 0"); // Default to 0 if variable not found
                }
            }

            Expr::Binary { left, operator, right } => {
                match operator {
                    TokenType::LogicalNot => { // Unary '!'
                        self.gen_expr(right); // Evaluate the operand
                        self.emit_line("    cmp rax, 0"); // Compare with 0
                        self.emit_line("    sete al");    // Set AL to 1 if RAX == 0 (true for logical NOT)
                        self.emit_line("    movzx rax, al"); // Zero-extend AL to RAX
                    }
                    TokenType::Minus if matches!(**left, Expr::Integer(0)) => { // Unary '-' (placeholder left operand)
                        self.gen_expr(right); // Evaluate the operand
                        self.emit_line("    neg rax"); // Negate RAX
                    }
                    _ => { // Binary operators
                        self.gen_expr(right);
                        self.emit_line("    push rax");
                        self.gen_expr(left);
                        self.emit_line("    pop rcx");

                        match operator {
                            TokenType::Plus => self.emit_line("    add rax, rcx"),
                            TokenType::Minus => self.emit_line("    sub rax, rcx"),
                            TokenType::Multiply => self.emit_line("    imul rax, rcx"),
                            TokenType::Divide => {
                                self.emit_line("    cqo");
                                self.emit_line("    idiv rcx");
                            }
                            TokenType::Equal => {
                                self.emit_line("    cmp rax, rcx");
                                self.emit_line("    sete al");
                                self.emit_line("    movzx rax, al");
                            }
                            TokenType::NotEqual => {
                                self.emit_line("    cmp rax, rcx");
                                self.emit_line("    setne al");
                                self.emit_line("    movzx rax, al");
                            }
                            TokenType::LessThan => {
                                self.emit_line("    cmp rax, rcx");
                                self.emit_line("    setl al");
                                self.emit_line("    movzx rax, al");
                            }
                            TokenType::LessEqual => {
                                self.emit_line("    cmp rax, rcx");
                                self.emit_line("    setle al");
                                self.emit_line("    movzx rax, al");
                            }
                            TokenType::GreaterThan => {
                                self.emit_line("    cmp rax, rcx");
                                self.emit_line("    setg al");
                                self.emit_line("    movzx rax, al");
                            }
                            TokenType::GreaterEqual => {
                                self.emit_line("    cmp rax, rcx");
                                self.emit_line("    setge al");
                                self.emit_line("    movzx rax, al");
                            }
                            TokenType::LogicalAnd => {
                                self.emit_line("    and rax, rcx");
                                self.emit_line("    cmp rax, 0"); // If both non-zero, result is non-zero
                                self.emit_line("    setne al");
                                self.emit_line("    movzx rax, al");
                            }
                            TokenType::LogicalOr => {
                                self.emit_line("    or rax, rcx");
                                self.emit_line("    cmp rax, 0"); // If either non-zero, result is non-zero
                                self.emit_line("    setne al");
                                self.emit_line("    movzx rax, al");
                            }
                            _ => {
                                self.emit_line("    ; unsupported binary op");
                                self.emit_line("    mov rax, 0"); // Default value
                            }
                        }
                    }
                }
            }
            Expr::Call { callee, arguments } => {
                // This is a generic function call.
                // For now, we'll treat it as unsupported as printf is handled by Stmt::PrintStmt.
                // A full compiler would need to resolve `callee` and pass `arguments`.
                self.emit_line(&format!("    ; unsupported general function call expression: {:?}", callee));
                self.emit_line("    mov rax, 0");
            }
            _ => {
                self.emit_line(&format!("    ; unsupported expr {:?}", expr));
                self.emit_line("    mov rax, 0");
            }
        }
    }

    fn emit_line(&mut self, line: &str) {
        self.output.push_str(line);
        self.output.push('\n');
    }

    fn new_label(&mut self, base: &str) -> String {
        let label = format!("{}_{}", base, self.label_count);
        self.label_count += 1;
        label
    }

    fn new_string_label(&mut self) -> String {
        let label = format!("str_{}", self.string_label_count);
        self.string_label_count += 1;
        label
    }
}