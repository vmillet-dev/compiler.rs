//! IR-based code generation backend
//! 
//! This module provides a clean, well-structured backend for generating
//! assembly code from intermediate representation (IR).


use crate::ir::{IrProgram, IrFunction, IrInstruction, IrValue, IrType};
use crate::lexer::TokenType;
use crate::codegen::core::{Emitter, CodeEmitter, CodeEmitterWithComment, Target, TargetPlatform, create_target};
use crate::codegen::core::{Instruction, Register, Operand, Size};
use crate::codegen::utils::{StackManager, RegisterAllocator};

/// IR-based code generation backend
pub struct IrBackend {
    /// Generated assembly output
    output: String,
    
    /// Stack management
    stack_manager: StackManager,
    
    /// Register allocation
    register_allocator: RegisterAllocator,
    
    /// Target platform configuration
    target: Box<dyn Target>,
    
    /// IR program being compiled
    ir_program: Option<IrProgram>,
}

impl IrBackend {
    /// Create a new IR backend with default target (Windows x64)
    pub fn new() -> Self {
        Self::new_with_target(TargetPlatform::WindowsX64)
    }
    
    /// Create a new IR backend with specified target platform
    pub fn new_with_target(target_platform: TargetPlatform) -> Self {
        Self {
            output: String::new(),
            stack_manager: StackManager::new(),
            register_allocator: RegisterAllocator::new(),
            target: create_target(target_platform),
            ir_program: None,
        }
    }
    
    /// Set the IR program to compile
    pub fn set_ir_program(&mut self, program: IrProgram) {
        self.ir_program = Some(program);
    }
    
    /// Generate assembly code from the IR program
    pub fn generate(&mut self) -> String {
        self.output.clear();
        
        if let Some(ir_program) = self.ir_program.clone() {
            self.generate_header();
            self.generate_data_section(&ir_program);
            self.generate_text_section(&ir_program);
        } else {
            self.emit_comment("Error: No IR program set");
        }
        
        self.output.clone()
    }
    
    /// Generate assembly file header
    fn generate_header(&mut self) {
        self.emit_section_header("MINI-C COMPILER GENERATED ASSEMBLY (FROM IR)");
        self.emit_comment(&format!("Target: {}", self.target.arch_name()));
        self.emit_comment(&format!("Calling Convention: {}", self.target.calling_convention_name()));
        self.emit_comment("Generated from: Intermediate Representation");
        self.emit_line("");
        
        // Assembly directives
        self.emit_comment("Assembly configuration");
        for directive in self.target.assembly_directives() {
            self.emit_line(&directive);
        }
        self.emit_line("");
        
        // Global and external declarations
        for global in self.target.global_declarations(&["main"]) {
            self.emit_line(&global);
        }
        for external in self.target.external_declarations() {
            self.emit_line(&external);
        }
        self.emit_line("");
    }
    
    /// Generate data section
    fn generate_data_section(&mut self, ir_program: &IrProgram) {
        self.emit_section_header("DATA SECTION - String Literals and Constants");
        self.emit_line(&self.target.data_section_header());
        self.emit_line("");
        
        if ir_program.global_strings.is_empty() {
            self.emit_comment("No string literals found");
        } else {
            for (label, content) in &ir_program.global_strings {
                self.emit_comment(&format!("String constant: \"{}\"", content.replace('\n', "\\n")));
                let formatted_literal = self.target.format_string_literal(label, content);
                self.emit_line(&formatted_literal);
            }
        }
        self.emit_line("");
    }
    
    /// Generate text section
    fn generate_text_section(&mut self, ir_program: &IrProgram) {
        self.emit_section_header("TEXT SECTION - Executable Code");
        self.emit_line(&self.target.text_section_header());
        self.emit_line("");
        
        // Add startup code if needed
        for startup_line in self.target.startup_code() {
            self.emit_line(&startup_line);
        }
        
        // Generate code for each function
        for function in &ir_program.functions {
            self.generate_function(function);
        }
    }
    
