// Example demonstrating the new codegen architecture
// This file shows how to use the refactored codegen module

use compiler_minic::codegen::{
    // Core traits and types
    Emitter, CodeEmitter, Instruction, Operand, Register, Size,
    // Backend implementations
    IrBackend,
    // Utilities
    RegisterAllocator, StackManager, InstructionFormatter,
};

fn main() {
    // Example 1: Using the new IrBackend
    println!("=== New IrBackend Example ===");
    let mut backend = IrBackend::new();
    
    // The backend implements Emitter trait
    backend.emit_comment("This is a comment");
    backend.emit_line("mov rax, 42");
    
    // It also implements CodeEmitter via blanket impl
    backend.emit_instruction(
        Instruction::Mov,
        vec![
            Operand::Register(Register::Rax),
            Operand::Immediate(42)
        ]
    );
    
    println!("Generated assembly:\n{}", backend.get_output());
    
    // Example 2: Using utilities independently
    println!("\n=== Utilities Example ===");
    
    // Register allocator
    let mut reg_alloc = RegisterAllocator::new();
    if let Some(reg) = reg_alloc.allocate("temp_var".to_string()) {
        println!("Allocated register {:?} for temp_var", reg);
    }
    
    // Stack manager
    let mut stack_mgr = StackManager::new();
    let offset = stack_mgr.allocate_variable("local_var".to_string(), compiler_minic::lexer::TokenType::Int);
    println!("Allocated stack offset {} for local_var", offset);
    
    // Instruction formatter
    let formatted = InstructionFormatter::format_instruction_with_size(
        &Instruction::Mov,
        &Size::Dword,
        &[
            Operand::Register(Register::Eax),
            Operand::Immediate(123)
        ]
    );
    println!("Formatted instruction: {}", formatted);
    
    // Example 3: Additional IrBackend features
    println!("\n=== Additional IrBackend Features ===");
    let mut backend2 = IrBackend::new();
    backend2.emit_section_header("EXAMPLE SECTION");
    backend2.emit_subsection_header("Example Subsection");
    let label = backend2.generate_label("example");
    backend2.emit_label(&label);
    println!("Additional features output:\n{}", backend2.get_output());
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_new_backend_basic_functionality() {
        let mut backend = IrBackend::new();
        backend.emit_comment("Test comment");
        backend.emit_instruction(Instruction::Mov, vec![
            Operand::Register(Register::Rax),
            Operand::Immediate(42)
        ]);
        
        let output = backend.get_output();
        assert!(output.contains("; Test comment"));
        assert!(output.contains("mov rax, 42"));
    }
    
    #[test]
    fn test_utilities_integration() {
        let mut reg_alloc = RegisterAllocator::new();
        let mut stack_mgr = StackManager::new();
        
        // Test register allocation
        let reg = reg_alloc.allocate("var1".to_string());
        assert!(reg.is_some());
        
        // Test stack management
        let offset = stack_mgr.allocate_variable("var2".to_string(), compiler_minic::lexer::TokenType::Int);
        assert_eq!(offset, -4); // Int takes 4 bytes
        
        // Test instruction formatting
        let formatted = InstructionFormatter::format_instruction(
            &Instruction::Add,
            &[Operand::Register(Register::Rax), Operand::Immediate(10)]
        );
        assert_eq!(formatted, "add rax, 10");
    }
    
    #[test]
    fn test_additional_features() {
        // Test additional IrBackend features
        let mut backend = IrBackend::new();
        backend.emit_section_header("TEST SECTION");
        let label = backend.generate_label("test");
        backend.emit_label(&label);
        
        let output = backend.get_output();
        assert!(output.contains("TEST SECTION"));
        assert!(output.contains("test_0:"));
    }
}