use std::fmt;

#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    Mov, Movsd, Movzx, Movq, Lea,
    Push, Pop,
    Add, Sub, Imul, Idiv, Inc, Neg, Cqo, Cdq, Addsd, Subsd, Mulsd, Divsd,
    Cmp, Test,
    Sete, Setne, Setl, Setle, Setg, Setge,
    Jmp, Je, Jle, Call, Ret,
    And, Or, Xor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Register {
    Rax, Rbp, Rsp, Rcx, Rdx, R8, R9,
    Eax, Edx, R8d, R9d,
    Al,
    Xmm0, Xmm1, Xmm2, Xmm3,
}

#[derive(Debug, Clone)]
pub enum Operand {
    Register(Register),
    Immediate(i64),
    Memory { base: Register, offset: i32 },
    Label(String),
    String(String),
}

#[derive(Debug, Clone, Copy)]
pub enum Size {
    Byte, Word, Dword, Qword,
}

impl Instruction {
    pub fn to_string(&self) -> &'static str {
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
}

impl Register {
    pub fn to_string(&self) -> &'static str {
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
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Register(reg) => write!(f, "{}", reg),
            Operand::Immediate(val) => write!(f, "{}", val),
            Operand::Memory { base, offset } => {
                if *offset >= 0 {
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
        let size_str = match self {
            Size::Byte => "byte",
            Size::Word => "word",
            Size::Dword => "dword",
            Size::Qword => "qword",
        };
        write!(f, "{}", size_str)
    }
}
