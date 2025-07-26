use crate::types::Type;
use crate::parser::ast::{Stmt, Expr};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Lifetime {
    pub id: usize,
    pub name: String,
    pub start_line: usize,
    pub end_line: usize,
}

impl Lifetime {
    pub fn new(id: usize, name: String, start_line: usize, end_line: usize) -> Self {
        Self {
            id,
            name,
            start_line,
            end_line,
        }
    }
    
    pub fn overlaps_with(&self, other: &Lifetime) -> bool {
        !(self.end_line < other.start_line || other.end_line < self.start_line)
    }
    
    pub fn contains_line(&self, line: usize) -> bool {
        line >= self.start_line && line <= self.end_line
    }
    
    pub fn duration(&self) -> usize {
        if self.end_line >= self.start_line {
            self.end_line - self.start_line + 1
        } else {
            0
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LifetimeConstraint {
    Outlives(Lifetime, Lifetime),
    Equal(Lifetime, Lifetime),
    MinDuration(Lifetime, usize),
}

impl LifetimeConstraint {
    pub fn is_satisfied(&self) -> bool {
        match self {
            LifetimeConstraint::Outlives(a, b) => {
                a.start_line <= b.start_line && a.end_line >= b.end_line
            }
            LifetimeConstraint::Equal(a, b) => {
                a.start_line == b.start_line && a.end_line == b.end_line
            }
            LifetimeConstraint::MinDuration(lifetime, min_duration) => {
                lifetime.duration() >= *min_duration
            }
        }
    }
    
    pub fn involves_lifetime(&self, lifetime_id: usize) -> bool {
        match self {
            LifetimeConstraint::Outlives(a, b) => a.id == lifetime_id || b.id == lifetime_id,
            LifetimeConstraint::Equal(a, b) => a.id == lifetime_id || b.id == lifetime_id,
            LifetimeConstraint::MinDuration(lifetime, _) => lifetime.id == lifetime_id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VariableUsage {
    pub name: String,
    pub var_type: Type,
    pub first_use: usize,
    pub last_use: usize,
    pub is_mutable: bool,
    pub usage_lines: Vec<usize>,
}

impl VariableUsage {
    pub fn new(name: String, var_type: Type, first_use: usize, is_mutable: bool) -> Self {
        Self {
            name,
            var_type,
            first_use,
            last_use: first_use,
            is_mutable,
            usage_lines: vec![first_use],
        }
    }
    
    pub fn add_usage(&mut self, line: usize) {
        if !self.usage_lines.contains(&line) {
            self.usage_lines.push(line);
            if line > self.last_use {
                self.last_use = line;
            }
        }
    }
    
    pub fn lifetime(&self) -> Lifetime {
        Lifetime::new(
            self.name.as_ptr() as usize, // Simple ID generation
            self.name.clone(),
            self.first_use,
            self.last_use,
        )
    }
}

pub struct LifetimeAnalyzer {
    lifetimes: HashMap<String, Lifetime>,
    constraints: Vec<LifetimeConstraint>,
    variable_usages: HashMap<String, VariableUsage>,
    next_lifetime_id: usize,
    current_line: usize,
}

impl LifetimeAnalyzer {
    pub fn new() -> Self {
        Self {
            lifetimes: HashMap::new(),
            constraints: Vec::new(),
            variable_usages: HashMap::new(),
            next_lifetime_id: 0,
            current_line: 1,
        }
    }
    
    pub fn analyze_statements(&mut self, statements: &[Stmt]) -> Result<(), String> {
        for stmt in statements {
            self.analyze_statement(stmt)?;
        }
        self.validate_constraints()
    }
    
    pub fn analyze_statement(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::VarDecl { var_type, name, initializer } => {
                self.analyze_variable_declaration(name, var_type.clone(), initializer.as_ref())?;
            }
            Stmt::ExprStmt(expr) => {
                self.analyze_expression(expr)?;
            }
            Stmt::If { condition, then_branch } => {
                self.analyze_expression(condition)?;
                for stmt in then_branch {
                    self.analyze_statement(stmt)?;
                }
            }
            Stmt::Return(value) => {
                if let Some(expr) = value {
                    self.analyze_expression(expr)?;
                }
            }
            Stmt::Block(statements) => {
                for stmt in statements {
                    self.analyze_statement(stmt)?;
                }
            }
            Stmt::Function { return_type: _, name: _, body, .. } => {
                for body_stmt in body {
                    self.analyze_statement(body_stmt)?;
                }
            }
            Stmt::PrintStmt { format_string, args } => {
                self.analyze_expression(format_string)?;
                for arg in args {
                    self.analyze_expression(arg)?;
                }
            }
            Stmt::While { condition, body } => {
                self.analyze_expression(condition)?;
                for stmt in body {
                    self.analyze_statement(stmt)?;
                }
            }
            Stmt::For { init, condition, update, body } => {
                if let Some(init_stmt) = init {
                    self.analyze_statement(init_stmt)?;
                }
                if let Some(cond_expr) = condition {
                    self.analyze_expression(cond_expr)?;
                }
                if let Some(update_expr) = update {
                    self.analyze_expression(update_expr)?;
                }
                for stmt in body {
                    self.analyze_statement(stmt)?;
                }
            }
            Stmt::Break | Stmt::Continue => {
            }
        }
        self.current_line += 1;
        Ok(())
    }
    
    pub fn analyze_expression(&mut self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::Identifier(name) => {
                self.record_variable_usage(name)?;
            }
            Expr::Binary { left, right, .. } => {
                self.analyze_expression(left)?;
                self.analyze_expression(right)?;
            }
            Expr::Unary { operand, .. } => {
                self.analyze_expression(operand)?;
            }
            Expr::Call { callee, arguments, .. } => {
                self.analyze_expression(callee)?;
                for arg in arguments {
                    self.analyze_expression(arg)?;
                }
            }
            Expr::Assignment { name, value } => {
                self.record_variable_usage(name)?;
                self.analyze_expression(value)?;
            }
            Expr::Integer(_) | Expr::Float(_) | Expr::String(_) | Expr::Char(_) => {
            }
            Expr::TypeCast { expr, .. } => {
                self.analyze_expression(expr)?;
            }
        }
        Ok(())
    }
    
    fn analyze_variable_declaration(
        &mut self,
        name: &str,
        var_type: Type,
        initializer: Option<&Expr>,
    ) -> Result<(), String> {
        let usage = VariableUsage::new(
            name.to_string(),
            var_type,
            self.current_line,
            true, // Assume mutable for now
        );
        
        self.variable_usages.insert(name.to_string(), usage);
        
        if let Some(init_expr) = initializer {
            self.analyze_expression(init_expr)?;
        }
        
        Ok(())
    }
    
    fn record_variable_usage(&mut self, name: &str) -> Result<(), String> {
        if let Some(usage) = self.variable_usages.get_mut(name) {
            usage.add_usage(self.current_line);
        } else {
            return Err(format!("Variable '{}' used before declaration at line {}", name, self.current_line));
        }
        Ok(())
    }
    
    pub fn generate_lifetimes(&mut self) {
        self.lifetimes.clear();
        
        for (name, usage) in &self.variable_usages {
            let lifetime = usage.lifetime();
            self.lifetimes.insert(name.clone(), lifetime);
        }
    }
    
    pub fn add_constraint(&mut self, constraint: LifetimeConstraint) {
        self.constraints.push(constraint);
    }
    
    pub fn validate_constraints(&self) -> Result<(), String> {
        for constraint in &self.constraints {
            if !constraint.is_satisfied() {
                return Err(format!("Lifetime constraint violated: {:?}", constraint));
            }
        }
        Ok(())
    }
    
    pub fn get_lifetime(&self, name: &str) -> Option<&Lifetime> {
        self.lifetimes.get(name)
    }
    
    pub fn get_lifetimes(&self) -> &HashMap<String, Lifetime> {
        &self.lifetimes
    }
    
    pub fn get_variable_usage(&self, name: &str) -> Option<&VariableUsage> {
        self.variable_usages.get(name)
    }
    
    pub fn get_variable_usages(&self) -> &HashMap<String, VariableUsage> {
        &self.variable_usages
    }
    
    pub fn reset(&mut self) {
        self.lifetimes.clear();
        self.constraints.clear();
        self.variable_usages.clear();
        self.next_lifetime_id = 0;
        self.current_line = 1;
    }
}

impl Default for LifetimeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
