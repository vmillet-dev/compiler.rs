pub mod x86_64_windows;

use crate::codegen::instruction::{Instruction, Register, Operand};
use crate::types::Type;
use std::collections::HashMap;

pub trait TargetArchitecture {
    type Register: Clone + PartialEq;
    type Instruction: Clone;
    type CallingConvention;
    
    fn emit_instruction(&mut self, instr: Self::Instruction);
    
    fn allocate_register(&mut self) -> Option<Self::Register>;
    
    fn free_register(&mut self, reg: Self::Register);
    
    fn calling_convention(&self) -> &Self::CallingConvention;
    
    fn emit_prologue(&mut self, function_name: &str, local_size: usize);
    
    fn emit_epilogue(&mut self);
    
    fn get_output(&self) -> String;
    
    fn parameter_register(&self, index: usize) -> Option<Self::Register>;
    
    fn return_register(&self) -> Self::Register;
    
    fn stack_pointer(&self) -> Self::Register;
    
    fn base_pointer(&self) -> Self::Register;
}

pub trait RegisterAllocator<R> {
    fn allocate(&mut self) -> Option<R>;
    
    fn free(&mut self, reg: R);
    
    fn is_available(&self, reg: &R) -> bool;
    
    fn available_registers(&self) -> Vec<R>;
    
    fn spill(&mut self, reg: R) -> MemoryLocation;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLocation {
    pub offset: i32,
    pub base: Register,
}

pub trait CallingConvention {
    type Register;
    
    fn parameter_registers(&self) -> &[Self::Register];
    
    fn return_register(&self) -> Self::Register;
    
    fn caller_saved_registers(&self) -> &[Self::Register];
    
    fn callee_saved_registers(&self) -> &[Self::Register];
    
    fn stack_alignment(&self) -> usize;
}

pub struct CodeGenerator<T: TargetArchitecture> {
    target: T,
    instructions: Vec<T::Instruction>,
    local_variables: HashMap<String, (Type, i32)>, // name -> (type, stack_offset)
    stack_offset: i32,
}

impl<T: TargetArchitecture> CodeGenerator<T> {
    pub fn new(target: T) -> Self {
        Self {
            target,
            instructions: Vec::new(),
            local_variables: HashMap::new(),
            stack_offset: 0,
        }
    }
    
    pub fn emit(&mut self, instruction: T::Instruction) 
    where 
        T::Instruction: Clone,
    {
        self.target.emit_instruction(instruction.clone());
        self.instructions.push(instruction);
    }
    
    pub fn allocate_local(&mut self, name: String, var_type: Type) -> i32 {
        self.stack_offset -= 8; // Assume 8-byte alignment for now
        self.local_variables.insert(name, (var_type, self.stack_offset));
        self.stack_offset
    }
    
    pub fn get_local_offset(&self, name: &str) -> Option<i32> {
        self.local_variables.get(name).map(|(_, offset)| *offset)
    }
    
    pub fn get_output(&self) -> String {
        self.target.get_output()
    }
    
    pub fn target(&self) -> &T {
        &self.target
    }
    
    pub fn target_mut(&mut self) -> &mut T {
        &mut self.target
    }
}