    /// Generate assembly for a single function
    fn generate_function(&mut self, function: &IrFunction) {
        self.emit_subsection_header(&format!("FUNCTION: {}", function.name));
        self.emit_line(&format!("{}:", function.name));
        
        // Reset state for new function
        self.stack_manager.reset();
        self.register_allocator.reset();
        
        // Function prologue
        self.emit_subsection_header("Function Prologue");
        let prologue_instructions = self.target.function_prologue();
        for (i, instr) in prologue_instructions.iter().enumerate() {
            let comment = match i {
                0 => Some("save caller's frame"),
                1 => Some("set up frame"),
                _ => None,
            };
            self.emit_line_with_comment(&format!("    {}", instr), comment);
        }
        self.emit_line("");
        
        // Generate instructions
        self.emit_subsection_header("Function Body");
        for ir_instr in &function.instructions {
            self.generate_ir_instruction(ir_instr);
        }
        
        self.emit_line("");
    }
    
    /// Generate assembly for a single IR instruction
    fn generate_ir_instruction(&mut self, ir_instr: &IrInstruction) {
        match ir_instr {
            IrInstruction::Alloca { name, var_type } => {
                self.generate_alloca(name, var_type);
            }
            IrInstruction::Store { value, dest, .. } => {
                self.generate_store(value, dest);
            }
            IrInstruction::Load { dest, src, .. } => {
                self.generate_load(dest, src);
            }
            IrInstruction::Return { value, .. } => {
                self.generate_return(value);
            }
            IrInstruction::BinaryOp { dest, op, left, right, .. } => {
                self.generate_binary_op(dest, op, left, right);
            }
            IrInstruction::Call { dest, func, args, .. } => {
                self.generate_call(dest, func, args);
            }
            _ => {
                self.emit_comment(&format!("TODO: Implement IR instruction: {:?}", ir_instr));
            }
        }
    }
    
    /// Generate alloca instruction
    fn generate_alloca(&mut self, name: &str, var_type: &IrType) {
        let token_type = self.ir_type_to_token_type(var_type);
        let offset = self.stack_manager.allocate_local(name.to_string(), token_type);
        self.emit_comment(&format!("alloca {} at offset {}", name, offset));
    }
    
    /// Generate store instruction
    fn generate_store(&mut self, value: &IrValue, dest: &IrValue) {
        if let IrValue::Local(dest_name) = dest {
            if let Some(dest_offset) = self.stack_manager.get_local_offset(dest_name) {
                match value {
                    IrValue::IntConstant(val) => {
                        self.emit_instruction_with_size_and_comment(
                            Instruction::Mov,
                            Size::Dword,
                            vec![
                                Operand::Memory { base: Register::Rbp, offset: dest_offset },
                                Operand::Immediate(*val)
                            ],
                            Some(&format!("store {} -> {}", val, dest_name))
                        );
                    }
                    IrValue::Local(src_name) => {
                        if let Some(src_offset) = self.stack_manager.get_local_offset(src_name) {
                            // Load from source to register, then store to destination
                            self.emit_instruction_with_size_and_comment(
                                Instruction::Mov,
                                Size::Dword,
                                vec![
                                    Operand::Register(Register::Eax),
                                    Operand::Memory { base: Register::Rbp, offset: src_offset }
                                ],
                                Some(&format!("load {}", src_name))
                            );
                            self.emit_instruction_with_size_and_comment(
                                Instruction::Mov,
                                Size::Dword,
                                vec![
                                    Operand::Memory { base: Register::Rbp, offset: dest_offset },
                                    Operand::Register(Register::Eax)
                                ],
                                Some(&format!("store to {}", dest_name))
                            );
                        }
                    }
                    _ => {
                        self.emit_comment(&format!("TODO: store {:?} -> {:?}", value, dest));
                    }
                }
            }
        }
    }
    
