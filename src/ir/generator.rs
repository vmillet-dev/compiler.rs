use crate::parser::ast::{Expr, Stmt, Parameter};
use crate::lexer::TokenType;
use crate::types::{Type, TypeChecker, TypeConstraint};
use super::ir::{IrProgram, IrFunction, IrInstruction, IrValue, IrType, IrBinaryOp, IrUnaryOp};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum IrGeneratorError {
    NestedFunctionsNotSupported,
    UnsupportedUnaryOperator(TokenType),
    ComplexFunctionCallsNotSupported,
    InvalidBinaryOperator(TokenType),
    UnsupportedConstruct(String),
}

/// IR Generator - converts AST to IR
pub struct IrGenerator {
    /// Counter for generating unique temporary variables
    temp_counter: usize,
    /// Counter for generating unique labels
    label_counter: usize,
    /// Current function being processed
    current_function: Option<IrFunction>,
    /// Global string constants
    string_constants: HashMap<String, String>,
    /// String label counter
    string_label_counter: usize,
    local_types: HashMap<String, IrType>,
    type_checker: TypeChecker,
    loop_stack: Vec<LoopContext>,
}

#[derive(Debug, Clone)]
struct LoopContext {
    continue_label: String,
    break_label: String,
}

impl IrGenerator {
    pub fn new() -> Self {
        Self {
            temp_counter: 0,
            label_counter: 0,
            current_function: None,
            string_constants: HashMap::new(),
            string_label_counter: 0,
            local_types: HashMap::new(),
            type_checker: TypeChecker::new(),
            loop_stack: Vec::new(),
        }
    }

    /// Generate IR from AST
    pub fn generate(&mut self, ast: &[Stmt]) -> Result<IrProgram, IrGeneratorError> {
        // First pass: collect variable types for symbol table
        self.collect_variable_types(ast);
        
        let mut functions = Vec::new();

        for stmt in ast {
            if let Stmt::Function { return_type, name, type_parameters, parameters, body } = stmt {
                let ir_function = self.generate_function(return_type, name, type_parameters, parameters, body)?;
                functions.push(ir_function);
            }
        }

        // Convert string constants to global strings
        let global_strings = self.string_constants.iter()
            .map(|(label, content)| (label.clone(), content.clone()))
            .collect();

        Ok(IrProgram {
            functions,
            global_strings,
        })
    }

    /// Generate a new temporary variable
    fn new_temp(&mut self) -> IrValue {
        let temp = IrValue::Temp(self.temp_counter);
        self.temp_counter += 1;
        temp
    }

    /// Generate a new label
    fn new_label(&mut self, prefix: &str) -> String {
        let label = format!("{}_{}", prefix, self.label_counter);
        self.label_counter += 1;
        label
    }

    /// Generate a string constant label
    fn get_string_label(&mut self, content: &str) -> String {
        // Check if we already have this string
        for (label, existing_content) in &self.string_constants {
            if existing_content == content {
                return label.clone();
            }
        }

        // Create new string label
        let label = format!("str_{}", self.string_label_counter);
        self.string_label_counter += 1;
        self.string_constants.insert(label.clone(), content.to_string());
        label
    }

