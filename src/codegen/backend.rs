use crate::codegen::instruction::{Instruction, Operand, Register, Size};
use crate::lexer::TokenType;
use crate::ir::ir::{IrProgram, IrFunction, IrInstruction, IrValue, IrType};
use std::collections::HashMap;

pub struct IrBackend {
    output: String,
    stack_offset: i32,
    locals: HashMap<String, i32>,
    local_types: HashMap<String, TokenType>,
    _register_allocator: RegisterAllocator,
    ir_program: Option<IrProgram>,
}

impl IrBackend {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            stack_offset: 0,
            locals: HashMap::new(),
            local_types: HashMap::new(),
            _register_allocator: RegisterAllocator::new(),
            ir_program: None,
        }
    }
    
    pub fn set_ir_program(&mut self, program: IrProgram) {
        self.ir_program = Some(program);
    }
    
    pub fn generate_from_ir(&mut self) -> String {
        let mut program = String::new();
        
        program.push_str("section .data\n");
        program.push_str("    format_int db '%d', 0\n");
        program.push_str("    format_float db '%.2f', 0\n");
        program.push_str("    format_char db '%c', 0\n");
        program.push_str("    newline db 10, 0\n\n");
        
        if let Some(ir_program) = &self.ir_program {
            for (label, value) in &ir_program.global_strings {
                program.push_str(&format!("    {} db '{}', 0\n", label, value));
            }
        }
        
        program.push_str("\nsection .text\n");
        program.push_str("    global _start\n");
        program.push_str("    extern printf\n");
        program.push_str("    extern exit\n\n");
        
        if let Some(ir_program) = &self.ir_program {
            let functions = ir_program.functions.clone();
            for function in &functions {
                self.generate_function_from_ir(function);
            }
        }
        
        program.push_str(&self.output);
        
        program
    }
    
    /// Generate assembly for a single IR function
    fn generate_function_from_ir(&mut self, function: &IrFunction) {
        self.emit_label(&function.name);
        
        // Function prologue
        let prologue = BackendUtils::generate_prologue();
        for instr in prologue {
            self.output.push_str(&format!("    {}\n", instr));
        }
        
        for ir_instr in &function.instructions {
            self.generate_ir_instruction(ir_instr);
        }
        
        // Function epilogue
        let epilogue = BackendUtils::generate_epilogue();
        for instr in epilogue {
            self.output.push_str(&format!("    {}\n", instr));
        }
    }
    
    /// Generate assembly for a single IR instruction
    fn generate_ir_instruction(&mut self, ir_instr: &IrInstruction) {
        match ir_instr {
            IrInstruction::Alloca { name, var_type } => {
                let token_type = self.ir_type_to_token_type(var_type);
                let (size, new_offset) = BackendUtils::calculate_stack_offset(&token_type, self.stack_offset);
                self.stack_offset = new_offset;
                self.locals.insert(name.clone(), new_offset);
                self.local_types.insert(name.clone(), token_type);
                self.emit_comment(&format!("alloca {} ({})", name, size));
            }
            IrInstruction::Store { value, dest, .. } => {
                if let IrValue::Local(dest_name) = dest {
                    if let Some(&dest_offset) = self.locals.get(dest_name) {
                        match value {
                            IrValue::IntConstant(val) => {
                                self.emit_instruction_with_size(
                                    Instruction::Mov,
                                    Size::Dword,
                                    vec![
                                        Operand::Memory { base: Register::Rbp, offset: dest_offset },
                                        Operand::Immediate(*val)
                                    ]
                                );
                            }
                            IrValue::Local(var) => {
                                if let Some(&var_offset) = self.locals.get(var) {
                                    self.emit_instruction_with_size(
                                        Instruction::Mov,
                                        Size::Dword,
                                        vec![
                                            Operand::Register(Register::Eax),
                                            Operand::Memory { base: Register::Rbp, offset: var_offset }
                                        ]
                                    );
                                    self.emit_instruction_with_size(
                                        Instruction::Mov,
                                        Size::Dword,
                                        vec![
                                            Operand::Memory { base: Register::Rbp, offset: dest_offset },
                                            Operand::Register(Register::Eax)
                                        ]
                                    );
                                }
                            }
                            _ => {
                                self.emit_comment(&format!("store {:?} -> {:?}", value, dest));
                            }
                        }
                    }
                }
            }
            IrInstruction::Load { dest, src, .. } => {
                if let (IrValue::Local(dest_name), IrValue::Local(src_name)) = (dest, src) {
                    if let Some(src_offset) = self.locals.get(src_name) {
                        self.emit_instruction_with_size(
                            Instruction::Mov,
                            Size::Dword,
                            vec![
                                Operand::Register(Register::Eax),
                                Operand::Memory { base: Register::Rbp, offset: *src_offset }
                            ]
                        );
                        self.emit_comment(&format!("load {} from {}", dest_name, src_name));
                    }
                }
            }
            IrInstruction::Return { value, .. } => {
                if let Some(value) = value {
                    match value {
                        IrValue::IntConstant(val) => {
                            self.emit_instruction_with_size(
                                Instruction::Mov,
                                Size::Dword,
                                vec![Operand::Register(Register::Eax), Operand::Immediate(*val)]
                            );
                        }
                        IrValue::Local(var) => {
                            if let Some(offset) = self.locals.get(var) {
                                self.emit_instruction_with_size(
                                    Instruction::Mov,
                                    Size::Dword,
                                    vec![
                                        Operand::Register(Register::Eax),
                                        Operand::Memory { base: Register::Rbp, offset: *offset }
                                    ]
                                );
                            }
                        }
                        _ => {
                            self.emit_comment(&format!("return {:?}", value));
                        }
                    }
                }
                
                let epilogue = BackendUtils::generate_epilogue();
                for instr in epilogue {
                    self.output.push_str(&format!("    {}\n", instr));
                }
            }
            _ => {
                self.emit_comment(&format!("IR instruction: {:?}", ir_instr));
            }
        }
    }
    
    fn ir_type_to_token_type(&self, ir_type: &IrType) -> TokenType {
        match ir_type {
            IrType::Int => TokenType::Int,
            IrType::Float => TokenType::FloatType,
            IrType::Char => TokenType::CharType,
            IrType::Void => TokenType::Void,
            _ => TokenType::Int, // Default fallback
        }
    }

    pub fn emit_instruction(&mut self, instr: Instruction, operands: Vec<Operand>) {
        let formatted = BackendUtils::format_instruction(&instr, &operands);
        self.output.push_str(&format!("    {}\n", formatted));
    }
    
    pub fn emit_instruction_with_size(&mut self, instr: Instruction, size: Size, operands: Vec<Operand>) {
        let formatted = BackendUtils::format_instruction_with_size(&instr, &size, &operands);
        self.output.push_str(&format!("    {}\n", formatted));
    }
    
    pub fn emit_instruction_with_size_and_comment(&mut self, instr: Instruction, size: Size, operands: Vec<Operand>, comment: Option<&str>) {
        let formatted = BackendUtils::format_instruction_with_size(&instr, &size, &operands);
        if let Some(comment) = comment {
            self.output.push_str(&format!("    {} ; {}\n", formatted, comment));
        } else {
            self.output.push_str(&format!("    {}\n", formatted));
        }
    }
    
    pub fn emit_comment(&mut self, comment: &str) {
        self.output.push_str(&format!("    ; {}\n", comment));
    }
    
    pub fn emit_label(&mut self, label: &str) {
        self.output.push_str(&format!("{}:\n", label));
    }
    
    pub fn get_stack_offset(&self) -> i32 {
        self.stack_offset
    }
    
    pub fn set_stack_offset(&mut self, offset: i32) {
        self.stack_offset = offset;
    }
    
    pub fn get_locals(&self) -> &HashMap<String, i32> {
        &self.locals
    }
    
    pub fn get_locals_mut(&mut self) -> &mut HashMap<String, i32> {
        &mut self.locals
    }
    
    pub fn get_local_types(&self) -> &HashMap<String, TokenType> {
        &self.local_types
    }
    
    pub fn get_local_types_mut(&mut self) -> &mut HashMap<String, TokenType> {
        &mut self.local_types
    }
    
    pub fn get_output(&self) -> &str {
        &self.output
    }
}