    /// Generate load instruction
    fn generate_load(&mut self, dest: &IrValue, src: &IrValue) {
        if let (IrValue::Local(dest_name), IrValue::Local(src_name)) = (dest, src) {
            if let Some(src_offset) = self.stack_manager.get_local_offset(src_name) {
                self.emit_instruction_with_size_and_comment(
                    Instruction::Mov,
                    Size::Dword,
                    vec![
                        Operand::Register(Register::Eax),
                        Operand::Memory { base: Register::Rbp, offset: src_offset }
                    ],
                    Some(&format!("load {} from {}", dest_name, src_name))
                );
            }
        }
    }
    
    /// Generate return instruction
    fn generate_return(&mut self, value: &Option<IrValue>) {
        if let Some(value) = value {
            match value {
                IrValue::IntConstant(val) => {
                    self.emit_instruction_with_size_and_comment(
                        Instruction::Mov,
                        Size::Dword,
                        vec![Operand::Register(Register::Eax), Operand::Immediate(*val)],
                        Some(&format!("return {}", val))
                    );
                }
                IrValue::Local(var_name) => {
                    if let Some(offset) = self.stack_manager.get_local_offset(var_name) {
                        self.emit_instruction_with_size_and_comment(
                            Instruction::Mov,
                            Size::Dword,
                            vec![
                                Operand::Register(Register::Eax),
                                Operand::Memory { base: Register::Rbp, offset: offset }
                            ],
                            Some(&format!("return {}", var_name))
                        );
                    }
                }
                _ => {
                    self.emit_comment(&format!("TODO: return {:?}", value));
                }
            }
        }
        
        // Generate epilogue
        let epilogue_instructions = self.target.function_epilogue();
        for (i, instr) in epilogue_instructions.iter().enumerate() {
            let comment = match i {
                0 => Some("restore stack"),
                1 => Some("restore frame"),
                2 => Some("return"),
                _ => None,
            };
            self.emit_line_with_comment(&format!("    {}", instr), comment);
        }
    }
    
