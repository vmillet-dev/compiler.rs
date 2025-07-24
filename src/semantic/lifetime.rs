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
            Stmt::Assignment { name, value } => {
                self.analyze_assignment(name, value)?;
            }
            Stmt::If { condition, then_branch, else_branch } => {
                self.analyze_expression(condition)?;
                self.analyze_statement(then_branch)?;
                if let Some(else_stmt) = else_branch {
                    self.analyze_statement(else_stmt)?;
                }
            }
            Stmt::Return { value } => {
                if let Some(expr) = value {
                    self.analyze_expression(expr)?;
                }
            }
            Stmt::Expression { expr } => {
                self.analyze_expression(expr)?;
            }
            Stmt::Function { return_type: _, name: _, body } => {
                for body_stmt in body {
                    self.analyze_statement(body_stmt)?;
                }
            }
            Stmt::Printf { format_str: _, args } => {
                for arg in args {
                    self.analyze_expression(arg)?;
                }
            }
            Stmt::Println { expr } => {
                if let Some(e) = expr {
                    self.analyze_expression(e)?;
                }
            }
        }
        self.current_line += 1;
        Ok(())
    }
    
    pub fn analyze_expression(&mut self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::Variable(name) => {
                self.record_variable_usage(name)?;
            }
            Expr::Binary { left, right, .. } => {
                self.analyze_expression(left)?;
                self.analyze_expression(right)?;
            }
            Expr::Unary { operand, .. } => {
                self.analyze_expression(operand)?;
            }
            Expr::Call { name, args } => {
                self.record_variable_usage(name)?;
                for arg in args {
                    self.analyze_expression(arg)?;
                }
            }
            Expr::Integer(_) | Expr::Float(_) | Expr::String(_) | Expr::Char(_) | Expr::Boolean(_) => {
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
    
    fn analyze_assignment(&mut self, name: &str, value: &Expr) -> Result<(), String> {
        self.record_variable_usage(name)?;
        self.analyze_expression(value)?;
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
    
    pub fn find_overlapping_lifetimes(&self) -> Vec<(String, String)> {
        let mut overlapping = Vec::new();
        let lifetime_vec: Vec<_> = self.lifetimes.iter().collect();
        
        for i in 0..lifetime_vec.len() {
            for j in (i + 1)..lifetime_vec.len() {
                let (name1, lifetime1) = lifetime_vec[i];
                let (name2, lifetime2) = lifetime_vec[j];
                
                if lifetime1.overlaps_with(lifetime2) {
                    overlapping.push((name1.clone(), name2.clone()));
                }
            }
        }
        
        overlapping
    }
    
    pub fn suggest_register_allocation(&self) -> HashMap<String, usize> {
        let mut allocation = HashMap::new();
        let mut register_counter = 0;
        
        let mut sorted_vars: Vec<_> = self.variable_usages.iter().collect();
        sorted_vars.sort_by_key(|(_, usage)| usage.first_use);
        
        for (name, _) in sorted_vars {
            allocation.insert(name.clone(), register_counter);
            register_counter += 1;
        }
        
        allocation
    }
    
    pub fn check_memory_safety(&self) -> Vec<String> {
        let mut issues = Vec::new();
        
        for (name, usage) in &self.variable_usages {
            if usage.usage_lines.len() > 1 {
                let sorted_lines = {
                    let mut lines = usage.usage_lines.clone();
                    lines.sort();
                    lines
                };
                
                for window in sorted_lines.windows(2) {
                    if window[1] - window[0] > 10 {
                        issues.push(format!(
                            "Variable '{}' has large gap in usage (lines {} to {}), potential use-after-free risk",
                            name, window[0], window[1]
                        ));
                    }
                }
            }
        }
        
        issues
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Type, TypeKind};

    #[test]
    fn test_lifetime_overlap() {
        let lifetime1 = Lifetime::new(1, "x".to_string(), 1, 5);
        let lifetime2 = Lifetime::new(2, "y".to_string(), 3, 7);
        let lifetime3 = Lifetime::new(3, "z".to_string(), 6, 10);
        
        assert!(lifetime1.overlaps_with(&lifetime2));
        assert!(!lifetime1.overlaps_with(&lifetime3));
        assert!(lifetime2.overlaps_with(&lifetime3));
    }
    
    #[test]
    fn test_variable_usage() {
        let mut usage = VariableUsage::new(
            "x".to_string(),
            Type::new(TypeKind::Int, vec![], false),
            1,
            true,
        );
        
        usage.add_usage(3);
        usage.add_usage(5);
        usage.add_usage(3); // Duplicate should be ignored
        
        assert_eq!(usage.first_use, 1);
        assert_eq!(usage.last_use, 5);
        assert_eq!(usage.usage_lines.len(), 3);
    }
    
    #[test]
    fn test_lifetime_constraint_validation() {
        let lifetime1 = Lifetime::new(1, "x".to_string(), 1, 10);
        let lifetime2 = Lifetime::new(2, "y".to_string(), 3, 7);
        
        let constraint = LifetimeConstraint::Outlives(lifetime1.clone(), lifetime2.clone());
        assert!(constraint.is_satisfied());
        
        let invalid_constraint = LifetimeConstraint::Outlives(lifetime2, lifetime1);
        assert!(!invalid_constraint.is_satisfied());
    }
}
