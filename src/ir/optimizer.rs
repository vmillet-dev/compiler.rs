use super::ir::{IrProgram, IrFunction, IrInstruction, IrValue, IrBinaryOp};
use std::collections::HashMap;

/// IR Optimizer - performs optimization passes on IR
pub struct IrOptimizer {
    /// Enable/disable specific optimizations
    pub constant_folding: bool,
    pub dead_code_elimination: bool,
    pub copy_propagation: bool,
}

impl IrOptimizer {
    pub fn new() -> Self {
        Self {
            constant_folding: true,
            dead_code_elimination: true,
            copy_propagation: true,
        }
    }

    /// Optimize an IR program
    pub fn optimize(&mut self, mut program: IrProgram) -> IrProgram {
        // Apply optimizations to each function
        for function in &mut program.functions {
            self.optimize_function(function);
        }
        
        program
    }

    /// Optimize a single function
    fn optimize_function(&mut self, function: &mut IrFunction) {
        if self.constant_folding {
            self.constant_folding_pass(function);
        }
        
        if self.copy_propagation {
            self.copy_propagation_pass(function);
        }
        
        if self.dead_code_elimination {
            self.dead_code_elimination_pass(function);
        }
    }

    /// Constant folding optimization pass
    fn constant_folding_pass(&mut self, function: &mut IrFunction) {
        let mut optimized_instructions = Vec::new();
        
        for instruction in &function.instructions {
            match instruction {
                IrInstruction::BinaryOp { dest, op, left, right, var_type } => {
                    // Try to fold constants
                    if let (IrValue::IntConstant(l), IrValue::IntConstant(r)) = (left, right) {
                        let result = match op {
                            IrBinaryOp::Add => l + r,
                            IrBinaryOp::Sub => l - r,
                            IrBinaryOp::Mul => l * r,
                            IrBinaryOp::Div if *r != 0 => l / r,
                            IrBinaryOp::Mod if *r != 0 => l % r,
                            IrBinaryOp::Eq => if l == r { 1 } else { 0 },
                            IrBinaryOp::Ne => if l != r { 1 } else { 0 },
                            IrBinaryOp::Lt => if l < r { 1 } else { 0 },
                            IrBinaryOp::Le => if l <= r { 1 } else { 0 },
                            IrBinaryOp::Gt => if l > r { 1 } else { 0 },
                            IrBinaryOp::Ge => if l >= r { 1 } else { 0 },
                            _ => {
                                // Can't fold this operation, keep original
                                optimized_instructions.push(instruction.clone());
                                continue;
                            }
                        };
                        
                        // Replace with a move of the constant result
                        optimized_instructions.push(IrInstruction::Move {
                            dest: dest.clone(),
                            src: IrValue::IntConstant(result),
                            var_type: var_type.clone(),
                        });
                    } else if let (IrValue::FloatConstant(l), IrValue::FloatConstant(r)) = (left, right) {
                        let result = match op {
                            IrBinaryOp::Add => l + r,
                            IrBinaryOp::Sub => l - r,
                            IrBinaryOp::Mul => l * r,
                            IrBinaryOp::Div if *r != 0.0 => l / r,
                            IrBinaryOp::Eq => if (l - r).abs() < f64::EPSILON { 1.0 } else { 0.0 },
                            IrBinaryOp::Ne => if (l - r).abs() >= f64::EPSILON { 1.0 } else { 0.0 },
                            IrBinaryOp::Lt => if l < r { 1.0 } else { 0.0 },
                            IrBinaryOp::Le => if l <= r { 1.0 } else { 0.0 },
                            IrBinaryOp::Gt => if l > r { 1.0 } else { 0.0 },
                            IrBinaryOp::Ge => if l >= r { 1.0 } else { 0.0 },
                            _ => {
                                optimized_instructions.push(instruction.clone());
                                continue;
                            }
                        };
                        
                        optimized_instructions.push(IrInstruction::Move {
                            dest: dest.clone(),
                            src: IrValue::FloatConstant(result),
                            var_type: var_type.clone(),
                        });
                    } else {
                        // Check for algebraic identities
                        match (op, left, right) {
                            // x + 0 = x
                            (IrBinaryOp::Add, val, IrValue::IntConstant(0)) |
                            (IrBinaryOp::Add, IrValue::IntConstant(0), val) => {
                                optimized_instructions.push(IrInstruction::Move {
                                    dest: dest.clone(),
                                    src: val.clone(),
                                    var_type: var_type.clone(),
                                });
                            }
                            // x - 0 = x
                            (IrBinaryOp::Sub, val, IrValue::IntConstant(0)) => {
                                optimized_instructions.push(IrInstruction::Move {
                                    dest: dest.clone(),
                                    src: val.clone(),
                                    var_type: var_type.clone(),
                                });
                            }
                            // x * 1 = x
                            (IrBinaryOp::Mul, val, IrValue::IntConstant(1)) |
                            (IrBinaryOp::Mul, IrValue::IntConstant(1), val) => {
                                optimized_instructions.push(IrInstruction::Move {
                                    dest: dest.clone(),
                                    src: val.clone(),
                                    var_type: var_type.clone(),
                                });
                            }
                            // x * 0 = 0
                            (IrBinaryOp::Mul, _, IrValue::IntConstant(0)) |
                            (IrBinaryOp::Mul, IrValue::IntConstant(0), _) => {
                                optimized_instructions.push(IrInstruction::Move {
                                    dest: dest.clone(),
                                    src: IrValue::IntConstant(0),
                                    var_type: var_type.clone(),
                                });
                            }
                            // x / 1 = x
                            (IrBinaryOp::Div, val, IrValue::IntConstant(1)) => {
                                optimized_instructions.push(IrInstruction::Move {
                                    dest: dest.clone(),
                                    src: val.clone(),
                                    var_type: var_type.clone(),
                                });
                            }
                            _ => {
                                optimized_instructions.push(instruction.clone());
                            }
                        }
                    }
                }
                _ => {
                    optimized_instructions.push(instruction.clone());
                }
            }
        }
        
        function.instructions = optimized_instructions;
    }

