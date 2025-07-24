use std::collections::HashMap;
use crate::ir::{IrProgram, IrFunction, IrInstruction, IrValue, IrType, IrBinaryOp, IrUnaryOp};
use super::instruction::{Instruction, Operand, Register, Size};
use super::emitter::{Emitter, CodeEmitter, CodeEmitterWithComment};

/// IR-based code generator that produces assembly from IR
pub struct IrCodegen {
    pub output: String,
    pub stack_offset: i32,
    pub locals: HashMap<String, i32>,
    pub temp_locations: HashMap<usize, i32>, // Map temp variables to stack locations
    pub data_strings: HashMap<String, String>,
    pub label_count: usize,
}

impl IrCodegen {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            stack_offset: 0,
            locals: HashMap::new(),
            temp_locations: HashMap::new(),
            data_strings: HashMap::new(),
            label_count: 0,
        }
    }

    /// Generate assembly from IR program
    pub fn generate(mut self, ir_program: &IrProgram) -> String {
        // Assembly file header
        self.emit_section_header("MINI-C COMPILER GENERATED ASSEMBLY (FROM IR)");
        self.emit_comment("Target: x86-64 Windows");
        self.emit_comment("Calling Convention: Microsoft x64");
        self.emit_comment("Generated from: Intermediate Representation");
        self.emit_line("");
        
        // Assembly directives
        self.emit_comment("Assembly configuration");
        self.emit_line("bits 64");
        self.emit_line("default rel");
        self.emit_line("global main");
        self.emit_line("extern printf");

        // Data section - process global strings
        self.emit_section_header("DATA SECTION - String Literals and Constants");
        self.emit_line("section .data");

        if ir_program.global_strings.is_empty() {
            self.emit_comment("No string literals found");
        } else {
            for (label, content) in &ir_program.global_strings {
                let formatted_content = content.replace('\n', "").replace("%f", "%.2f");
                self.emit_comment(&format!("String constant: \"{}\"", content.replace('\n', "\\n")));
                self.emit_line(&format!("    {}: db \"{}\", 10, 0", label, formatted_content));
                self.data_strings.insert(label.clone(), content.clone());
            }
        }

        // Text section
        self.emit_section_header("TEXT SECTION - Executable Code");
        self.emit_line("section .text");

        // Generate code for each function
        for function in &ir_program.functions {
            self.generate_function(function);
        }

        self.output
    }

    /// Generate assembly for a single function
    fn generate_function(&mut self, function: &IrFunction) {
        self.emit_subsection_header(&format!("FUNCTION: {}", function.name));
        self.emit_line(&format!("{}:", function.name));
        
        // Reset state for new function
        self.stack_offset = 0;
        self.locals.clear();
        self.temp_locations.clear();

        // Function prologue
        self.emit_subsection_header("Function Prologue");
        self.emit_instruction_with_comment(Instruction::Push, vec![
            Operand::Register(Register::Rbp)
        ], Some("save caller's frame"));
        self.emit_instruction_with_comment(Instruction::Mov, vec![
            Operand::Register(Register::Rbp), 
            Operand::Register(Register::Rsp)
        ], Some("set up frame"));

        // Calculate stack space needed
        let stack_space = self.calculate_stack_space(function);
        if stack_space > 0 {
            self.emit_instruction_with_comment(Instruction::Sub, vec![
                Operand::Register(Register::Rsp), 
                Operand::Immediate(stack_space as i64)
            ], Some(&format!("allocate {} bytes for locals and temps", stack_space)));
        }

        // Generate function body
        self.emit_subsection_header("Function Body");
        for instruction in &function.instructions {
            self.generate_instruction(instruction);
        }

        // Function epilogue
        self.emit_subsection_header("Function Epilogue");
        self.emit_stack_layout_summary();
        
        if stack_space > 0 {
            self.emit_instruction_with_comment(Instruction::Add, vec![
                Operand::Register(Register::Rsp), 
                Operand::Immediate(stack_space as i64)
            ], Some("deallocate stack space"));
        }
        
        self.emit_instruction_with_comment(Instruction::Pop, vec![
            Operand::Register(Register::Rbp)
        ], Some("restore frame"));
        self.emit_instruction_with_comment(Instruction::Ret, vec![], Some("return"));
        
        self.emit_line(""); // Add spacing after function
    }

    /// Calculate the stack space needed for a function
    fn calculate_stack_space(&mut self, function: &IrFunction) -> i32 {
        let mut space = 32; // Shadow space for Windows x64 ABI
        
        // Allocate space for local variables
        for (name, ir_type) in &function.local_vars {
            let size = self.get_type_size(ir_type);
            space += size;
            self.locals.insert(name.clone(), -space);
        }
        
        // Allocate space for temporary variables
        let mut _temp_count = 0;
        for instruction in &function.instructions {
            if let Some(temp_id) = self.extract_temp_id(instruction) {
                if !self.temp_locations.contains_key(&temp_id) {
                    _temp_count += 1;
                    space += 8; // Assume 8 bytes for all temps
                    self.temp_locations.insert(temp_id, -space);
                }
            }
        }
        
        // Align to 16 bytes
        (space + 15) & !15
    }

    /// Extract temporary variable ID from instruction if present
    fn extract_temp_id(&self, instruction: &IrInstruction) -> Option<usize> {
        match instruction {
            IrInstruction::BinaryOp { dest, .. } |
            IrInstruction::UnaryOp { dest, .. } |
            IrInstruction::Load { dest, .. } |
            IrInstruction::Move { dest, .. } => {
                if let IrValue::Temp(id) = dest {
                    Some(*id)
                } else {
                    None
                }
            }
            IrInstruction::Call { dest: Some(dest), .. } => {
                if let IrValue::Temp(id) = dest {
                    Some(*id)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Get the size in bytes for an IR type
    fn get_type_size(&self, ir_type: &IrType) -> i32 {
        match ir_type {
            IrType::Int => 4,
            IrType::Float => 8,
            IrType::Char => 1,
            IrType::String => 8, // Pointer size
            IrType::Void => 0,
            IrType::Pointer(_) => 8,
        }
    }

    /// Generate assembly for a single IR instruction
    fn generate_instruction(&mut self, instruction: &IrInstruction) {
        match instruction {
            IrInstruction::Alloca { var_type, name } => {
                // Space already allocated in prologue, just add comment
                let size = self.get_type_size(var_type);
                let offset = self.locals.get(name).copied().unwrap_or(0);
                self.emit_comment(&format!("alloca {} {} at [rbp{}] ({} bytes)", 
                    var_type, name, offset, size));
            }

            IrInstruction::Load { dest, src, var_type } => {
                let src_operand = self.ir_value_to_operand(src);
                let dest_operand = self.ir_value_to_operand(dest);
                let size = self.ir_type_to_size(var_type);
                
                // Use register as intermediate for memory-to-memory moves
                let reg = match size {
                    Size::Byte => Register::Al,
                    Size::Dword => Register::Eax,
                    Size::Qword => Register::Rax,
                    _ => Register::Eax,
                };
                
                self.emit_instruction_with_size_and_comment(Instruction::Mov, size, vec![
                    Operand::Register(reg),
                    src_operand
                ], Some(&format!("load {} {} to register", var_type, self.ir_value_to_string(src))));
                
                self.emit_instruction_with_size_and_comment(Instruction::Mov, size, vec![
                    dest_operand,
                    Operand::Register(reg)
                ], Some("store to destination"));
            }

            IrInstruction::Store { value, dest, var_type } => {
                let value_operand = self.ir_value_to_operand(value);
                let dest_operand = self.ir_value_to_operand(dest);
                let size = self.ir_type_to_size(var_type);
                
                // Handle different value types appropriately
                match (value, var_type) {
                    (IrValue::FloatConstant(f), IrType::Float) => {
                        // For float constants, we need to handle them specially
                        self.emit_comment(&format!("store float constant {} to {}", f, self.ir_value_to_string(dest)));
                        // Move the float bits as integer first, then convert
                        let bits = f.to_bits() as i64;
                        self.emit_instruction_with_comment(Instruction::Mov, vec![
                            Operand::Register(Register::Rax),
                            Operand::Immediate(bits)
                        ], Some("load float bits"));
                        self.emit_instruction_with_size_and_comment(Instruction::Mov, Size::Qword, vec![
                            dest_operand,
                            Operand::Register(Register::Rax)
                        ], Some("store float"));
                    }
                    _ => {
                        // For other types, use register as intermediate if needed
                        let reg = match size {
                            Size::Byte => Register::Al,
                            Size::Dword => Register::Eax,
                            Size::Qword => Register::Rax,
                            _ => Register::Eax,
                        };
                        
                        // Check if we need an intermediate register
                        let needs_intermediate = matches!(value_operand, Operand::Memory { .. }) && 
                                               matches!(dest_operand, Operand::Memory { .. });
                        
                        if needs_intermediate {
                            self.emit_instruction_with_size_and_comment(Instruction::Mov, size, vec![
                                Operand::Register(reg),
                                value_operand
                            ], Some(&format!("load {} to register", self.ir_value_to_string(value))));
                            
                            self.emit_instruction_with_size_and_comment(Instruction::Mov, size, vec![
                                dest_operand,
                                Operand::Register(reg)
                            ], Some(&format!("store to {}", self.ir_value_to_string(dest))));
                        } else {
                            self.emit_instruction_with_size_and_comment(Instruction::Mov, size, vec![
                                dest_operand,
                                value_operand
                            ], Some(&format!("store {} to {}", self.ir_value_to_string(value), self.ir_value_to_string(dest))));
                        }
                    }
                }
            }

            IrInstruction::BinaryOp { dest, op, left, right, var_type } => {
                self.generate_binary_op(dest, op, left, right, var_type);
            }

            IrInstruction::UnaryOp { dest, op, operand, var_type } => {
                self.generate_unary_op(dest, op, operand, var_type);
            }

            IrInstruction::Call { dest, func, args, return_type } => {
                self.generate_function_call(dest, func, args, return_type);
            }

            IrInstruction::Branch { condition, true_label, false_label } => {
                let condition_operand = self.ir_value_to_operand(condition);
                
                // Load condition to register first, then compare
                self.emit_instruction_with_comment(Instruction::Mov, vec![
                    Operand::Register(Register::Eax),
                    condition_operand
                ], Some("load condition"));
                
                self.emit_instruction_with_comment(Instruction::Cmp, vec![
                    Operand::Register(Register::Eax),
                    Operand::Immediate(0)
                ], Some("test condition"));
                
                self.emit_instruction(Instruction::Je, vec![
                    Operand::Label(false_label.clone())
                ]);
                self.emit_instruction(Instruction::Jmp, vec![
                    Operand::Label(true_label.clone())
                ]);
            }

            IrInstruction::Jump { label } => {
                self.emit_instruction(Instruction::Jmp, vec![
                    Operand::Label(label.clone())
                ]);
            }

            IrInstruction::Label { name } => {
                self.emit_line(&format!("{}:", name));
            }

            IrInstruction::Return { value, var_type } => {
                if let Some(val) = value {
                    let val_operand = self.ir_value_to_operand(val);
                    let register = match var_type {
                        IrType::Float => Register::Xmm0,
                        _ => Register::Eax,
                    };
                    
                    self.emit_instruction_with_comment(Instruction::Mov, vec![
                        Operand::Register(register),
                        val_operand
                    ], Some(&format!("return {}", self.ir_value_to_string(val))));
                } else {
                    self.emit_instruction_with_comment(Instruction::Xor, vec![
                        Operand::Register(Register::Eax),
                        Operand::Register(Register::Eax)
                    ], Some("return 0"));
                }
            }

            IrInstruction::Print { format_string, args } => {
                self.generate_print_call(format_string, args);
            }

            IrInstruction::Move { dest, src, var_type } => {
                let src_operand = self.ir_value_to_operand(src);
                let dest_operand = self.ir_value_to_operand(dest);
                let size = self.ir_type_to_size(var_type);
                
                // Use register as intermediate for memory-to-memory moves
                let needs_intermediate = matches!(src_operand, Operand::Memory { .. }) && 
                                       matches!(dest_operand, Operand::Memory { .. });
                
                if needs_intermediate {
                    let reg = match size {
                        Size::Byte => Register::Al,
                        Size::Dword => Register::Eax,
                        Size::Qword => Register::Rax,
                        _ => Register::Eax,
                    };
                    
                    self.emit_instruction_with_size_and_comment(Instruction::Mov, size, vec![
                        Operand::Register(reg),
                        src_operand
                    ], Some(&format!("load {} to register", self.ir_value_to_string(src))));
                    
                    self.emit_instruction_with_size_and_comment(Instruction::Mov, size, vec![
                        dest_operand,
                        Operand::Register(reg)
                    ], Some(&format!("move to {}", self.ir_value_to_string(dest))));
                } else {
                    self.emit_instruction_with_size_and_comment(Instruction::Mov, size, vec![
                        dest_operand,
                        src_operand
                    ], Some(&format!("move {} to {}", self.ir_value_to_string(src), self.ir_value_to_string(dest))));
                }
            }

            IrInstruction::Convert { dest, dest_type, src, src_type } => {
                // Type conversion - simplified implementation
                let src_operand = self.ir_value_to_operand(src);
                let dest_operand = self.ir_value_to_operand(dest);
                
                self.emit_comment(&format!("convert {} {} to {} {}", 
                    src_type, self.ir_value_to_string(src), dest_type, self.ir_value_to_string(dest)));
                
                // For now, just move (would need proper conversion logic)
                self.emit_instruction(Instruction::Mov, vec![dest_operand, src_operand]);
            }

            IrInstruction::Comment { text } => {
                self.emit_comment(text);
            }
        }
    }

    /// Generate binary operation
    fn generate_binary_op(&mut self, dest: &IrValue, op: &IrBinaryOp, left: &IrValue, right: &IrValue, var_type: &IrType) {
        let left_operand = self.ir_value_to_operand(left);
        let right_operand = self.ir_value_to_operand(right);
        let dest_operand = self.ir_value_to_operand(dest);
        
        match var_type {
            IrType::Float => {
                // Floating point operations
                self.emit_instruction_with_comment(Instruction::Movsd, vec![
                    Operand::Register(Register::Xmm0),
                    left_operand
                ], Some("load left operand"));
                
                let asm_op = match op {
                    IrBinaryOp::Add => Instruction::Addsd,
                    IrBinaryOp::Sub => Instruction::Subsd,
                    IrBinaryOp::Mul => Instruction::Mulsd,
                    IrBinaryOp::Div => Instruction::Divsd,
                    _ => {
                        self.emit_comment(&format!("Unsupported float operation: {}", op));
                        return;
                    }
                };
                
                self.emit_instruction_with_comment(asm_op, vec![
                    Operand::Register(Register::Xmm0),
                    right_operand
                ], Some(&format!("{} operation", op)));
                
                self.emit_instruction_with_comment(Instruction::Movsd, vec![
                    dest_operand,
                    Operand::Register(Register::Xmm0)
                ], Some("store result"));
            }
            _ => {
                // Integer operations
                self.emit_instruction_with_comment(Instruction::Mov, vec![
                    Operand::Register(Register::Eax),
                    left_operand
                ], Some("load left operand"));
                
                let asm_op = match op {
                    IrBinaryOp::Add => Instruction::Add,
                    IrBinaryOp::Sub => Instruction::Sub,
                    IrBinaryOp::Mul => Instruction::Imul,
                    IrBinaryOp::Div => {
                        // Division requires special handling
                        self.emit_instruction(Instruction::Cdq, vec![]);
                        self.emit_instruction(Instruction::Idiv, vec![right_operand]);
                        self.emit_instruction(Instruction::Mov, vec![dest_operand, Operand::Register(Register::Eax)]);
                        return;
                    }
                    IrBinaryOp::Eq | IrBinaryOp::Ne | IrBinaryOp::Lt | 
                    IrBinaryOp::Le | IrBinaryOp::Gt | IrBinaryOp::Ge => {
                        // Comparison operations
                        self.emit_instruction(Instruction::Cmp, vec![
                            Operand::Register(Register::Eax),
                            right_operand
                        ]);
                        
                        let set_op = match op {
                            IrBinaryOp::Eq => Instruction::Sete,
                            IrBinaryOp::Ne => Instruction::Setne,
                            IrBinaryOp::Lt => Instruction::Setl,
                            IrBinaryOp::Le => Instruction::Setle,
                            IrBinaryOp::Gt => Instruction::Setg,
                            IrBinaryOp::Ge => Instruction::Setge,
                            _ => unreachable!(),
                        };
                        
                        self.emit_instruction(set_op, vec![Operand::Register(Register::Al)]);
                        self.emit_instruction(Instruction::Movzx, vec![
                            Operand::Register(Register::Eax),
                            Operand::Register(Register::Al)
                        ]);
                        self.emit_instruction(Instruction::Mov, vec![dest_operand, Operand::Register(Register::Eax)]);
                        return;
                    }
                    _ => {
                        self.emit_comment(&format!("Unsupported operation: {}", op));
                        return;
                    }
                };
                
                self.emit_instruction_with_comment(asm_op, vec![
                    Operand::Register(Register::Eax),
                    right_operand
                ], Some(&format!("{} operation", op)));
                
                self.emit_instruction_with_comment(Instruction::Mov, vec![
                    dest_operand,
                    Operand::Register(Register::Eax)
                ], Some("store result"));
            }
        }
    }

    /// Generate unary operation
    fn generate_unary_op(&mut self, dest: &IrValue, op: &IrUnaryOp, operand: &IrValue, _var_type: &IrType) {
        let operand_op = self.ir_value_to_operand(operand);
        let dest_operand = self.ir_value_to_operand(dest);
        
        match op {
            IrUnaryOp::Neg => {
                self.emit_instruction_with_comment(Instruction::Mov, vec![
                    Operand::Register(Register::Eax),
                    operand_op
                ], Some("load operand"));
                
                self.emit_instruction_with_comment(Instruction::Neg, vec![
                    Operand::Register(Register::Eax)
                ], Some("negate"));
                
                self.emit_instruction_with_comment(Instruction::Mov, vec![
                    dest_operand,
                    Operand::Register(Register::Eax)
                ], Some("store result"));
            }
            IrUnaryOp::Not => {
                self.emit_instruction_with_comment(Instruction::Mov, vec![
                    Operand::Register(Register::Eax),
                    operand_op
                ], Some("load operand"));
                
                self.emit_instruction_with_comment(Instruction::Cmp, vec![
                    Operand::Register(Register::Eax),
                    Operand::Immediate(0)
                ], Some("test for zero"));
                
                self.emit_instruction(Instruction::Sete, vec![Operand::Register(Register::Al)]);
                self.emit_instruction(Instruction::Movzx, vec![
                    Operand::Register(Register::Eax),
                    Operand::Register(Register::Al)
                ]);
                
                self.emit_instruction_with_comment(Instruction::Mov, vec![
                    dest_operand,
                    Operand::Register(Register::Eax)
                ], Some("store result"));
            }
        }
    }

    /// Generate function call
    fn generate_function_call(&mut self, dest: &Option<IrValue>, func: &str, args: &[IrValue], return_type: &IrType) {
        self.emit_comment(&format!("call {} with {} args", func, args.len()));
        
        // For now, simplified function call handling
        // In a real implementation, you'd handle calling conventions properly
        
        if let Some(dest_val) = dest {
            let dest_operand = self.ir_value_to_operand(dest_val);
            let register = match return_type {
                IrType::Float => Register::Xmm0,
                _ => Register::Eax,
            };
            
            self.emit_instruction_with_comment(Instruction::Mov, vec![
                dest_operand,
                Operand::Register(register)
            ], Some("store return value"));
        }
    }

    /// Generate print call
    fn generate_print_call(&mut self, format_string: &IrValue, args: &[IrValue]) {
        self.emit_comment("--- print statement ---");
        
        // Handle printf call - simplified implementation
        if let IrValue::StringConstant(label) = format_string {
            self.emit_instruction_with_comment(Instruction::Lea, vec![
                Operand::Register(Register::Rcx),
                Operand::Label(label.clone())
            ], Some("load format string"));
            
            // Load arguments into registers (simplified)
            for (i, arg) in args.iter().enumerate() {
                let arg_operand = self.ir_value_to_operand(arg);
                let reg = match i {
                    0 => Register::Rdx,
                    1 => Register::R8,
                    2 => Register::R9,
                    _ => {
                        self.emit_comment("Too many arguments for simplified printf");
                        break;
                    }
                };
                
                // For memory operands, use intermediate register with proper size
                if matches!(arg_operand, Operand::Memory { .. }) {
                    self.emit_instruction_with_comment(Instruction::Mov, vec![
                        Operand::Register(Register::Eax),
                        arg_operand
                    ], Some(&format!("load arg {} to eax", i)));
                    
                    // Use movsxd to sign-extend 32-bit to 64-bit for 64-bit registers
                    match reg {
                        Register::Rdx | Register::R8 | Register::R9 => {
                            self.emit_instruction_with_comment(Instruction::Mov, vec![
                                Operand::Register(reg),
                                Operand::Register(Register::Rax)
                            ], Some(&format!("move to arg register {}", i)));
                        }
                        _ => {
                            self.emit_instruction_with_comment(Instruction::Mov, vec![
                                Operand::Register(reg),
                                Operand::Register(Register::Eax)
                            ], Some(&format!("move to arg register {}", i)));
                        }
                    }
                } else {
                    self.emit_instruction_with_comment(Instruction::Mov, vec![
                        Operand::Register(reg),
                        arg_operand
                    ], Some(&format!("load arg {}", i)));
                }
            }
            
            self.emit_instruction_with_comment(Instruction::Call, vec![
                Operand::Label("printf".to_string())
            ], Some("call printf"));
        }
    }

    /// Convert IR value to assembly operand
    fn ir_value_to_operand(&self, value: &IrValue) -> Operand {
        match value {
            IrValue::IntConstant(i) => Operand::Immediate(*i),
            IrValue::FloatConstant(f) => {
                // For floats, we'd need to handle this differently in a real implementation
                Operand::Immediate(f.to_bits() as i64)
            }
            IrValue::CharConstant(c) => Operand::Immediate(*c as i64),
            IrValue::StringConstant(label) => Operand::Label(label.clone()),
            IrValue::Local(name) => {
                let offset = self.locals.get(name).copied().unwrap_or(0);
                Operand::Memory { base: Register::Rbp, offset }
            }
            IrValue::Temp(id) => {
                let offset = self.temp_locations.get(id).copied().unwrap_or(0);
                Operand::Memory { base: Register::Rbp, offset }
            }
            IrValue::Parameter(_name) => {
                // Parameters would be at positive offsets from RBP
                let offset = 16; // Simplified - would need proper parameter handling
                Operand::Memory { base: Register::Rbp, offset }
            }
            IrValue::Global(name) => Operand::Label(name.clone()),
        }
    }

    /// Convert IR type to assembly size
    fn ir_type_to_size(&self, ir_type: &IrType) -> Size {
        match ir_type {
            IrType::Int => Size::Dword,
            IrType::Float => Size::Qword,
            IrType::Char => Size::Byte,
            IrType::String => Size::Qword,
            IrType::Void => Size::Qword,
            IrType::Pointer(_) => Size::Qword,
        }
    }

    /// Convert IR value to string for comments
    fn ir_value_to_string(&self, value: &IrValue) -> String {
        match value {
            IrValue::IntConstant(i) => i.to_string(),
            IrValue::FloatConstant(f) => f.to_string(),
            IrValue::CharConstant(c) => format!("'{}'", c),
            IrValue::StringConstant(label) => format!("@{}", label),
            IrValue::Local(name) => format!("%{}", name),
            IrValue::Temp(id) => format!("%t{}", id),
            IrValue::Parameter(name) => format!("%{}", name),
            IrValue::Global(name) => format!("@{}", name),
        }
    }
}

// Implement the emitter traits for IrCodegen
impl Emitter for IrCodegen {
    fn emit_line(&mut self, line: &str) {
        self.output.push_str(line);
        self.output.push('\n');
    }

    fn emit_comment(&mut self, comment: &str) {
        self.emit_line(&format!("; {}", comment));
    }
}

// Helper methods for IrCodegen
impl IrCodegen {
    /// Emit a section header with clear visual separation
    pub fn emit_section_header(&mut self, title: &str) {
        self.emit_line("");
        self.emit_line(&format!("; {}", "=".repeat(60)));
        self.emit_line(&format!("; {}", title));
        self.emit_line(&format!("; {}", "=".repeat(60)));
        self.emit_line("");
    }

    /// Emit a subsection header for better organization
    pub fn emit_subsection_header(&mut self, title: &str) {
        self.emit_line("");
        self.emit_line(&format!("; {}", "-".repeat(40)));
        self.emit_line(&format!("; {}", title));
        self.emit_line(&format!("; {}", "-".repeat(40)));
    }

    /// Emit a stack layout summary for debugging
    pub fn emit_stack_layout_summary(&mut self) {
        self.emit_comment("STACK LAYOUT SUMMARY:");
        self.emit_comment("RBP+0  : Saved RBP (caller's frame pointer)");
        
        if self.locals.is_empty() && self.temp_locations.is_empty() {
            self.emit_comment("No local variables or temporaries allocated");
        } else {
            // Collect local variables info to avoid borrowing issues
            let locals_info: Vec<(String, i32)> = self.locals.iter()
                .map(|(name, &offset)| (name.clone(), offset))
                .collect();
            
            if !locals_info.is_empty() {
                self.emit_comment("Local variables:");
                for (name, offset) in locals_info {
                    self.emit_comment(&format!("RBP{:3} : {}", offset, name));
                }
            }
            
            // Collect temp variables info to avoid borrowing issues
            let temps_info: Vec<(usize, i32)> = self.temp_locations.iter()
                .map(|(&temp_id, &offset)| (temp_id, offset))
                .collect();
            
            if !temps_info.is_empty() {
                self.emit_comment("Temporary variables:");
                for (temp_id, offset) in temps_info {
                    self.emit_comment(&format!("RBP{:3} : %t{}", offset, temp_id));
                }
            }
        }
        self.emit_line("");
    }
}