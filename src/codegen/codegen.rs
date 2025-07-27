use std::collections::HashMap;
use crate::ir::{IrProgram, IrFunction, IrInstruction, IrValue, IrType};
use crate::codegen::core::{Emitter, Target, TargetPlatform, create_target};
use crate::codegen::utils::{RegisterAllocator, StackManager};

/// Modern IR backend with clean architecture
pub struct Codegen {
    pub output: String,
    pub stack_offset: i32,
    pub locals: HashMap<String, i32>,
    pub temp_locations: HashMap<usize, i32>, // Map temp variables to stack locations
    pub data_strings: HashMap<String, String>,
    pub label_count: usize,
    pub target: Box<dyn Target>,
    #[allow(dead_code)]
    stack_manager: StackManager,
    #[allow(dead_code)]
    register_allocator: RegisterAllocator,
}

impl Codegen {
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
            stack_manager: StackManager::new(),
            register_allocator: RegisterAllocator::new(),
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

    /// Calculate the stack space needed for a function
    pub fn calculate_stack_space(&mut self, function: &IrFunction) -> i32 {
        let mut space = 32; // Shadow space for Windows x64 ABI
        
        // Allocate space for local variables
        for (name, ir_type) in &function.local_vars {
            let size = self.get_type_size(ir_type);
            space += size;
            self.locals.insert(name.clone(), -space);
        }
        
        // Allocate space for temporary variables
        let mut _temp_count = 0;
        for instruction in &function.instructions {
            if let Some(temp_id) = self.extract_temp_id(instruction) {
                if !self.temp_locations.contains_key(&temp_id) {
                    _temp_count += 1;
                    space += 8; // Assume 8 bytes for all temps
                    self.temp_locations.insert(temp_id, -space);
                }
            }
        }
        
        // Align to 16 bytes
        (space + 15) & !15
    }

    /// Extract temporary variable ID from instruction if present
    pub fn extract_temp_id(&self, instruction: &IrInstruction) -> Option<usize> {
        match instruction {
            IrInstruction::BinaryOp { dest, .. } |
            IrInstruction::UnaryOp { dest, .. } |
            IrInstruction::Load { dest, .. } |
            IrInstruction::Move { dest, .. } => {
                if let IrValue::Temp(id) = dest {
                    Some(*id)
                } else {
                    None
                }
            }
            IrInstruction::Call { dest: Some(dest), .. } => {
                if let IrValue::Temp(id) = dest {
                    Some(*id)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Get the size in bytes for an IR type
    pub fn get_type_size(&self, ir_type: &IrType) -> i32 {
        match ir_type {
            IrType::Int => 4,
            IrType::Float => 8,
            IrType::Char => 1,
            IrType::String => 8, // Pointer size
            IrType::Void => 0,
            IrType::Pointer(_) => 8,
        }
    }
}

// Implement the emitter traits for IrBackend
impl Emitter for Codegen {
    fn emit_line(&mut self, line: &str) {
        self.output.push_str(line);
        self.output.push('\n');
    }

    fn emit_comment(&mut self, comment: &str) {
        self.emit_line(&format!("; {}", comment));
    }
}

// Helper methods for IrBackend
impl Codegen {
    /// Emit a section header with clear visual separation
    pub fn emit_section_header(&mut self, title: &str) {
        self.emit_line("");
        self.emit_line(&format!("; {}", "=".repeat(60)));
        self.emit_line(&format!("; {}", title));
        self.emit_line(&format!("; {}", "=".repeat(60)));
        self.emit_line("");
    }

    /// Emit a subsection header with lighter visual separation
    pub fn emit_subsection_header(&mut self, title: &str) {
        self.emit_line("");
        self.emit_line(&format!("; {}", "-".repeat(40)));
        self.emit_line(&format!("; {}", title));
        self.emit_line(&format!("; {}", "-".repeat(40)));
    }

    /// Generate a unique label
    pub fn generate_label(&mut self, prefix: &str) -> String {
        let label = format!("{}_{}", prefix, self.label_count);
        self.label_count += 1;
        label
    }

    /// Emit a label
    pub fn emit_label(&mut self, label: &str) {
        self.emit_line(&format!("{}:", label));
    }

    /// Emit stack layout summary for debugging
    pub fn emit_stack_layout_summary(&mut self) {
        self.emit_comment("Stack Layout Summary:");
        if self.locals.is_empty() && self.temp_locations.is_empty() {
            self.emit_comment("  No local variables or temporaries");
        } else {
            // Clone the data to avoid borrowing issues
            let locals = self.locals.clone();
            let temp_locations = self.temp_locations.clone();
            
            for (name, offset) in &locals {
                self.emit_comment(&format!("  Local '{}' at offset {}", name, offset));
            }
            for (temp_id, offset) in &temp_locations {
                self.emit_comment(&format!("  Temp %{} at offset {}", temp_id, offset));
            }
        }
    }

    /// Get the generated output
    pub fn get_output(&self) -> &str {
        &self.output
    }
}

// Include generator implementations
#[allow(unused_imports)]
use crate::codegen::generators::*;