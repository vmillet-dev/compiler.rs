use std::collections::HashMap;
use crate::lexer::TokenType;
use crate::parser::ast::{Expr, Stmt};

pub trait AstAnalyzer {
    fn collect_variable_types(&mut self, ast: &[Stmt]);
    fn collect_format_strings(&mut self, ast: &[Stmt]);
    fn get_local_types(&self) -> &HashMap<String, TokenType>;
    fn get_data_strings(&self) -> &HashMap<String, String>;
    fn get_data_strings_mut(&mut self) -> &mut HashMap<String, String>;
    fn new_string_label(&mut self) -> String;
}

impl AstAnalyzer for super::Codegen {
    fn collect_variable_types(&mut self, ast: &[Stmt]) {
        for stmt in ast {
            match stmt {
                Stmt::Function { body, .. } => {
                    self.collect_variable_types(body);
                }
                Stmt::VarDecl { var_type, name, .. } => {
                    // Store variable type for later use
                    self.local_types.insert(name.clone(), var_type.clone());
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

    fn collect_format_strings(&mut self, ast: &[Stmt]) {
        for stmt in ast {
            match stmt {
                Stmt::Function { body, .. } => {
                    self.collect_format_strings(body);
                }
                Stmt::PrintStmt { format_string, args } => {
                    if let Expr::String(s) = format_string {
                        if s.is_empty() {
                            // Simple println(expr) case - need to create format string
                            if args.len() == 1 {
                                let arg = &args[0];
                                let format_str = match arg {
                                    Expr::Integer(_) => "%d\n",
                                    Expr::Float(_) => "%.6f\n",
                                    Expr::Char(_) => "%c\n",
                                    Expr::Identifier(var_name) => {
                                        // Use stored type information
                                        match self.local_types.get(var_name) {
                                            Some(TokenType::Int) => "%d\n",
                                            Some(TokenType::FloatType) => "%.6f\n",
                                            Some(TokenType::CharType) => "%c\n",
                                            _ => "%d\n", // Default to integer
                                        }
                                    },
                                    _ => "%d\n", // Default to integer
                                };
                                
                                if !self.data_strings.contains_key(format_str) {
                                    let label = self.new_string_label();
                                    self.data_strings.insert(format_str.to_string(), label);
                                }
                            }
                        } else {
                            // Regular format string case
                            if !self.data_strings.contains_key(s) {
                                let label = self.new_string_label();
                                self.data_strings.insert(s.clone(), label);
                            }
                        }
                    }
                }
                Stmt::If { then_branch, .. } => {
                    self.collect_format_strings(then_branch);
                }
                Stmt::Block(stmts) => {
                    self.collect_format_strings(stmts);
                }
                _ => {}
            }
        }
    }

    fn get_local_types(&self) -> &HashMap<String, TokenType> {
        &self.local_types
    }

    fn get_data_strings(&self) -> &HashMap<String, String> {
        &self.data_strings
    }

    fn get_data_strings_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.data_strings
    }

    fn new_string_label(&mut self) -> String {
        let label = format!("str_{}", self.string_label_count);
        self.string_label_count += 1;
        label
    }
}