    /// Generate IR for a function
    fn generate_function(&mut self, return_type: &Type, name: &str, type_parameters: &[String], parameters: &[Parameter], body: &[Stmt]) -> Result<IrFunction, IrGeneratorError> {
        for type_param in type_parameters {
            self.type_checker.add_constraint(type_param.clone(), TypeConstraint::Size(8)); // Default constraint
        }
        
        // Convert parameters to IR format
        let ir_parameters: Vec<(String, IrType)> = parameters.iter().map(|param| {
            let ir_type = if let Some(token_type) = param.param_type.to_token_type() {
                IrType::from(token_type)
            } else {
                IrType::Int // Default fallback
            };
            self.local_types.insert(param.name.clone(), ir_type.clone());
            (param.name.clone(), ir_type)
        }).collect();
        
        let function = IrFunction {
            name: name.to_string(),
            return_type: if let Some(token_type) = return_type.to_token_type() {
                IrType::from(token_type)
            } else {
                IrType::Int // Default fallback
            },
            parameters: ir_parameters,
            instructions: Vec::new(),
            local_vars: Vec::new(),
        };

        self.current_function = Some(function.clone());

        // Generate instructions for function body
        for stmt in body {
            self.generate_stmt(stmt)?;
        }

        // Ensure function has a return if it doesn't already
        if let Some(last_instruction) = self.current_function.as_ref().unwrap().instructions.last() {
            if !matches!(last_instruction, IrInstruction::Return { .. }) {
                if let Some(token_type) = return_type.to_token_type() {
                    match token_type {
                        TokenType::Void => {
                        self.emit_instruction(IrInstruction::Return {
                            value: None,
                            var_type: IrType::Void,
                        });
                    }
                        crate::lexer::TokenType::Int => {
                        self.emit_instruction(IrInstruction::Return {
                            value: Some(IrValue::IntConstant(0)),
                            var_type: IrType::Int,
                        });
                    }
                        _ => {
                            self.emit_instruction(IrInstruction::Return {
                                value: None,
                                var_type: IrType::Int, // Default fallback
                            });
                        }
                    }
                } else {
                    self.emit_instruction(IrInstruction::Return {
                        value: None,
                        var_type: IrType::Int, // Default fallback
                    });
                }
            }
        }

        Ok(self.current_function.take().unwrap_or_else(|| IrFunction {
            name: name.to_string(),
            return_type: IrType::from(return_type.to_token_type().unwrap_or(TokenType::Void)),
            parameters: Vec::new(),
            instructions: Vec::new(),
            local_vars: Vec::new(),
        }))
    }

    /// Emit an instruction to the current function
    fn emit_instruction(&mut self, instruction: IrInstruction) {
        if let Some(ref mut function) = self.current_function {
            function.instructions.push(instruction);
        }
    }

