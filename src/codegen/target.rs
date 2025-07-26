use crate::codegen::instruction::Register;

/// Represents different target platforms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetPlatform {
    WindowsX64,
    LinuxX64,
    MacOSX64,
}

/// Represents different calling conventions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallingConvention {
    MicrosoftX64,
    SystemV,
    AppleX64,
}

/// Target-specific configuration and behavior
pub trait Target {
    /// Get the target platform
    fn platform(&self) -> TargetPlatform;
    
    /// Get the calling convention
    fn calling_convention(&self) -> CallingConvention;
    
    /// Get the target architecture name for comments
    fn arch_name(&self) -> &'static str;
    
    /// Get the calling convention name for comments
    fn calling_convention_name(&self) -> &'static str;
    
    /// Generate assembly file header directives
    fn assembly_directives(&self) -> Vec<String>;
    
    /// Generate data section header
    fn data_section_header(&self) -> String;
    
    /// Generate text section header
    fn text_section_header(&self) -> String;
    
    /// Generate external function declarations
    fn external_declarations(&self) -> Vec<String>;
    
    /// Generate global symbol declarations
    fn global_declarations(&self, symbols: &[&str]) -> Vec<String>;
    
    /// Generate function prologue instructions
    fn function_prologue(&self) -> Vec<String>;
    
    /// Generate function epilogue instructions
    fn function_epilogue(&self) -> Vec<String>;
    
    /// Get parameter passing registers in order
    fn parameter_registers(&self) -> Vec<Register>;
    
    /// Get return value register
    fn return_register(&self) -> Register;
    
    /// Get stack pointer register
    fn stack_pointer(&self) -> Register;
    
    /// Get base pointer register
    fn base_pointer(&self) -> Register;
    
    /// Get stack alignment requirement in bytes
    fn stack_alignment(&self) -> usize;
    
    /// Format a string literal for the target platform
    fn format_string_literal(&self, label: &str, content: &str) -> String;
    
    /// Format a function call instruction
    fn format_function_call(&self, function_name: &str) -> Vec<String>;
    
    /// Get the size and alignment for a data type
    fn type_info(&self, type_name: &str) -> (usize, usize); // (size, alignment)
    
    /// Generate platform-specific startup code if needed
    fn startup_code(&self) -> Vec<String>;
}

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

/// Linux x64 target implementation
pub struct LinuxX64Target;

impl Target for LinuxX64Target {
    fn platform(&self) -> TargetPlatform {
        TargetPlatform::LinuxX64
    }
    
    fn calling_convention(&self) -> CallingConvention {
        CallingConvention::SystemV
    }
    
    fn arch_name(&self) -> &'static str {
        "x86-64 Linux"
    }
    
    fn calling_convention_name(&self) -> &'static str {
        "System V ABI"
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
        // System V ABI uses different parameter registers
        vec![Register::Rax, Register::Rdx, Register::Rcx, Register::R8, Register::R9] // Note: RDI, RSI would be more accurate but not in our Register enum
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
        vec![
            "_start:".to_string(),
            "    call main".to_string(),
            "    mov rdi, rax".to_string(),
            "    mov rax, 60".to_string(),
            "    syscall".to_string(),
        ]
    }
}

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
        vec![Register::Rax, Register::Rdx, Register::Rcx, Register::R8, Register::R9] // Note: RDI, RSI would be more accurate
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

/// Factory function to create target instances
pub fn create_target(platform: TargetPlatform) -> Box<dyn Target> {
    match platform {
        TargetPlatform::WindowsX64 => Box::new(WindowsX64Target),
        TargetPlatform::LinuxX64 => Box::new(LinuxX64Target),
        TargetPlatform::MacOSX64 => Box::new(MacOSX64Target),
    }
}

/// Helper function to parse target platform from string
pub fn parse_target_platform(target_str: &str) -> Result<TargetPlatform, String> {
    match target_str.to_lowercase().as_str() {
        "windows" | "win" | "windows-x64" | "win64" => Ok(TargetPlatform::WindowsX64),
        "linux" | "linux-x64" | "linux64" => Ok(TargetPlatform::LinuxX64),
        "macos" | "darwin" | "macos-x64" | "darwin-x64" => Ok(TargetPlatform::MacOSX64),
        _ => Err(format!("Unknown target platform: {}", target_str)),
    }
}