    /// Copy propagation optimization pass
    fn copy_propagation_pass(&mut self, function: &mut IrFunction) {
        let mut copy_map: HashMap<IrValue, IrValue> = HashMap::new();
        let mut optimized_instructions = Vec::new();
        
        for instruction in &function.instructions {
            match instruction {
                IrInstruction::Move { dest, src, var_type } => {
                    // Record the copy
                    copy_map.insert(dest.clone(), src.clone());
                    optimized_instructions.push(IrInstruction::Move {
                        dest: dest.clone(),
                        src: self.substitute_value(src, &copy_map),
                        var_type: var_type.clone(),
                    });
                }
                _ => {
                    // Substitute known copies in other instructions
                    let optimized_instruction = self.substitute_instruction(instruction, &copy_map);
                    optimized_instructions.push(optimized_instruction);
                }
            }
        }
        
        function.instructions = optimized_instructions;
    }

    /// Dead code elimination pass
    fn dead_code_elimination_pass(&mut self, function: &mut IrFunction) {
        let mut used_values = std::collections::HashSet::new();
        
        // First pass: mark all used values
        for instruction in &function.instructions {
            match instruction {
                IrInstruction::Store { value, .. } => {
                    used_values.insert(value.clone());
                }
                IrInstruction::BinaryOp { left, right, .. } => {
                    used_values.insert(left.clone());
                    used_values.insert(right.clone());
                }
                IrInstruction::UnaryOp { operand, .. } => {
                    used_values.insert(operand.clone());
                }
                IrInstruction::Return { value: Some(val), .. } => {
                    used_values.insert(val.clone());
                }
                IrInstruction::Branch { condition, .. } => {
                    used_values.insert(condition.clone());
                }
                IrInstruction::Call { args, .. } => {
                    for arg in args {
                        used_values.insert(arg.clone());
                    }
                }
                IrInstruction::Print { format_string, args } => {
                    used_values.insert(format_string.clone());
                    for arg in args {
                        used_values.insert(arg.clone());
                    }
                }
                IrInstruction::Move { src, .. } => {
                    used_values.insert(src.clone());
                }
                _ => {}
            }
        }
        
        // Second pass: remove instructions that define unused values
        let mut optimized_instructions = Vec::new();
        for instruction in &function.instructions {
            let should_keep = match instruction {
                IrInstruction::BinaryOp { dest, .. } |
                IrInstruction::UnaryOp { dest, .. } |
                IrInstruction::Move { dest, .. } => {
                    used_values.contains(dest)
                }
                IrInstruction::Load { dest, .. } => {
                    used_values.contains(dest)
                }
                _ => true, // Keep all other instructions (they have side effects)
            };
            
            if should_keep {
                optimized_instructions.push(instruction.clone());
            }
        }
        
        function.instructions = optimized_instructions;
    }

    /// Substitute values in an instruction based on copy map
    fn substitute_instruction(&self, instruction: &IrInstruction, copy_map: &HashMap<IrValue, IrValue>) -> IrInstruction {
        match instruction {
            IrInstruction::Store { value, dest, var_type } => {
                IrInstruction::Store {
                    value: self.substitute_value(value, copy_map),
                    dest: dest.clone(),
                    var_type: var_type.clone(),
                }
            }
            IrInstruction::BinaryOp { dest, op, left, right, var_type } => {
                IrInstruction::BinaryOp {
                    dest: dest.clone(),
                    op: op.clone(),
                    left: self.substitute_value(left, copy_map),
                    right: self.substitute_value(right, copy_map),
                    var_type: var_type.clone(),
                }
            }
            IrInstruction::UnaryOp { dest, op, operand, var_type } => {
                IrInstruction::UnaryOp {
                    dest: dest.clone(),
                    op: op.clone(),
                    operand: self.substitute_value(operand, copy_map),
                    var_type: var_type.clone(),
                }
            }
            IrInstruction::Return { value, var_type } => {
                IrInstruction::Return {
                    value: value.as_ref().map(|v| self.substitute_value(v, copy_map)),
                    var_type: var_type.clone(),
                }
            }
            IrInstruction::Branch { condition, true_label, false_label } => {
                IrInstruction::Branch {
                    condition: self.substitute_value(condition, copy_map),
                    true_label: true_label.clone(),
                    false_label: false_label.clone(),
                }
            }
            _ => instruction.clone(),
        }
    }

    /// Substitute a value if it exists in the copy map
    fn substitute_value(&self, value: &IrValue, copy_map: &HashMap<IrValue, IrValue>) -> IrValue {
        copy_map.get(value).cloned().unwrap_or_else(|| value.clone())
    }
}

impl Default for IrOptimizer {
    fn default() -> Self {
        Self::new()
    }
}