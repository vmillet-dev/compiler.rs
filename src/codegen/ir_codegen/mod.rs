use std::collections::HashMap;
use crate::ir::IrProgram;
use super::emitter::Emitter;

mod function_generator;
mod stack_manager;
mod instruction_generator;
mod operation_generator;
mod call_generator;
mod value_converter;
mod emitter_impl;

// The modules are used internally via impl blocks, no need to re-export

/// IR-based code generator that produces assembly from IR
pub struct IrCodegen {
    pub output: String,
    pub stack_offset: i32,
    pub locals: HashMap<String, i32>,
    pub temp_locations: HashMap<usize, i32>, // Map temp variables to stack locations
    pub data_strings: HashMap<String, String>,
    pub label_count: usize,
}

impl IrCodegen {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            stack_offset: 0,
            locals: HashMap::new(),
            temp_locations: HashMap::new(),
            data_strings: HashMap::new(),
            label_count: 0,
        }
    }

    /// Generate assembly from IR program
    pub fn generate(mut self, ir_program: &IrProgram) -> String {
        // Assembly file header
        self.emit_section_header("MINI-C COMPILER GENERATED ASSEMBLY (FROM IR)");
        self.emit_comment("Target: x86-64 Windows");
        self.emit_comment("Calling Convention: Microsoft x64");
        self.emit_comment("Generated from: Intermediate Representation");
        self.emit_line("");
        
        // Assembly directives
        self.emit_comment("Assembly configuration");
        self.emit_line("bits 64");
        self.emit_line("default rel");
        self.emit_line("global main");
        self.emit_line("extern printf");

        // Data section - process global strings
        self.emit_section_header("DATA SECTION - String Literals and Constants");
        self.emit_line("section .data");

        if ir_program.global_strings.is_empty() {
            self.emit_comment("No string literals found");
        } else {
            for (label, content) in &ir_program.global_strings {
                let formatted_content = content.replace('\n', "").replace("%f", "%.2f");
                self.emit_comment(&format!("String constant: \"{}\"", content.replace('\n', "\\n")));
                self.emit_line(&format!("    {}: db \"{}\", 10, 0", label, formatted_content));
                self.data_strings.insert(label.clone(), content.clone());
            }
        }

        // Text section
        self.emit_section_header("TEXT SECTION - Executable Code");
        self.emit_line("section .text");

        // Generate code for each function
        for function in &ir_program.functions {
            self.generate_function(function);
        }

        self.output
    }
}