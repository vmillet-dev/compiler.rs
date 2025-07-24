use super::{TargetArchitecture, RegisterAllocator, CallingConvention, MemoryLocation};
use crate::codegen::instruction::{Register, Operand, Size};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub enum X86Instruction {
    Mov { dest: Operand, src: Operand, size: Size },
    Add { dest: Operand, src: Operand, size: Size },
    Sub { dest: Operand, src: Operand, size: Size },
    Mul { operand: Operand, size: Size },
    Div { operand: Operand, size: Size },
    Cmp { left: Operand, right: Operand, size: Size },
    Je { label: String },
    Jne { label: String },
    Jl { label: String },
    Jle { label: String },
    Jg { label: String },
    Jge { label: String },
    Jmp { label: String },
    Call { target: String },
    Ret,
    Push { operand: Operand, size: Size },
    Pop { operand: Operand, size: Size },
    Label { name: String },
    Comment { text: String },
}

pub struct X86_64Windows {
    output: String,
    register_allocator: X86RegisterAllocator,
    calling_convention: WindowsX64CallingConvention,
}

impl X86_64Windows {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            register_allocator: X86RegisterAllocator::new(),
            calling_convention: WindowsX64CallingConvention::new(),
        }
    }
    
    fn format_instruction(&self, instr: &X86Instruction) -> String {
        match instr {
            X86Instruction::Mov { dest, src, size } => {
                format!("    mov {}, {}", 
                       self.format_operand(dest, size), 
                       self.format_operand(src, size))
            }
            X86Instruction::Add { dest, src, size } => {
                format!("    add {}, {}", 
                       self.format_operand(dest, size), 
                       self.format_operand(src, size))
            }
            X86Instruction::Sub { dest, src, size } => {
                format!("    sub {}, {}", 
                       self.format_operand(dest, size), 
                       self.format_operand(src, size))
            }
            X86Instruction::Mul { operand, size } => {
                format!("    imul {}", self.format_operand(operand, size))
            }
            X86Instruction::Div { operand, size } => {
                format!("    idiv {}", self.format_operand(operand, size))
            }
            X86Instruction::Cmp { left, right, size } => {
                format!("    cmp {}, {}", 
                       self.format_operand(left, size), 
                       self.format_operand(right, size))
            }
            X86Instruction::Je { label } => format!("    je {}", label),
            X86Instruction::Jne { label } => format!("    jne {}", label),
            X86Instruction::Jl { label } => format!("    jl {}", label),
            X86Instruction::Jle { label } => format!("    jle {}", label),
            X86Instruction::Jg { label } => format!("    jg {}", label),
            X86Instruction::Jge { label } => format!("    jge {}", label),
            X86Instruction::Jmp { label } => format!("    jmp {}", label),
            X86Instruction::Call { target } => format!("    call {}", target),
            X86Instruction::Ret => "    ret".to_string(),
            X86Instruction::Push { operand, size } => {
                format!("    push {}", self.format_operand(operand, size))
            }
            X86Instruction::Pop { operand, size } => {
                format!("    pop {}", self.format_operand(operand, size))
            }
            X86Instruction::Label { name } => format!("{}:", name),
            X86Instruction::Comment { text } => format!("    ; {}", text),
        }
    }
    
    fn format_operand(&self, operand: &Operand, size: &Size) -> String {
        match operand {
            Operand::Register(reg) => self.format_register(reg, size),
            Operand::Immediate(value) => value.to_string(),
            Operand::Memory { base, offset } => {
                if *offset == 0 {
                    format!("[{}]", self.format_register(base, size))
                } else if *offset > 0 {
                    format!("[{}+{}]", self.format_register(base, size), offset)
                } else {
                    format!("[{}{}]", self.format_register(base, size), offset)
                }
            }
            Operand::Label(label) => label.clone(),
            Operand::String(s) => format!("\"{}\"", s),
        }
    }
    
    fn format_register(&self, register: &Register, size: &Size) -> String {
        match (register, size) {
            (Register::Rax, Size::Qword) => "rax".to_string(),
            (Register::Rax, Size::Dword) => "eax".to_string(),
            (Register::Rbp, Size::Qword) => "rbp".to_string(),
            (Register::Rsp, Size::Qword) => "rsp".to_string(),
            (Register::Rcx, Size::Qword) => "rcx".to_string(),
            (Register::Rcx, Size::Dword) => "ecx".to_string(),
            (Register::Rdx, Size::Qword) => "rdx".to_string(),
            (Register::Rdx, Size::Dword) => "edx".to_string(),
            (Register::R8, Size::Qword) => "r8".to_string(),
            (Register::R8, Size::Dword) => "r8d".to_string(),
            (Register::R9, Size::Qword) => "r9".to_string(),
            (Register::R9, Size::Dword) => "r9d".to_string(),
            _ => format!("{:?}", register).to_lowercase(),
        }
    }
}

impl TargetArchitecture for X86_64Windows {
    type Register = Register;
    type Instruction = X86Instruction;
    type CallingConvention = WindowsX64CallingConvention;
    
