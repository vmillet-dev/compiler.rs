use std::collections::HashMap;
use crate::lexer::TokenType;
use crate::parser::ast::Stmt;
use super::instruction::{Instruction, Operand, Register};
use super::emitter::{Emitter, CodeEmitter};
use super::statement::StatementGenerator;
use super::analyzer::AstAnalyzer;

pub struct Codegen {
    pub label_count: usize,
    pub stack_offset: i32,
    pub locals: HashMap<String, i32>,
    pub local_types: HashMap<String, TokenType>, // Track variable types
    pub output: String,
    pub data_strings: HashMap<String, String>, // To store format strings and their labels
    pub string_label_count: usize, // For unique string labels
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
}

impl Emitter for Codegen {
    fn emit_line(&mut self, line: &str) {
        self.output.push_str(line);
        self.output.push('\n');
    }

    fn emit_comment(&mut self, comment: &str) {
        self.emit_line(&format!("; {}", comment));
    }
}