    /// Generate IR for a statement
    fn generate_stmt(&mut self, stmt: &Stmt) -> Result<(), IrGeneratorError> {
        match stmt {
            Stmt::VarDecl { var_type, name, initializer } => {
                let ir_type = if let Some(token_type) = var_type.to_token_type() {
                    IrType::from(token_type)
                } else {
                    IrType::Int // Default fallback
                };
                
                // Emit variable allocation
                self.emit_instruction(IrInstruction::Alloca {
                    var_type: ir_type.clone(),
                    name: name.clone(),
                });

                // Add to local variables
                if let Some(ref mut function) = self.current_function {
                    function.local_vars.push((name.clone(), ir_type.clone()));
                }

                // Handle initialization
                if let Some(init_expr) = initializer {
                    let init_value = self.generate_expr(init_expr);
                    self.emit_instruction(IrInstruction::Store {
                        value: init_value,
                        dest: IrValue::Local(name.clone()),
                        var_type: ir_type,
                    });
                }
            }

            Stmt::Return(expr_opt) => {
                if let Some(expr) = expr_opt {
                    let value = self.generate_expr(expr);
                    let return_type = self.infer_expr_type(expr);
                    self.emit_instruction(IrInstruction::Return {
                        value: Some(value),
                        var_type: return_type,
                    });
                } else {
                    self.emit_instruction(IrInstruction::Return {
                        value: None,
                        var_type: IrType::Void,
                    });
                }
            }

            Stmt::ExprStmt(expr) => {
                self.generate_expr(expr);
            }

            Stmt::Block(stmts) => {
                for stmt in stmts {
                    self.generate_stmt(stmt)?;
                }
            }

            Stmt::If { condition, then_branch } => {
                let condition_value = self.generate_expr(condition);
                let then_label = self.new_label("if_then");
                let end_label = self.new_label("if_end");

                // Branch based on condition
                self.emit_instruction(IrInstruction::Branch {
                    condition: condition_value,
                    true_label: then_label.clone(),
                    false_label: end_label.clone(),
                });

                // Then branch
                self.emit_instruction(IrInstruction::Label {
                    name: then_label,
                });
                for stmt in then_branch {
                    self.generate_stmt(stmt)?;
                }
                self.emit_instruction(IrInstruction::Jump {
                    label: end_label.clone(),
                });

                // End label
                self.emit_instruction(IrInstruction::Label {
                    name: end_label,
                });
            }

            Stmt::PrintStmt { format_string, args } => {
                // Handle format string generation similar to direct AST path
                if let Expr::String(s) = format_string {
                    if s.is_empty() && args.len() == 1 {
                        let arg = &args[0];
                        let format_str = match arg {
                            Expr::Integer(_) => "%d\n",
                            Expr::Float(_) => "%.6f\n", 
                            Expr::Char(_) => "%c\n",
                            Expr::Identifier(var_name) => {
                                // Use type inference for variables
                                let var_type = self.infer_identifier_type(var_name);
                                match var_type {
                                    IrType::Int => "%d\n",
                                    IrType::Float => "%.6f\n",
                                    IrType::Char => "%c\n",
                                    _ => "%d\n", // Default to integer
                                }
                            }
                            _ => "%d\n", // Default to integer format
                        };
                        
                        // Create string constant for the generated format string
                        let format_label = self.get_string_label(format_str);
                        let format_value = IrValue::StringConstant(format_label);
                        
                        let mut arg_values = Vec::new();
                        for arg in args {
                            arg_values.push(self.generate_expr(arg));
                        }

                        self.emit_instruction(IrInstruction::Print {
                            format_string: format_value,
                            args: arg_values,
                        });
                    } else {
                        let format_value = self.generate_expr(format_string);
                        let mut arg_values = Vec::new();
                        
                        for arg in args {
                            arg_values.push(self.generate_expr(arg));
                        }

                        self.emit_instruction(IrInstruction::Print {
                            format_string: format_value,
                            args: arg_values,
                        });
                    }
                } else {
                    let format_value = self.generate_expr(format_string);
                    let mut arg_values = Vec::new();
                    
                    for arg in args {
                        arg_values.push(self.generate_expr(arg));
                    }

                    self.emit_instruction(IrInstruction::Print {
                        format_string: format_value,
                        args: arg_values,
                    });
                }
            }

            Stmt::Function { .. } => {
                // Functions are handled at the top level
                return Err(IrGeneratorError::NestedFunctionsNotSupported);
            }
            
            Stmt::While { condition, body } => {
                let loop_start = self.new_label("loop_start");
                let loop_end = self.new_label("loop_end");
                
                self.loop_stack.push(LoopContext {
                    continue_label: loop_start.clone(),
                    break_label: loop_end.clone(),
                });
                
                self.emit_instruction(IrInstruction::Label { name: loop_start.clone() });
                
                // Evaluate condition
                let condition_val = self.generate_expr(condition);
                
                self.emit_instruction(IrInstruction::Branch {
                    condition: condition_val,
                    true_label: format!("loop_body_{}", self.label_counter - 1),
                    false_label: loop_end.clone(),
                });
                
                let body_label = format!("loop_body_{}", self.label_counter - 1);
                self.emit_instruction(IrInstruction::Label { name: body_label });
                
                // Generate body
                for stmt in body {
                    self.generate_stmt(stmt)?;
                }
                
                self.emit_instruction(IrInstruction::Jump { label: loop_start });
                
                self.emit_instruction(IrInstruction::Label { name: loop_end });
                
                self.loop_stack.pop();
            }
            
            Stmt::For { init, condition, update, body } => {
                // Generate initialization if present
                if let Some(init_stmt) = init {
                    self.generate_stmt(init_stmt)?;
                }
                
                let loop_start = self.new_label("for_start");
                let loop_continue = self.new_label("for_continue");
                let loop_end = self.new_label("for_end");
                
                self.loop_stack.push(LoopContext {
                    continue_label: loop_continue.clone(),
                    break_label: loop_end.clone(),
                });
                
                self.emit_instruction(IrInstruction::Label { name: loop_start.clone() });
                
                if let Some(cond) = condition {
                    let condition_val = self.generate_expr(cond);
                    self.emit_instruction(IrInstruction::Branch {
                        condition: condition_val,
                        true_label: format!("for_body_{}", self.label_counter - 2),
                        false_label: loop_end.clone(),
                    });
                    
                    let body_label = format!("for_body_{}", self.label_counter - 2);
                    self.emit_instruction(IrInstruction::Label { name: body_label });
                }
                
                // Generate body
                for stmt in body {
                    self.generate_stmt(stmt)?;
                }
                
                self.emit_instruction(IrInstruction::Label { name: loop_continue });
                
                // Generate update expression if present
                if let Some(update_expr) = update {
                    self.generate_expr(update_expr);
                }
                
                self.emit_instruction(IrInstruction::Jump { label: loop_start });
                
                self.emit_instruction(IrInstruction::Label { name: loop_end });
                
                self.loop_stack.pop();
            }
            
            Stmt::Break => {
                if let Some(loop_ctx) = self.loop_stack.last() {
                    self.emit_instruction(IrInstruction::Jump { 
                        label: loop_ctx.break_label.clone() 
                    });
                } else {
                    return Err(IrGeneratorError::UnsupportedConstruct("break statement outside of loop".to_string()));
                }
            }
            
            Stmt::Continue => {
                if let Some(loop_ctx) = self.loop_stack.last() {
                    self.emit_instruction(IrInstruction::Jump { 
                        label: loop_ctx.continue_label.clone() 
                    });
                } else {
                    return Err(IrGeneratorError::UnsupportedConstruct("continue statement outside of loop".to_string()));
                }
            }
        }
        Ok(())
    }

