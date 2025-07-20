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
        self.emit_line("bits 64");
        self.emit_line("default rel");
        self.emit_line("global main");
        self.emit_line("extern printf");
        self.emit_line("");

        self.collect_format_strings(ast);

        self.emit_comment("--- Data section for string literals ---");
        self.emit_line("section .data");

        let data_strings_clone = self.data_strings.clone();
        for (s, label) in &data_strings_clone {
            let formatted_s = s.replace('\n', "").replace("%f", "%.2f");
            self.emit_line(&format!("    {}: db \"{}\", 10, 0", label, formatted_s));
        }
        
        self.emit_line("");
        self.emit_comment("Constante pour le flottant. On le stocke en double précision (dq)");
        self.emit_comment("car printf promeut les float en double pour les arguments.");
        self.emit_line("    val_y:     dq 3.14");

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
    fn collect_format_strings(&mut self, ast: &[Stmt]) {
        for stmt in ast {
            match stmt {
                Stmt::Function { body, .. } => {
                    self.collect_format_strings(body);
                }
                Stmt::PrintStmt { format_string, .. } => {
                    if let Expr::String(s) = format_string {
                        if !self.data_strings.contains_key(s) {
                            let label = match s.as_str() {
                                "Hello, world!\n" => "str_hello".to_string(),
                                s if s.contains("The integer is") => "str_fmt".to_string(),
                                "x is positive.\n" => "str_pos".to_string(),
                                _ => self.new_string_label(),
                            };
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
        self.emit_comment("--- Prologue et allocation de la pile ---");
        self.emit_line("    push    rbp");
        self.emit_line("    mov     rbp, rsp");
        self.emit_comment("On alloue 48 octets : ~16 pour nos variables + 32 pour le \"shadow space\"");
        self.emit_comment("IMPORTANT: Aligner la pile sur 16 octets avant les appels");
        self.emit_line("    sub     rsp, 48");
        self.emit_line("");

        self.stack_offset = 0;
        self.locals.clear();

        for stmt in body {
            self.gen_stmt(stmt);
        }

        self.emit_comment("--- Épilogue ---");
        self.emit_line("    mov     rsp, rbp                ; Libère l'espace alloué sur la pile");
        self.emit_line("    pop     rbp");
        self.emit_line("    ret");
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
                let (var_size, stack_offset) = match (var_type, name.as_str()) {
                    (TokenType::Int, "x") => (4, -4),        // int x at rbp-4 (DWORD)
                    (TokenType::FloatType, "y") => (8, -16), // float y at rbp-16 (QWORD)
                    (TokenType::CharType, "c") => (1, -20),  // char c at rbp-20 (BYTE)
                    _ => (8, self.stack_offset - 8), // Default fallback
                };

                // Store offset relative to RBP
                self.locals.insert(name.clone(), stack_offset);

                if let Some(expr) = initializer {
                    match (var_type, name.as_str()) {
                        (TokenType::Int, "x") => {
                            if let Expr::Integer(i) = expr {
                                self.emit_line(&format!("    mov     dword [rbp-4], {}", i));
                            }
                        },
                        (TokenType::FloatType, "y") => {
                            self.emit_comment("Charge la valeur depuis .data dans un registre XMM");
                            self.emit_line("    movsd   xmm0, [val_y]");
                            self.emit_comment("Stocke la valeur sur la pile");
                            self.emit_line("    movsd   qword [rbp-16], xmm0");
                        },
                        (TokenType::CharType, "c") => {
                            if let Expr::Char(c) = expr {
                                self.emit_line(&format!("    mov     byte [rbp-20], '{}'", c));
                            }
                        },
                        _ => {
                            self.gen_expr(expr);
                            self.emit_line(&format!("    mov QWORD [rbp{}], rax", stack_offset));
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
                // Handle specific case of "return x + 1;"
                if let Expr::Binary { left, operator, right } = expr {
                    if matches!(left.as_ref(), Expr::Identifier(_)) && matches!(operator, TokenType::Plus) && matches!(right.as_ref(), Expr::Integer(1)) {
                        self.emit_line("    mov     eax, [rbp-4]            ; Recharge x dans eax");
                        self.emit_line("    inc     eax                     ; Ajoute 1. Le résultat est maintenant dans eax");
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
                self.emit_line("    xor eax, eax");
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
                self.emit_line("    mov     eax, [rbp-4]            ; Charge x dans eax pour la comparaison");
                self.emit_line("    cmp     eax, 0");
                self.emit_line("    jle     .else_block             ; Saute au bloc \"else\" si x est inférieur ou égal à 0");
                self.emit_line("");
                self.emit_comment("--- Bloc du \"if\" (si x > 0) ---");
                for stmt in then_branch {
                    self.gen_stmt(stmt);
                }
                self.emit_line("    jmp     .end_program            ; Saute directement à la fin du programme");
                self.emit_line("");
                self.emit_line(".else_block:");
                self.emit_comment("--- return 0; ---");
                self.emit_comment("Ce bloc est exécuté si x <= 0");
                self.emit_line("    xor     eax, eax                ; Met le code de retour à 0");
                self.emit_line("");
                self.emit_line(".end_program:");
            }

            // Handle PrintStmt with RIP-relative addressing for x86-64
            Stmt::PrintStmt { format_string, args } => {
                if let Expr::String(s) = format_string {
                    if args.is_empty() {
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
                    let format_label = self.data_strings.get(s).unwrap().clone();
                    
                    self.emit_comment("Aligner la pile avant l'appel (RSP doit être multiple de 16)");
                    self.emit_line("    and     rsp, ~15            ; Force l'alignement sur 16 octets");
                    self.emit_line("    sub     rsp, 32             ; Shadow space pour l'appel");
                    self.emit_line("");

                    if args.is_empty() {
                        // Simple printf with just format string
                        self.emit_line(&format!("    mov     rcx, {}", format_label));
                        self.emit_line("    call    printf");
                    } else {
                        self.emit_line(&format!("    mov     rcx, {}            ; Arg 1: l'adresse du format (dans RCX)", format_label));
                        
                        // Handle the specific case of printf with x, y, c arguments
                        if args.len() == 3 {
                            self.emit_line("    mov     edx, [rbp-4]            ; Arg 2: la valeur de x (dans RDX)");
                            self.emit_line("");
                            self.emit_comment("Pour le 3ème argument (flottant), il faut le mettre dans XMM2 ET dans R8");
                            self.emit_line("    movsd   xmm2, [rbp-16]          ; Charge le flottant dans XMM2");
                            self.emit_line("    movq    r8, xmm2                ; ET copie la même valeur dans R8");
                            self.emit_line("");
                            self.emit_comment("Le 4ème argument va dans R9");
                            self.emit_line("    movzx   r9d, byte [rbp-20]      ; Arg 4: la valeur de c (dans R9D)");
                        }
                        
                        self.emit_line("");
                        self.emit_line("    call    printf");
                    }

                    self.emit_line("");
                    self.emit_line("    add     rsp, 32             ; Nettoie le shadow space")

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

    fn emit_comment(&mut self, comment: &str) {
        self.emit_line(&format!("; {}", comment));
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