impl Default for IrBackend {
    fn default() -> Self {
        Self::new()
    }
}

pub struct BackendUtils;

impl BackendUtils {
    pub fn calculate_stack_offset(var_type: &TokenType, current_offset: i32) -> (usize, i32) {
        match var_type {
            TokenType::Int => {
                let new_offset = current_offset - 4;
                (4, new_offset)
            },
            TokenType::FloatType => {
                let new_offset = current_offset - 8;
                (8, new_offset)
            },
            TokenType::CharType => {
                let new_offset = current_offset - 1;
                (1, new_offset)
            },
            _ => {
                let new_offset = current_offset - 8;
                (8, new_offset)
            }
        }
    }
    
    pub fn format_instruction(instr: &Instruction, operands: &[Operand]) -> String {
        let instr_str = format!("{:?}", instr).to_lowercase();
        if operands.is_empty() {
            instr_str
        } else {
            let operands_str = operands.iter()
                .map(|op| Self::format_operand(op))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{} {}", instr_str, operands_str)
        }
    }
    
    pub fn format_instruction_with_size(instr: &Instruction, size: &Size, operands: &[Operand]) -> String {
        let instr_str = format!("{:?}", instr).to_lowercase();
        let size_suffix = match size {
            Size::Byte => "b",
            Size::Word => "w", 
            Size::Dword => "d",
            Size::Qword => "q",
        };
        
        if operands.is_empty() {
            format!("{}{}", instr_str, size_suffix)
        } else {
            let operands_str = operands.iter()
                .map(|op| Self::format_operand(op))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}{} {}", instr_str, size_suffix, operands_str)
        }
    }
    
    pub fn format_operand(operand: &Operand) -> String {
        match operand {
            Operand::Register(reg) => format!("{:?}", reg).to_lowercase(),
            Operand::Immediate(val) => val.to_string(),
            Operand::Memory { base, offset } => {
                if *offset == 0 {
                    format!("[{}]", format!("{:?}", base).to_lowercase())
                } else if *offset > 0 {
                    format!("[{}+{}]", format!("{:?}", base).to_lowercase(), offset)
                } else {
                    format!("[{}{}]", format!("{:?}", base).to_lowercase(), offset)
                }
            },
            Operand::String(s) => s.clone(),
            Operand::Label(label) => label.clone(),
        }
    }
    
    pub fn generate_prologue() -> Vec<String> {
        vec![
            "push rbp".to_string(),
            "mov rbp, rsp".to_string(),
        ]
    }
    
    pub fn generate_epilogue() -> Vec<String> {
        vec![
            "mov rsp, rbp".to_string(),
            "pop rbp".to_string(),
            "ret".to_string(),
        ]
    }
}

pub struct RegisterAllocator {
    available_registers: Vec<Register>,
    allocated_registers: HashMap<String, Register>,
}

impl RegisterAllocator {
    pub fn new() -> Self {
        Self {
            available_registers: vec![
                Register::Rax, Register::Rcx, Register::Rdx, Register::R8, Register::R9,
            ],
            allocated_registers: HashMap::new(),
        }
    }
    
    pub fn allocate(&mut self, var_name: String) -> Option<Register> {
        if let Some(reg) = self.available_registers.pop() {
            self.allocated_registers.insert(var_name, reg);
            Some(reg)
        } else {
            None // Need to spill to memory
        }
    }
    
    pub fn free(&mut self, var_name: &str) -> Option<Register> {
        if let Some(reg) = self.allocated_registers.remove(var_name) {
            self.available_registers.push(reg);
            Some(reg)
        } else {
            None
        }
    }
    
    pub fn get_register(&self, var_name: &str) -> Option<Register> {
        self.allocated_registers.get(var_name).copied()
    }
    
    pub fn is_available(&self, reg: Register) -> bool {
        self.available_registers.contains(&reg)
    }
}

impl Default for RegisterAllocator {
    fn default() -> Self {
        Self::new()
    }
}
