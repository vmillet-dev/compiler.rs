use crate::ir::{IrValue, IrType};
use crate::codegen::core::{Instruction, Operand, Register};
use crate::codegen::core::{Emitter, CodeEmitterWithComment};
use crate::codegen::Codegen;

impl Codegen {
    /// Generate function call
    pub fn generate_function_call(&mut self, dest: &Option<IrValue>, func: &str, args: &[IrValue], return_type: &IrType) {
        self.emit_comment(&format!("call {} with {} args", func, args.len()));
        
        // For now, simplified function call handling
        // In a real implementation, you'd handle calling conventions properly
        
        if let Some(dest_val) = dest {
            let dest_operand = self.ir_value_to_operand(dest_val);
            let register = match return_type {
                IrType::Float => Register::Xmm0,
                _ => self.target.return_register(),
            };
            
            match return_type {
                IrType::Float => {
                    self.emit_instruction_with_comment(Instruction::Movsd, vec![
                        dest_operand,
                        Operand::Register(register)
                    ], Some("store return value"));
                }
                _ => {
                    self.emit_instruction_with_comment(Instruction::Mov, vec![
                        dest_operand,
                        Operand::Register(register)
                    ], Some("store return value"));
                }
            }
        }
    }

    /// Generate print call
    pub fn generate_print_call(&mut self, format_string: &IrValue, args: &[IrValue]) {
        self.emit_comment("--- print statement ---");
        
        // Handle printf call - simplified implementation
        if let IrValue::StringConstant(label) = format_string {
            let param_regs = self.target.parameter_registers();
            if !param_regs.is_empty() {
                self.emit_instruction_with_comment(Instruction::Lea, vec![
                    Operand::Register(param_regs[0]),
                    Operand::Label(label.clone())
                ], Some("load format string"));
            }
            
            // Load arguments into registers with proper float handling
            for (i, arg) in args.iter().enumerate() {
                if i + 1 >= param_regs.len() {
                    self.emit_comment("Too many arguments for simplified printf");
                    break;
                }
                let reg = param_regs[i + 1]; // +1 because first param is format string
                
                // Handle different argument types
                match arg {
                    IrValue::FloatConstant(f) => {
                        // For float constants, load the float bits into a register and then move to arg register
                        let float_bits = f.to_bits() as i64;
                        self.emit_instruction_with_comment(Instruction::Mov, vec![
                            Operand::Register(Register::Rax),
                            Operand::Immediate(float_bits)
                        ], Some(&format!("load float bits for arg {}", i)));
                        self.emit_instruction_with_comment(Instruction::Mov, vec![
                            Operand::Register(reg),
                            Operand::Register(Register::Rax)
                        ], Some(&format!("move to arg register {}", i)));
                    }
                    IrValue::Temp(_) | IrValue::Local(_) => {
                        let arg_operand = self.ir_value_to_operand(arg);
                        // Check if this is a float by looking at the memory location
                        // For now, assume temp variables that are floats need special handling
                        if let IrValue::Temp(_temp_id) = arg {
                            // Check if this temp was created from a float operation
                            if matches!(arg_operand, Operand::Memory { .. }) {
                                // For now, load as 64-bit value (could be float or int)
                                self.emit_instruction_with_comment(Instruction::Mov, vec![
                                    Operand::Register(Register::Rax),
                                    arg_operand
                                ], Some(&format!("load arg {} to register", i)));
                                self.emit_instruction_with_comment(Instruction::Mov, vec![
                                    Operand::Register(reg),
                                    Operand::Register(Register::Rax)
                                ], Some(&format!("move to arg register {}", i)));
                            }
                        } else if let IrValue::Local(_) = arg {
                            if matches!(arg_operand, Operand::Memory { .. }) {
                                self.emit_instruction_with_comment(Instruction::Mov, vec![
                                    Operand::Register(Register::Rax),
                                    arg_operand
                                ], Some(&format!("load arg {} to register", i)));
                                self.emit_instruction_with_comment(Instruction::Mov, vec![
                                    Operand::Register(reg),
                                    Operand::Register(Register::Rax)
                                ], Some(&format!("move to arg register {}", i)));
                            }
                        }
                    }
                    _ => {
                        // Handle other types (int constants, char constants, etc.)
                        let arg_operand = self.ir_value_to_operand(arg);
                        self.emit_instruction_with_comment(Instruction::Mov, vec![
                            Operand::Register(reg),
                            arg_operand
                        ], Some(&format!("load arg {}", i)));
                    }
                }
            }
            
            let call_instructions = self.target.format_function_call("printf");
            for call_instr in call_instructions {
                self.emit_line_with_comment(&format!("    {}", call_instr), Some("call printf"));
            }
        }
    }
}