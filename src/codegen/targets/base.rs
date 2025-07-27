use crate::codegen::Register;

/// Represents different target platforms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetPlatform {
    WindowsX64,
    LinuxX64,
    MacOSX64,
    MacOSArm64,
}

/// Represents different calling conventions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallingConvention {
    MicrosoftX64,
    SystemV,
    AppleX64,
    AppleArm64,
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