use super::base::{Target, TargetPlatform, CallingConvention};
use crate::codegen::core::instruction::Register;

/// macOS x64 target implementation
pub struct MacOSX64Target;

impl Target for MacOSX64Target {
    fn platform(&self) -> TargetPlatform {
        TargetPlatform::MacOSX64
    }
    
    fn calling_convention(&self) -> CallingConvention {
        CallingConvention::AppleX64
    }
    
    fn arch_name(&self) -> &'static str {
        "x86-64 macOS"
    }
    
    fn calling_convention_name(&self) -> &'static str {
        "Apple x64 ABI"
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
            "extern _printf".to_string(), // macOS prefixes with underscore
            "extern _exit".to_string(),
        ]
    }
    
    fn global_declarations(&self, symbols: &[&str]) -> Vec<String> {
        symbols.iter().map(|symbol| format!("global _{}", symbol)).collect() // macOS prefixes with underscore
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
        // macOS uses System V-like calling convention
        vec![Register::Rdi, Register::Rsi, Register::Rdx, Register::Rcx, Register::R8, Register::R9]
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
        vec![format!("call     _{}", function_name)] // macOS prefixes with underscore
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
        vec![] // macOS doesn't need special startup code for our use case
    }
}