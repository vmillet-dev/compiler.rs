//! Assembly instruction definitions and operand types
//! 
//! This module defines the core instruction set, registers, operands,
//! and sizes used in assembly code generation.

use std::fmt;

/// Assembly instruction types supported by the code generator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
    // Data movement
    Mov, Movsd, Movzx, Movq, Lea,
    
    // Stack operations
    Push, Pop,
    
    // Arithmetic operations
    Add, Sub, Imul, Idiv, Inc, Neg, Cqo, Cdq,
    
    // Floating point arithmetic
    Addsd, Subsd, Mulsd, Divsd,
    
    // Comparison and testing
    Cmp, Test,
    
    // Conditional set operations
    Sete, Setne, Setl, Setle, Setg, Setge,
    
    // Control flow
    Jmp, Je, Jle, Call, Ret,
    
    // Logical operations
    And, Or, Xor,
}

/// CPU registers available for code generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Register {
    // 64-bit general purpose registers
    Rax, Rbp, Rsp, Rcx, Rdx, R8, R9,
    
    // 32-bit general purpose registers
    Eax, Edx, R8d, R9d,
    
    // 8-bit registers
    Al,
    
    // SSE registers for floating point
    Xmm0, Xmm1, Xmm2, Xmm3,
}

/// Operand types for assembly instructions
#[derive(Debug, Clone)]
pub enum Operand {
    /// Direct register reference
    Register(Register),
    
    /// Immediate constant value
    Immediate(i64),
    
    /// Memory location with base register and offset
    Memory { base: Register, offset: i32 },
    
    /// Label reference for jumps and calls
    Label(String),
    
    /// String literal (for data section)
    String(String),
}

/// Data size specifications for memory operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Size {
    Byte,   // 8-bit
    Word,   // 16-bit
    Dword,  // 32-bit
    Qword,  // 64-bit
}

impl Instruction {
    /// Get the string representation of the instruction
    pub fn as_str(&self) -> &'static str {
        match self {
            Instruction::Mov => "mov",
            Instruction::Movsd => "movsd",
            Instruction::Movzx => "movzx",
            Instruction::Movq => "movq",
            Instruction::Lea => "lea",
            Instruction::Push => "push",
            Instruction::Pop => "pop",
            Instruction::Add => "add",
            Instruction::Sub => "sub",
            Instruction::Imul => "imul",
            Instruction::Idiv => "idiv",
            Instruction::Inc => "inc",
            Instruction::Neg => "neg",
            Instruction::Cqo => "cqo",
            Instruction::Cdq => "cdq",
            Instruction::Addsd => "addsd",
            Instruction::Subsd => "subsd",
            Instruction::Mulsd => "mulsd",
            Instruction::Divsd => "divsd",
            Instruction::Cmp => "cmp",
            Instruction::Test => "test",
            Instruction::Sete => "sete",
            Instruction::Setne => "setne",
            Instruction::Setl => "setl",
            Instruction::Setle => "setle",
            Instruction::Setg => "setg",
            Instruction::Setge => "setge",
            Instruction::Jmp => "jmp",
            Instruction::Je => "je",
            Instruction::Jle => "jle",
            Instruction::Call => "call",
            Instruction::Ret => "ret",
            Instruction::And => "and",
            Instruction::Or => "or",
            Instruction::Xor => "xor",
        }
    }
    
    /// Check if this instruction modifies the stack pointer
    pub fn modifies_stack(&self) -> bool {
        matches!(self, Instruction::Push | Instruction::Pop | Instruction::Call | Instruction::Ret)
    }
    
    /// Check if this instruction is a control flow instruction
    pub fn is_control_flow(&self) -> bool {
        matches!(self, 
            Instruction::Jmp | Instruction::Je | Instruction::Jle | 
            Instruction::Call | Instruction::Ret
        )
    }
}

impl Register {
    /// Get the string representation of the register
    pub fn as_str(&self) -> &'static str {
        match self {
            Register::Rax => "rax",
            Register::Rbp => "rbp",
            Register::Rsp => "rsp",
            Register::Rcx => "rcx",
            Register::Rdx => "rdx",
            Register::R8 => "r8",
            Register::R9 => "r9",
            Register::Eax => "eax",
            Register::Edx => "edx",
            Register::R8d => "r8d",
            Register::R9d => "r9d",
            Register::Al => "al",
            Register::Xmm0 => "xmm0",
            Register::Xmm1 => "xmm1",
            Register::Xmm2 => "xmm2",
            Register::Xmm3 => "xmm3",
        }
    }
    
    /// Get the size of the register in bytes
    pub fn size_bytes(&self) -> usize {
        match self {
            Register::Al => 1,
            Register::Eax | Register::Edx | Register::R8d | Register::R9d => 4,
            Register::Rax | Register::Rbp | Register::Rsp | Register::Rcx | 
            Register::Rdx | Register::R8 | Register::R9 => 8,
            Register::Xmm0 | Register::Xmm1 | Register::Xmm2 | Register::Xmm3 => 16,
        }
    }
    
    /// Check if this is a floating point register
    pub fn is_float_register(&self) -> bool {
        matches!(self, Register::Xmm0 | Register::Xmm1 | Register::Xmm2 | Register::Xmm3)
    }
}

impl Size {
    /// Get the size in bytes
    pub fn bytes(&self) -> usize {
        match self {
            Size::Byte => 1,
            Size::Word => 2,
            Size::Dword => 4,
            Size::Qword => 8,
        }
    }
    
    /// Get the NASM size specifier
    pub fn nasm_specifier(&self) -> &'static str {
        match self {
            Size::Byte => "byte",
            Size::Word => "word",
            Size::Dword => "dword",
            Size::Qword => "qword",
        }
    }
}

// Display implementations for formatting
impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Register(reg) => write!(f, "{}", reg),
            Operand::Immediate(val) => write!(f, "{}", val),
            Operand::Memory { base, offset } => {
                if *offset == 0 {
                    write!(f, "[{}]", base)
                } else if *offset > 0 {
                    write!(f, "[{}+{}]", base, offset)
                } else {
                    write!(f, "[{}{}]", base, offset)
                }
            },
            Operand::Label(label) => write!(f, "{}", label),
            Operand::String(s) => write!(f, "{}", s),
        }
    }
}

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.nasm_specifier())
    }
}