    /// Generate IR for an expression, returning the value
    fn generate_expr(&mut self, expr: &Expr) -> IrValue {
        match expr {
            Expr::Integer(i) => IrValue::IntConstant(*i),
            
            Expr::Float(f) => IrValue::FloatConstant(*f),
            
            Expr::Char(c) => IrValue::CharConstant(*c),
            
            Expr::String(s) => {
                let label = self.get_string_label(s);
                IrValue::StringConstant(label)
            }
            
            Expr::Identifier(name) => {
                // Load the variable value
                let temp = self.new_temp();
                let var_type = self.infer_identifier_type(name);
                
                self.emit_instruction(IrInstruction::Load {
                    dest: temp.clone(),
                    src: IrValue::Local(name.clone()),
                    var_type,
                });
                
                temp
            }
            
            Expr::Binary { left, operator, right } => {
                match operator {
                    TokenType::LogicalAnd => {
                        let result_temp = self.new_temp();
                        let false_label = self.new_label("and_false");
                        let end_label = self.new_label("and_end");
                        
                        // Evaluate left operand
                        let left_value = self.generate_expr(left);
                        
                        self.emit_instruction(IrInstruction::Branch {
                            condition: left_value,
                            true_label: format!("and_eval_right_{}", self.label_counter - 2),
                            false_label: false_label.clone(),
                        });
                        
                        // Evaluate right operand
                        let eval_right_label = format!("and_eval_right_{}", self.label_counter - 2);
                        self.emit_instruction(IrInstruction::Label { name: eval_right_label });
                        let right_value = self.generate_expr(right);
                        
                        self.emit_instruction(IrInstruction::Move {
                            dest: result_temp.clone(),
                            src: right_value,
                            var_type: IrType::Int,
                        });
                        self.emit_instruction(IrInstruction::Jump { label: end_label.clone() });
                        
                        self.emit_instruction(IrInstruction::Label { name: false_label });
                        self.emit_instruction(IrInstruction::Move {
                            dest: result_temp.clone(),
                            src: IrValue::IntConstant(0),
                            var_type: IrType::Int,
                        });
                        
                        // End label
                        self.emit_instruction(IrInstruction::Label { name: end_label });
                        
                        result_temp
                    }
                    
                    TokenType::LogicalOr => {
                        let result_temp = self.new_temp();
                        let true_label = self.new_label("or_true");
                        let end_label = self.new_label("or_end");
                        
                        // Evaluate left operand
                        let left_value = self.generate_expr(left);
                        
                        self.emit_instruction(IrInstruction::Branch {
                            condition: left_value,
                            true_label: true_label.clone(),
                            false_label: format!("or_eval_right_{}", self.label_counter - 2),
                        });
                        
                        // Evaluate right operand
                        let eval_right_label = format!("or_eval_right_{}", self.label_counter - 2);
                        self.emit_instruction(IrInstruction::Label { name: eval_right_label });
                        let right_value = self.generate_expr(right);
                        
                        self.emit_instruction(IrInstruction::Move {
                            dest: result_temp.clone(),
                            src: right_value,
                            var_type: IrType::Int,
                        });
                        self.emit_instruction(IrInstruction::Jump { label: end_label.clone() });
                        
                        self.emit_instruction(IrInstruction::Label { name: true_label });
                        self.emit_instruction(IrInstruction::Move {
                            dest: result_temp.clone(),
                            src: IrValue::IntConstant(1),
                            var_type: IrType::Int,
                        });
                        
                        // End label
                        self.emit_instruction(IrInstruction::Label { name: end_label });
                        
                        result_temp
                    }
                    
                    _ => {
                        let left_value = self.generate_expr(left);
                        let right_value = self.generate_expr(right);
                        let result_temp = self.new_temp();
                        let op = IrBinaryOp::from(operator.clone());
                        let expr_type = self.infer_expr_type(expr);
                        
                        self.emit_instruction(IrInstruction::BinaryOp {
                            dest: result_temp.clone(),
                            op,
                            left: left_value,
                            right: right_value,
                            var_type: expr_type,
                        });
                        
                        result_temp
                    }
                }
            }
            
            Expr::Unary { operator, operand } => {
                let operand_value = self.generate_expr(operand);
                let result_temp = self.new_temp();
                let op = match operator {
                    TokenType::Minus => IrUnaryOp::Neg,
                    TokenType::LogicalNot => IrUnaryOp::Not,
                    _ => return IrValue::IntConstant(0), // Return default value for unsupported operators
                };
                let expr_type = self.infer_expr_type(expr);
                
                self.emit_instruction(IrInstruction::UnaryOp {
                    dest: result_temp.clone(),
                    op,
                    operand: operand_value,
                    var_type: expr_type,
                });
                
                result_temp
            }
            
            Expr::Call { callee, arguments, .. } => {
                let func_name = match callee.as_ref() {
                    Expr::Identifier(name) => name.clone(),
                    _ => return IrValue::IntConstant(0), // Return default value for complex function calls
                };
                
                let mut arg_values = Vec::new();
                for arg in arguments {
                    arg_values.push(self.generate_expr(arg));
                }
                
                let result_temp = self.new_temp();
                let return_type = match func_name.as_str() {
                    "printf" | "println" => IrType::Int, // printf returns int (number of chars printed)
                    "main" => IrType::Int, // main function returns int
                    _ => IrType::Int, // Default fallback for unknown functions
                };
                
                self.emit_instruction(IrInstruction::Call {
                    dest: Some(result_temp.clone()),
                    func: func_name,
                    args: arg_values,
                    return_type,
                });
                
                result_temp
            }
            
            Expr::Assignment { name, value } => {
                let value_result = self.generate_expr(value);
                let var_type = self.infer_identifier_type(name);
                
                self.emit_instruction(IrInstruction::Store {
                    value: value_result.clone(),
                    dest: IrValue::Local(name.clone()),
                    var_type,
                });
                
                value_result
            }
            
            Expr::TypeCast { expr, target_type } => {
                let expr_value = self.generate_expr(expr);
                let src_type = self.infer_expr_type(expr);
                let target_ir_type = if let Some(token_type) = target_type.to_token_type() {
                    IrType::from(token_type)
                } else {
                    IrType::Int // Default fallback
                };
                
                let temp = self.new_temp();
                self.emit_instruction(IrInstruction::Cast {
                    dest: temp.clone(),
                    src: expr_value,
                    dest_type: target_ir_type,
                    src_type,
                });
                
                temp
            }
        }
    }

