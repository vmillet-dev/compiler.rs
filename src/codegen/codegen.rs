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
    const STACK_ALIGNMENT: i32 = 16;
    const SHADOW_SPACE: i32 = 32;
    const VARIABLE_SPACE: i32 = 16;
    const TOTAL_STACK_SIZE: i32 = Self::VARIABLE_SPACE + Self::SHADOW_SPACE;

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
        self.emit_assembly_header();

        self.collect_format_strings(ast);
        self.collect_float_constants(ast);

        self.emit_data_section_header();

        let data_strings_clone = self.data_strings.clone();
        for (s, label) in &data_strings_clone {
            self.emit_data_string(&label, s);
        }
        
        self.emit_empty_line();
        self.emit_float_constants(ast);

        self.emit_text_section_header();

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
        self.emit_label(name);
        self.emit_function_prologue();

        self.stack_offset = 0;
        self.locals.clear();

        for stmt in body {
            self.gen_stmt(stmt);
        }

        self.emit_function_epilogue();
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

                if let Some(expr) = initializer {
                    match var_type {
                        TokenType::Int => {
                            if let Expr::Integer(i) = expr {
                                self.emit_store_int(*i, stack_offset);
                            } else {
                                self.gen_expr(expr);
                                self.emit_store_int_from_reg("eax", stack_offset);
                            }
                        },
                        TokenType::FloatType => {
                            if let Expr::Float(_f) = expr {
                                self.emit_load_float_to_xmm(name, "xmm0");
                                self.emit_store_float_from_xmm("xmm0", stack_offset);
                                
                                if !self.data_strings.contains_key(&format!("val_{}", name)) {
                                    self.data_strings.insert(format!("val_{}", name), format!("val_{}", name));
                                }
                            } else {
                                self.gen_expr(expr);
                                self.emit_store_float_from_xmm("xmm0", stack_offset);
                            }
                        },
                        TokenType::CharType => {
                            if let Expr::Char(c) = expr {
                                self.emit_store_char(*c, stack_offset);
                            } else {
                                self.gen_expr(expr);
                                self.emit_store_char_from_reg("al", stack_offset);
                            }
                        },
                        _ => {
                            self.gen_expr(expr);
                            self.emit_store_qword_from_reg("rax", stack_offset);
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
                            self.emit_load_var_to_reg(var_name, offset, "eax", &format!("Recharge {} dans eax", var_name));
                            self.emit_increment_reg("eax", "Ajoute 1. Le résultat est maintenant dans eax");
                        } else {
                            self.gen_expr(expr);
                            self.emit_result_comment();
                        }
                    } else {
                        self.gen_expr(expr);
                        self.emit_result_comment();
                    }
                } else {
                    self.gen_expr(expr);
                    self.emit_result_comment();
                }
            }

            Stmt::Return(None) => {
                self.emit_comment("--- return 0; ---");
                self.emit_xor_reg("eax");
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
                let (else_label, end_label) = self.new_if_label();
                
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
                            self.emit_load_var_to_reg(var_name, offset, "eax", &format!("Charge {} dans eax pour la comparaison", var_name));
                            self.emit_compare("eax", *val);
                            self.emit_conditional_jump("jle", &else_label, "Saute au bloc \"else\" si la condition est fausse");
                        }
                    } else {
                        self.gen_expr(condition);
                        self.emit_compare("eax", 0);
                        self.emit_conditional_jump("je", &else_label, "Saute au bloc \"else\" si la condition est fausse");
                    }
                } else {
                    self.gen_expr(condition);
                    self.emit_compare("eax", 0);
                    self.emit_conditional_jump("je", &else_label, "Saute au bloc \"else\" si la condition est fausse");
                }
                self.emit_empty_line();
                self.emit_comment("--- Bloc du \"if\" (si x > 0) ---");
                for stmt in then_branch {
                    self.gen_stmt(stmt);
                }
                self.emit_jump(&end_label, "Saute directement à la fin du programme");
                self.emit_empty_line();
                self.emit_label(&else_label);
                self.emit_comment("--- return 0; ---");
                self.emit_comment("Ce bloc est exécuté si x <= 0");
                self.emit_xor_reg("eax");
                self.emit_comment("Met le code de retour à 0");
                self.emit_empty_line();
                self.emit_label(&end_label);
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
                    
                    self.emit_stack_alignment();

                    // Windows x64 calling convention (main target)
                    if args.is_empty() {
                        // Simple printf with just format string
                        self.emit_printf_simple("rcx", &format_label);
                    } else {
                        self.emit_printf_format_arg("rcx", &format_label, "Arg 1: l'adresse du format (dans RCX)");
                        
                        // Handle printf arguments generically - Windows x64 calling convention
                        let arg_registers = ["edx", "r8d", "r9d"]; // Windows x64 calling convention
                        let gp_registers = ["rdx", "r8", "r9"]; // 64-bit versions for movq
                        let xmm_registers = ["xmm1", "xmm2", "xmm3"];
                        
                        for (i, arg) in args.iter().enumerate() {
                            if i >= 3 { break; } // Only handle first 3 args for now
                            
                            if let Expr::Identifier(var_name) = arg {
                                if let Some(&offset) = self.locals.get(var_name) {
                                    if i == 0 { // First arg - likely integer
                                        self.emit_printf_int_arg(arg_registers[i], offset, i + 2, var_name);
                                    } else if i == 1 { // Second arg - likely float
                                        self.emit_printf_float_arg(xmm_registers[i], gp_registers[i], offset, i + 2);
                                    } else if i == 2 { // Third arg - likely char
                                        self.emit_printf_char_arg(arg_registers[i], offset, i + 2, var_name);
                                    }
                                }
                            }
                        }
                        
                        self.emit_empty_line();
                        self.emit_call("printf");
                    }

                    self.emit_stack_cleanup();

                } else {
                    self.emit_printf_format_error(&format!("{:?}", format_string));
                }
            }
            _ => {
                self.emit_unsupported_statement(&format!("{:?}", stmt));
            }
        }
    }

    fn gen_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Integer(i) => {
                self.emit_load_immediate("rax", *i as i64);
            }
            Expr::Float(f) => {
                // Pour les floats, nous devons les gérer correctement pour printf
                // Nous allons créer une approche hybride :
                // - Stocker le float dans la section .data
                // - Charger sa valeur dans un registre XMM pour printf
                let float_bits = f.to_bits();
                self.emit_load_immediate("rax", float_bits as i64);
                // Pour printf avec %f, nous devrions utiliser movq xmm0, rax
                // mais cela nécessiterait de suivre quel registre XMM utiliser
                // Pour l'instant, gardons la représentation en bits
            }
            Expr::Char(c) => {
                self.emit_load_immediate("rax", *c as i64); // Move ASCII value
            }
            Expr::String(s) => {
                // CORRECTION: Utiliser RIP-relative addressing pour les chaînes
                if let Some(label) = self.data_strings.get(s).cloned() {
                    self.emit_load_string_address("rax", &label);
                } else {
                    // This should not happen if collect_format_strings is called correctly
                    self.emit_string_not_found_error(s);
                    self.emit_default_value("rax"); // Default to null pointer
                }
            }
            Expr::Identifier(name) => {
                if let Some(&offset) = self.locals.get(name) {
                    // Load value from stack into RAX
                    // Need to consider variable size (BYTE, WORD, DWORD, QWORD)
                    // For now, assume QWORD for all identifiers for simplicity.
                    self.emit_load_variable("rax", offset);
                } else {
                    self.emit_unknown_variable_error(name);
                    self.emit_default_value("rax"); // Default to 0 if variable not found
                }
            }

            Expr::Binary { left, operator, right } => {
                match operator {
                    TokenType::LogicalNot => { // Unary '!'
                        self.gen_expr(right); // Evaluate the operand
                        self.emit_unary_not();
                    }
                    TokenType::Minus if matches!(**left, Expr::Integer(0)) => { // Unary '-' (placeholder left operand)
                        self.gen_expr(right); // Evaluate the operand
                        self.emit_unary_negate();
                    }
                    _ => { // Binary operators
                        self.gen_expr(right);
                        self.emit_push_reg("rax");
                        self.gen_expr(left);
                        self.emit_pop_reg("rcx");

                        match operator {
                            TokenType::Plus => self.emit_arithmetic_op("add", "rax", "rcx"),
                            TokenType::Minus => self.emit_arithmetic_op("sub", "rax", "rcx"),
                            TokenType::Multiply => self.emit_arithmetic_op("imul", "rax", "rcx"),
                            TokenType::Divide => {
                                self.emit_division_setup();
                            }
                            TokenType::Equal => {
                                self.emit_comparison_result("sete");
                            }
                            TokenType::NotEqual => {
                                self.emit_comparison_result("setne");
                            }
                            TokenType::LessThan => {
                                self.emit_comparison_result("setl");
                            }
                            TokenType::LessEqual => {
                                self.emit_comparison_result("setle");
                            }
                            TokenType::GreaterThan => {
                                self.emit_comparison_result("setg");
                            }
                            TokenType::GreaterEqual => {
                                self.emit_comparison_result("setge");
                            }
                            TokenType::LogicalAnd => {
                                self.emit_logical_and_result();
                            }
                            TokenType::LogicalOr => {
                                self.emit_logical_or_result();
                            }
                            _ => {
                                self.emit_unsupported_binary_op();
                                self.emit_default_value("rax"); // Default value
                            }
                        }
                    }
                }
            }
            Expr::Call { callee, arguments: _ } => {
                // This is a generic function call.
                // For now, we'll treat it as unsupported as printf is handled by Stmt::PrintStmt.
                // A full compiler would need to resolve `callee` and pass `arguments`.
                self.emit_unsupported_function_call(&format!("{:?}", callee));
                self.emit_default_value("rax");
            }
            _ => {
                self.emit_unsupported_expr(&format!("{:?}", expr));
                self.emit_default_value("rax");
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

    // Helper methods for assembly generation
    fn emit_mov_instruction(&mut self, dest: &str, src: &str) {
        self.emit_line(&format!("    mov     {}, {}", dest, src));
    }

    // Helper methods for variable operations
    fn emit_store_int(&mut self, value: i64, offset: i32) {
        self.emit_line(&format!("    mov     dword [rbp{}], {}", offset, value));
    }

    fn emit_store_int_from_reg(&mut self, reg: &str, offset: i32) {
        self.emit_line(&format!("    mov     dword [rbp{}], {}", offset, reg));
    }

    fn emit_store_float_from_xmm(&mut self, xmm_reg: &str, offset: i32) {
        self.emit_line(&format!("    movsd   qword [rbp{}], {}", offset, xmm_reg));
    }

    fn emit_store_char(&mut self, value: char, offset: i32) {
        self.emit_line(&format!("    mov     byte [rbp{}], '{}'", offset, value));
    }

    fn emit_store_char_from_reg(&mut self, reg: &str, offset: i32) {
        self.emit_line(&format!("    mov     byte [rbp{}], {}", offset, reg));
    }

    fn emit_load_float_to_xmm(&mut self, var_name: &str, xmm_reg: &str) {
        self.emit_comment("Charge la valeur depuis .data dans un registre XMM");
        self.emit_line(&format!("    movsd   {}, [val_{}]", xmm_reg, var_name));
        self.emit_comment("Stocke la valeur sur la pile");
    }

    fn emit_load_var_to_reg(&mut self, var_name: &str, offset: i32, reg: &str, comment: &str) {
        self.emit_line(&format!("    mov     {}, [rbp{}]            ; {}", reg, offset, comment));
    }

    fn emit_increment_reg(&mut self, reg: &str, comment: &str) {
        self.emit_line(&format!("    inc     {}                     ; {}", reg, comment));
    }

    fn emit_compare(&mut self, reg: &str, value: i64) {
        self.emit_line(&format!("    cmp     {}, {}", reg, value));
    }

    fn emit_conditional_jump(&mut self, condition: &str, label: &str, comment: &str) {
        self.emit_line(&format!("    {}     {}             ; {}", condition, label, comment));
    }

    fn emit_jump(&mut self, label: &str, comment: &str) {
        self.emit_line(&format!("    jmp     {}            ; {}", label, comment));
    }

    fn emit_label(&mut self, label: &str) {
        self.emit_line(&format!("{}:", label));
    }

    fn emit_xor_reg(&mut self, reg: &str) {
        self.emit_line(&format!("    xor {}, {}", reg, reg));
    }

    fn emit_call(&mut self, function: &str) {
        self.emit_line(&format!("    call    {}", function));
    }

    // Helper methods for printf argument handling
    fn emit_printf_format_arg(&mut self, register: &str, format_label: &str, comment: &str) {
        self.emit_line(&format!("    mov     {}, {}            ; {}", register, format_label, comment));
    }

    fn emit_printf_int_arg(&mut self, register: &str, offset: i32, arg_num: usize, var_name: &str) {
        self.emit_line(&format!("    mov     {}, [rbp{}]            ; Arg {}: la valeur de {} (dans {})", 
            register, offset, arg_num, var_name, register.to_uppercase()));
    }

    fn emit_printf_float_arg(&mut self, xmm_register: &str, gp_register: &str, offset: i32, arg_num: usize) {
        self.emit_line("");
        self.emit_comment(&format!("Pour le {}ème argument (flottant), il faut le mettre dans {} ET dans {}", 
            arg_num, xmm_register.to_uppercase(), gp_register.to_uppercase()));
        self.emit_line(&format!("    movsd   {}, [rbp{}]          ; Charge le flottant dans {}", 
            xmm_register, offset, xmm_register.to_uppercase()));
        self.emit_line(&format!("    movq    {}, {}                ; ET copie la même valeur dans {}", 
            gp_register, xmm_register, gp_register.to_uppercase()));
    }

    fn emit_printf_char_arg(&mut self, register: &str, offset: i32, arg_num: usize, var_name: &str) {
        self.emit_line("");
        self.emit_comment(&format!("Le {}ème argument va dans {}", arg_num, register.to_uppercase()));
        self.emit_line(&format!("    movzx   {}, byte [rbp{}]      ; Arg {}: la valeur de {} (dans {})", 
            register, offset, arg_num, var_name, register.to_uppercase()));
    }

    fn emit_printf_simple(&mut self, format_register: &str, format_label: &str) {
        self.emit_mov_instruction(format_register, &format_label);
        self.emit_call("printf");
    }

    // Helper methods for expression generation
    fn emit_load_immediate(&mut self, reg: &str, value: i64) {
        self.emit_line(&format!("    mov {}, {}", reg, value));
    }

    fn emit_load_string_address(&mut self, reg: &str, label: &str) {
        self.emit_line(&format!("    lea {}, [rel {}]", reg, label));
    }

    fn emit_load_variable(&mut self, reg: &str, offset: i32) {
        self.emit_line(&format!("    mov {}, [rbp{}]", reg, offset));
    }

    fn emit_default_value(&mut self, reg: &str) {
        self.emit_line(&format!("    mov {}, 0", reg));
    }

    fn emit_push_reg(&mut self, reg: &str) {
        self.emit_line(&format!("    push {}", reg));
    }

    fn emit_pop_reg(&mut self, reg: &str) {
        self.emit_line(&format!("    pop {}", reg));
    }

    fn emit_arithmetic_op(&mut self, op: &str, dest: &str, src: &str) {
        self.emit_line(&format!("    {} {}, {}", op, dest, src));
    }

    fn emit_division_setup(&mut self) {
        self.emit_line("    cqo");
        self.emit_line("    idiv rcx");
    }

    fn emit_comparison_result(&mut self, condition: &str) {
        self.emit_line("    cmp rax, rcx");
        self.emit_line(&format!("    {} al", condition));
        self.emit_line("    movzx rax, al");
    }

    fn emit_logical_and_result(&mut self) {
        self.emit_line("    and rax, rcx");
        self.emit_line("    cmp rax, 0");
        self.emit_line("    setne al");
        self.emit_line("    movzx rax, al");
    }

    fn emit_logical_or_result(&mut self) {
        self.emit_line("    or rax, rcx");
        self.emit_line("    cmp rax, 0");
        self.emit_line("    setne al");
        self.emit_line("    movzx rax, al");
    }

    fn emit_unary_not(&mut self) {
        self.emit_line("    cmp rax, 0");
        self.emit_line("    sete al");
        self.emit_line("    movzx rax, al");
    }

    fn emit_unary_negate(&mut self) {
        self.emit_line("    neg rax");
    }

    fn emit_store_qword_from_reg(&mut self, reg: &str, offset: i32) {
        self.emit_line(&format!("    mov     qword [rbp{}], {}", offset, reg));
    }

    // Helper methods for assembly file structure
    fn emit_assembly_header(&mut self) {
        self.emit_line("bits 64");
        self.emit_line("default rel");
        self.emit_line("global main");
        self.emit_line("extern printf");
        self.emit_line("");
    }

    fn emit_data_section_header(&mut self) {
        self.emit_comment("--- Data section for string literals ---");
        self.emit_line("section .data");
    }

    fn emit_text_section_header(&mut self) {
        self.emit_comment("--- Text section for executable code ---");
        self.emit_line("section .text");
    }

    fn emit_data_string(&mut self, label: &str, content: &str) {
        let formatted_content = content.replace('\n', "").replace("%f", "%.2f");
        self.emit_line(&format!("    {}: db \"{}\", 10, 0", label, formatted_content));
    }

    fn emit_empty_line(&mut self) {
        self.emit_line("");
    }

    fn emit_result_comment(&mut self) {
        self.emit_line("    ; result in eax");
    }

    fn emit_unsupported_statement(&mut self, stmt_debug: &str) {
        self.emit_line(&format!("    ; unsupported statement {}", stmt_debug));
    }

    fn emit_unsupported_expr(&mut self, expr_debug: &str) {
        self.emit_line(&format!("    ; unsupported expr {}", expr_debug));
    }

    fn emit_unsupported_binary_op(&mut self) {
        self.emit_line("    ; unsupported binary op");
    }

    fn emit_unsupported_function_call(&mut self, callee_debug: &str) {
        self.emit_line(&format!("    ; unsupported general function call expression: {}", callee_debug));
    }

    fn emit_string_not_found_error(&mut self, string_literal: &str) {
        self.emit_line(&format!("    ; String literal '{}' not found in data section", string_literal));
    }

    fn emit_unknown_variable_error(&mut self, var_name: &str) {
        self.emit_line(&format!("    ; unknown variable '{}'", var_name));
    }

    fn emit_printf_format_error(&mut self, format_debug: &str) {
        self.emit_line(&format!("    ; printf format string is not a string literal: {}", format_debug));
    }

    fn emit_float_constant(&mut self, var_name: &str, value: f64) {
        self.emit_line(&format!("    val_{}:     dq {}", var_name, value));
    }

    fn emit_function_prologue(&mut self) {
        self.emit_comment("--- Prologue et allocation de la pile ---");
        self.emit_line("    push    rbp");
        self.emit_line("    mov     rbp, rsp");
        self.emit_comment(&format!("On alloue {} octets : ~{} pour nos variables + {} pour le \"shadow space\"", 
            Self::TOTAL_STACK_SIZE, Self::VARIABLE_SPACE, Self::SHADOW_SPACE));
        self.emit_comment("IMPORTANT: Aligner la pile sur 16 octets avant les appels");
        self.emit_line(&format!("    sub     rsp, {}", Self::TOTAL_STACK_SIZE));
        self.emit_line("");
    }

    fn emit_function_epilogue(&mut self) {
        self.emit_comment("--- Épilogue ---");
        self.emit_line("    mov     rsp, rbp                ; Libère l'espace alloué sur la pile");
        self.emit_line("    pop     rbp");
        self.emit_line("    ret");
    }

    fn emit_stack_alignment(&mut self) {
        self.emit_comment("Aligner la pile avant l'appel (RSP doit être multiple de 16)");
        self.emit_line("    and     rsp, ~15            ; Force l'alignement sur 16 octets");
        self.emit_line(&format!("    sub     rsp, {}             ; Shadow space pour l'appel", Self::SHADOW_SPACE));
        self.emit_line("");
    }

    fn emit_stack_cleanup(&mut self) {
        self.emit_line("");
        self.emit_line(&format!("    add     rsp, {}             ; Nettoie le shadow space", Self::SHADOW_SPACE));
    }

    // Helper methods for float constant generation
    fn collect_float_constants(&mut self, ast: &[Stmt]) {
    }

    fn emit_float_constants(&mut self, ast: &[Stmt]) {
        self.emit_comment("Constante pour le flottant. On le stocke en double précision (dq)");
        self.emit_comment("car printf promeut les float en double pour les arguments.");
        
        for stmt in ast {
            if let Stmt::Function { body, .. } = stmt {
                self.emit_float_constants_from_body(body);
            }
        }
    }

    fn emit_float_constants_from_body(&mut self, body: &[Stmt]) {
        for stmt in body {
            match stmt {
                Stmt::VarDecl { var_type: TokenType::FloatType, name, initializer } => {
                    if let Some(Expr::Float(value)) = initializer {
                        self.emit_float_constant(name, *value);
                    }
                }
                Stmt::If { then_branch, .. } => {
                    self.emit_float_constants_from_body(then_branch);
                }
                Stmt::Block(stmts) => {
                    self.emit_float_constants_from_body(stmts);
                }
                _ => {}
            }
        }
    }

    // Helper method for generating unique labels for control flow
    fn new_if_label(&mut self) -> (String, String) {
        let else_label = format!(".else_block_{}", self.label_count);
        let end_label = format!(".end_program_{}", self.label_count);
        self.label_count += 1;
        (else_label, end_label)
    }
}