    fn emit_instruction(&mut self, instr: Self::Instruction) {
        let formatted = self.format_instruction(&instr);
        self.output.push_str(&formatted);
        self.output.push('\n');
    }
    
    fn allocate_register(&mut self) -> Option<Self::Register> {
        self.register_allocator.allocate()
    }
    
    fn free_register(&mut self, reg: Self::Register) {
        self.register_allocator.free(reg);
    }
    
    fn calling_convention(&self) -> &Self::CallingConvention {
        &self.calling_convention
    }
    
    fn emit_prologue(&mut self, function_name: &str, local_size: usize) {
        self.emit_instruction(X86Instruction::Label { name: function_name.to_string() });
        self.emit_instruction(X86Instruction::Push { 
            operand: Operand::Register(Register::Rbp), 
            size: Size::Qword 
        });
        self.emit_instruction(X86Instruction::Mov { 
            dest: Operand::Register(Register::Rbp), 
            src: Operand::Register(Register::Rsp), 
            size: Size::Qword 
        });
        
        if local_size > 0 {
            self.emit_instruction(X86Instruction::Sub { 
                dest: Operand::Register(Register::Rsp), 
                src: Operand::Immediate(local_size as i64), 
                size: Size::Qword 
            });
        }
    }
    
    fn emit_epilogue(&mut self) {
        self.emit_instruction(X86Instruction::Mov { 
            dest: Operand::Register(Register::Rsp), 
            src: Operand::Register(Register::Rbp), 
            size: Size::Qword 
        });
        self.emit_instruction(X86Instruction::Pop { 
            operand: Operand::Register(Register::Rbp), 
            size: Size::Qword 
        });
        self.emit_instruction(X86Instruction::Ret);
    }
    
    fn get_output(&self) -> String {
        self.output.clone()
    }
    
    fn parameter_register(&self, index: usize) -> Option<Self::Register> {
        let param_regs = self.calling_convention.parameter_registers();
        param_regs.get(index).copied()
    }
    
    fn return_register(&self) -> Self::Register {
        self.calling_convention.return_register()
    }
    
    fn stack_pointer(&self) -> Self::Register {
        Register::Rsp
    }
    
    fn base_pointer(&self) -> Self::Register {
        Register::Rbp
    }
}

impl Default for X86_64Windows {
    fn default() -> Self {
        Self::new()
    }
}

pub struct X86RegisterAllocator {
    available_registers: HashSet<Register>,
    allocated_registers: HashSet<Register>,
}

impl X86RegisterAllocator {
    pub fn new() -> Self {
        let mut available = HashSet::new();
        available.insert(Register::Rax);
        available.insert(Register::Rcx);
        available.insert(Register::Rdx);
        available.insert(Register::R8);
        available.insert(Register::R9);
        
        Self {
            available_registers: available,
            allocated_registers: HashSet::new(),
        }
    }
}

impl RegisterAllocator<Register> for X86RegisterAllocator {
    fn allocate(&mut self) -> Option<Register> {
        if let Some(&reg) = self.available_registers.iter().next() {
            self.available_registers.remove(&reg);
            self.allocated_registers.insert(reg);
            Some(reg)
        } else {
            None
        }
    }
    
    fn free(&mut self, reg: Register) {
        if self.allocated_registers.remove(&reg) {
            self.available_registers.insert(reg);
        }
    }
    
    fn is_available(&self, reg: &Register) -> bool {
        self.available_registers.contains(reg)
    }
    
    fn available_registers(&self) -> Vec<Register> {
        self.available_registers.iter().copied().collect()
    }
    
    fn spill(&mut self, reg: Register) -> MemoryLocation {
        self.free(reg);
        MemoryLocation {
            offset: -8, // Simple stack offset
            base: Register::Rbp,
        }
    }
}

impl Default for X86RegisterAllocator {
    fn default() -> Self {
        Self::new()
    }
}

pub struct WindowsX64CallingConvention {
    parameter_registers: Vec<Register>,
    caller_saved: Vec<Register>,
    callee_saved: Vec<Register>,
}

impl WindowsX64CallingConvention {
    pub fn new() -> Self {
        Self {
            parameter_registers: vec![Register::Rcx, Register::Rdx, Register::R8, Register::R9],
            caller_saved: vec![Register::Rax, Register::Rcx, Register::Rdx, Register::R8, Register::R9],
            callee_saved: vec![Register::Rbp, Register::Rsp],
        }
    }
}

impl CallingConvention for WindowsX64CallingConvention {
    type Register = Register;
    
    fn parameter_registers(&self) -> &[Self::Register] {
        &self.parameter_registers
    }
    
    fn return_register(&self) -> Self::Register {
        Register::Rax
    }
    
    fn caller_saved_registers(&self) -> &[Self::Register] {
        &self.caller_saved
    }
    
    fn callee_saved_registers(&self) -> &[Self::Register] {
        &self.callee_saved
    }
    
    fn stack_alignment(&self) -> usize {
        16 // x86-64 requires 16-byte stack alignment
    }
}

impl Default for WindowsX64CallingConvention {
    fn default() -> Self {
        Self::new()
    }
}