    /// Infer the type of an expression (simplified type inference)
    fn infer_expr_type(&self, expr: &Expr) -> IrType {
        match expr {
            Expr::Integer(_) => IrType::Int,
            Expr::Float(_) => IrType::Float,
            Expr::Char(_) => IrType::Char,
            Expr::String(_) => IrType::String,
            Expr::Identifier(name) => self.infer_identifier_type(name),
            Expr::Binary { left, operator, .. } => {
                match operator {
                    TokenType::Equal | TokenType::NotEqual | 
                    TokenType::LessThan | TokenType::LessEqual |
                    TokenType::GreaterThan | TokenType::GreaterEqual => IrType::Int, // Boolean as int
                    _ => self.infer_expr_type(left), // Use left operand type
                }
            }
            Expr::Unary { operand, .. } => self.infer_expr_type(operand),
            Expr::Call { callee, .. } => {
                if let Expr::Identifier(func_name) = callee.as_ref() {
                    match func_name.as_str() {
                        "printf" | "println" => IrType::Int, // printf returns int
                        "main" => IrType::Int, // main function returns int
                        _ => IrType::Int, // Default fallback for unknown functions
                    }
                } else {
                    IrType::Int // Default fallback
                }
            }
            Expr::Assignment { name, .. } => self.infer_identifier_type(name),
            Expr::TypeCast { target_type, .. } => {
                if let Some(token_type) = target_type.to_token_type() {
                    IrType::from(token_type)
                } else {
                    IrType::Int
                }
            }
        }
    }

