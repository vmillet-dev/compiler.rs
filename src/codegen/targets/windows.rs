use super::base::{Target, TargetPlatform, CallingConvention};
use crate::codegen::Register;

/// Windows x64 target implementation
pub struct WindowsX64Target;

impl Target for WindowsX64Target {
    fn platform(&self) -> TargetPlatform {
        TargetPlatform::WindowsX64
    }
    
    fn calling_convention(&self) -> CallingConvention {
        CallingConvention::MicrosoftX64
    }
    
    fn arch_name(&self) -> &'static str {
        "x86-64 Windows"
    }
    
    fn calling_convention_name(&self) -> &'static str {
        "Microsoft x64"
    }
    
    fn assembly_directives(&self) -> Vec<String> {
        vec![
            "bits 64".to_string(),
            "default rel".to_string(),
        ]
    }
    
    fn data_section_header(&self) -> String {
        "section .data".to_string()
    }
    
    fn text_section_header(&self) -> String {
        "section .text".to_string()
    }
    
    fn external_declarations(&self) -> Vec<String> {
        vec![
            "extern printf".to_string(),
            "extern exit".to_string(),
        ]
    }
    
    fn global_declarations(&self, symbols: &[&str]) -> Vec<String> {
        symbols.iter().map(|symbol| format!("global {}", symbol)).collect()
    }
    
    fn function_prologue(&self) -> Vec<String> {
        vec![
            "push rbp".to_string(),
            "mov rbp, rsp".to_string(),
        ]
    }
    
    fn function_epilogue(&self) -> Vec<String> {
        vec![
            "mov rsp, rbp".to_string(),
            "pop rbp".to_string(),
            "ret".to_string(),
        ]
    }
    
    fn parameter_registers(&self) -> Vec<Register> {
        // Microsoft x64 calling convention
        vec![Register::Rcx, Register::Rdx, Register::R8, Register::R9]
    }
    
    fn return_register(&self) -> Register {
        Register::Rax
    }
    
    fn stack_pointer(&self) -> Register {
        Register::Rsp
    }
    
    fn base_pointer(&self) -> Register {
        Register::Rbp
    }
    
    fn stack_alignment(&self) -> usize {
        16
    }
    
    fn format_string_literal(&self, label: &str, content: &str) -> String {
        let formatted_content = content.replace('\n', "").replace("%f", "%.2f");
        format!("    {}: db \"{}\", 10, 0", label, formatted_content)
    }
    
    fn format_function_call(&self, function_name: &str) -> Vec<String> {
        vec![format!("call     {}", function_name)]
    }

    fn format_function_name(&self, function_name: &str) -> String {
        format!("{}:", function_name)
    }

    fn type_info(&self, type_name: &str) -> (usize, usize) {
        match type_name {
            "int" | "i32" => (4, 4),
            "float" | "f32" => (4, 4),
            "double" | "f64" => (8, 8),
            "char" | "i8" => (1, 1),
            "ptr" | "pointer" => (8, 8),
            _ => (8, 8), // Default to pointer size
        }
    }
    
    fn startup_code(&self) -> Vec<String> {
        vec![] // Windows doesn't need special startup code for our use case
    }
}