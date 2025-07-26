use std::collections::HashMap;
use crate::ir::IrProgram;
use super::emitter::Emitter;
use super::target::{Target, TargetPlatform, create_target};

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
    pub target: Box<dyn Target>,
}

impl IrCodegen {
    pub fn new() -> Self {
        Self::new_with_target(TargetPlatform::WindowsX64)
    }
    
    pub fn new_with_target(target_platform: TargetPlatform) -> Self {
        Self {
            output: String::new(),
            stack_offset: 0,
            locals: HashMap::new(),
            temp_locations: HashMap::new(),
            data_strings: HashMap::new(),
            label_count: 0,
            target: create_target(target_platform),
        }
    }

    /// Generate assembly from IR program
    pub fn generate(mut self, ir_program: &IrProgram) -> String {
        // Assembly file header
        self.emit_section_header("MINI-C COMPILER GENERATED ASSEMBLY (FROM IR)");
        self.emit_comment(&format!("Target: {}", self.target.arch_name()));
        self.emit_comment(&format!("Calling Convention: {}", self.target.calling_convention_name()));
        self.emit_comment("Generated from: Intermediate Representation");
        self.emit_line("");
        
        // Assembly directives
        self.emit_comment("Assembly configuration");
        for directive in self.target.assembly_directives() {
            self.emit_line(&directive);
        }
        
        // Global and external declarations
        for global in self.target.global_declarations(&["main"]) {
            self.emit_line(&global);
        }
        for external in self.target.external_declarations() {
            self.emit_line(&external);
        }

        // Data section - process global strings
        self.emit_section_header("DATA SECTION - String Literals and Constants");
        self.emit_line(&self.target.data_section_header());

        if ir_program.global_strings.is_empty() {
            self.emit_comment("No string literals found");
        } else {
            for (label, content) in &ir_program.global_strings {
                self.emit_comment(&format!("String constant: \"{}\"", content.replace('\n', "\\n")));
                let formatted_literal = self.target.format_string_literal(label, content);
                self.emit_line(&formatted_literal);
                self.data_strings.insert(label.clone(), content.clone());
            }
        }

        // Text section
        self.emit_section_header("TEXT SECTION - Executable Code");
        self.emit_line(&self.target.text_section_header());
        
        // Add startup code if needed
        for startup_line in self.target.startup_code() {
            self.emit_line(&startup_line);
        }

        // Generate code for each function
        for function in &ir_program.functions {
            self.generate_function(function);
        }

        self.output
    }
}