    /// Collect variable types from AST for symbol table
    fn collect_variable_types(&mut self, ast: &[Stmt]) {
        for stmt in ast {
            match stmt {
                Stmt::Function { body, .. } => {
                    self.collect_variable_types(body);
                }
                Stmt::VarDecl { var_type, name, .. } => {
                    // Store variable type for later use
                    let ir_type = if let Some(token_type) = var_type.to_token_type() {
                    IrType::from(token_type)
                } else {
                    IrType::Int // Default fallback
                };
                    self.local_types.insert(name.clone(), ir_type);
                }
                Stmt::If { then_branch, .. } => {
                    self.collect_variable_types(then_branch);
                }
                Stmt::Block(stmts) => {
                    self.collect_variable_types(stmts);
                }
                _ => {}
            }
        }
    }

    /// Infer the type of an identifier using symbol table lookup
    fn infer_identifier_type(&self, name: &str) -> IrType {
        // Look up the variable type in the symbol table
        self.local_types.get(name)
            .cloned()
            .unwrap_or_else(|| {
                // Try to infer from context or use intelligent fallback
                if name.contains("float") || name.contains("f") {
                    IrType::Float
                } else if name.contains("char") || name.contains("c") {
                    IrType::Char
                } else if name.contains("str") || name.contains("string") {
                    IrType::String
                } else {
                    IrType::Int // Default fallback
                }
            })
    }
}

impl Default for IrGenerator {
    fn default() -> Self {
        Self::new()
    }
}