    /// Generate binary operation
    fn generate_binary_op(&mut self, dest: &IrValue, op: &crate::ir::IrBinaryOp, left: &IrValue, right: &IrValue) {
        use crate::ir::IrBinaryOp;
        
        let (asm_op, op_name) = match op {
            IrBinaryOp::Add => (Instruction::Add, "add"),
            IrBinaryOp::Sub => (Instruction::Sub, "subtract"),
            IrBinaryOp::Mul => (Instruction::Imul, "multiply"),
            IrBinaryOp::Div => (Instruction::Idiv, "divide"),
            IrBinaryOp::Eq => (Instruction::Sete, "equal"),
            IrBinaryOp::Ne => (Instruction::Setne, "not equal"),
            IrBinaryOp::Lt => (Instruction::Setl, "less than"),
            IrBinaryOp::Le => (Instruction::Setle, "less or equal"),
            IrBinaryOp::Gt => (Instruction::Setg, "greater than"),
            IrBinaryOp::Ge => (Instruction::Setge, "greater or equal"),
            _ => {
                self.emit_comment(&format!("TODO: Implement binary operation: {:?}", op));
                return;
            }
        };
        
        self.emit_comment(&format!("Binary operation: {}", op_name));
        
        // Load left operand into EAX
        match left {
            IrValue::IntConstant(val) => {
                self.emit_instruction_with_size(
                    Instruction::Mov,
                    Size::Dword,
                    vec![Operand::Register(Register::Eax), Operand::Immediate(*val)]
                );
            }
            IrValue::Local(var_name) => {
                if let Some(offset) = self.stack_manager.get_local_offset(var_name) {
                    self.emit_instruction_with_size(
                        Instruction::Mov,
                        Size::Dword,
                        vec![
                            Operand::Register(Register::Eax),
                            Operand::Memory { base: Register::Rbp, offset: offset }
                        ]
                    );
                }
            }
            _ => {}
        }
        
        // Handle comparison operations differently
        if matches!(op, IrBinaryOp::Eq | IrBinaryOp::Ne | IrBinaryOp::Lt | IrBinaryOp::Le | IrBinaryOp::Gt | IrBinaryOp::Ge) {
            // Compare with right operand
            match right {
                IrValue::IntConstant(val) => {
                    self.emit_instruction_with_size(
                        Instruction::Cmp,
                        Size::Dword,
                        vec![Operand::Register(Register::Eax), Operand::Immediate(*val)]
                    );
                }
                IrValue::Local(var_name) => {
                    if let Some(offset) = self.stack_manager.get_local_offset(var_name) {
                        self.emit_instruction_with_size(
                            Instruction::Cmp,
                            Size::Dword,
                            vec![
                                Operand::Register(Register::Eax),
                                Operand::Memory { base: Register::Rbp, offset: offset }
                            ]
                        );
                    }
                }
                _ => {}
            }
            
            // Set result based on comparison
            self.emit_instruction_with_comment(
                asm_op,
                vec![Operand::Register(Register::Al)],
                Some(&format!("set result for {}", op_name))
            );
            
            // Zero-extend AL to EAX
            self.emit_instruction_with_size_and_comment(
                Instruction::Movzx,
                Size::Dword,
                vec![Operand::Register(Register::Eax), Operand::Register(Register::Al)],
                Some("zero-extend result to 32-bit")
            );
        } else {
            // Perform arithmetic operation with right operand
            match right {
                IrValue::IntConstant(val) => {
                    self.emit_instruction_with_size(
                        asm_op,
                        Size::Dword,
                        vec![Operand::Register(Register::Eax), Operand::Immediate(*val)]
                    );
                }
                IrValue::Local(var_name) => {
                    if let Some(offset) = self.stack_manager.get_local_offset(var_name) {
                        self.emit_instruction_with_size(
                            asm_op,
                            Size::Dword,
                            vec![
                                Operand::Register(Register::Eax),
                                Operand::Memory { base: Register::Rbp, offset: offset }
                            ]
                        );
                    }
                }
                _ => {}
            }
        }
        
        // Store result to destination
        if let IrValue::Local(dest_name) = dest {
            if let Some(dest_offset) = self.stack_manager.get_local_offset(dest_name) {
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
    }
    
    /// Generate function call
    fn generate_call(&mut self, _dest: &Option<IrValue>, function: &str, _args: &[IrValue]) {
        self.emit_comment(&format!("Call function: {}", function));
        let call_instructions = self.target.format_function_call(function);
        for instr in call_instructions {
            self.emit_line(&format!("    {}", instr));
        }
    }
    
    /// Convert IR type to token type
    fn ir_type_to_token_type(&self, ir_type: &IrType) -> TokenType {
        match ir_type {
            IrType::Int => TokenType::Int,
            IrType::Float => TokenType::FloatType,
            IrType::Char => TokenType::CharType,
            IrType::Void => TokenType::Void,
            _ => TokenType::Int, // Default fallback
        }
    }
    
    /// Get the generated output
    pub fn output(&self) -> &str {
        &self.output
    }
    
    /// Get the stack manager
    pub fn stack_manager(&self) -> &StackManager {
        &self.stack_manager
    }
    
    /// Get the register allocator
    pub fn register_allocator(&self) -> &RegisterAllocator {
        &self.register_allocator
    }
}

impl Default for IrBackend {
    fn default() -> Self {
        Self::new()
    }
}

// Implement Emitter trait for IrBackend
impl Emitter for IrBackend {
    fn emit_line(&mut self, line: &str) {
        self.output.push_str(line);
        self.output.push('\n');
    }
    
    fn emit_comment(&mut self, comment: &str) {
        self.output.push_str(&format!("; {}\n", comment));
    }
}