use crate::types::{Type, TargetTypeConfig};
use crate::semantic::symbol_table::SymbolTable;
use crate::semantic::lifetime_simple::{LifetimeAnalyzer, Lifetime};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum AllocationStrategy {
    Stack,
    Heap,
    Register,
    Static,
}

#[derive(Debug, Clone)]
pub struct MemoryLayout {
    pub strategy: AllocationStrategy,
    pub offset: i32,
    pub size: usize,
    pub alignment: usize,
    pub lifetime: Option<Lifetime>,
}

impl MemoryLayout {
    pub fn new(strategy: AllocationStrategy, offset: i32, size: usize, alignment: usize) -> Self {
        Self {
            strategy,
            offset,
            size,
            alignment,
            lifetime: None,
        }
    }
    
    pub fn with_lifetime(mut self, lifetime: Lifetime) -> Self {
        self.lifetime = Some(lifetime);
        self
    }
    
    pub fn is_aligned(&self, address: usize) -> bool {
        address % self.alignment == 0
    }
    
    pub fn aligned_offset(&self, base_offset: i32) -> i32 {
        let alignment = self.alignment as i32;
        let misalignment = base_offset % alignment;
        if misalignment == 0 {
            base_offset
        } else {
            base_offset + (alignment - misalignment)
        }
    }
}

pub struct StackFrameManager {
    current_offset: i32,
    max_offset: i32,
    target_config: TargetTypeConfig,
    variable_layouts: HashMap<String, MemoryLayout>,
    scope_stack: Vec<i32>, // Track offset at each scope entry
}

impl StackFrameManager {
    pub fn new(target_config: TargetTypeConfig) -> Self {
        Self {
            current_offset: 0,
            max_offset: 0,
            target_config,
            variable_layouts: HashMap::new(),
            scope_stack: vec![0],
        }
    }
    
    pub fn new_with_default_alignment(_alignment: usize) -> Self {
        Self::new(TargetTypeConfig::x86_64())
    }
    
    pub fn allocate_variable(&mut self, name: String, var_type: &Type) -> MemoryLayout {
        let size = var_type.size_with_config(&self.target_config);
        let alignment = var_type.alignment_with_config(&self.target_config);
        
        self.current_offset = self.align_offset(self.current_offset, alignment);
        self.current_offset -= size as i32; // Stack grows downward
        
        let layout = MemoryLayout::new(
            AllocationStrategy::Stack,
            self.current_offset,
            size,
            alignment,
        );
        
        self.variable_layouts.insert(name, layout.clone());
        
        if self.current_offset.abs() > self.max_offset.abs() {
            self.max_offset = self.current_offset;
        }
        
        layout
    }
    
    pub fn enter_scope(&mut self) {
        self.scope_stack.push(self.current_offset);
    }
    
    pub fn exit_scope(&mut self) -> Result<Vec<String>, String> {
        if self.scope_stack.len() <= 1 {
            return Err("Cannot exit global scope".to_string());
        }
        
        let scope_start_offset = self.scope_stack.pop()
            .ok_or_else(|| "Scope stack is empty".to_string())?;
        let mut deallocated_vars = Vec::new();
        
        self.variable_layouts.retain(|name, layout| {
            if layout.offset < scope_start_offset {
                deallocated_vars.push(name.clone());
                false
            } else {
                true
            }
        });
        
        self.current_offset = scope_start_offset;
        
        Ok(deallocated_vars)
    }
    
    pub fn get_layout(&self, name: &str) -> Option<&MemoryLayout> {
        self.variable_layouts.get(name)
    }
    
    pub fn frame_size(&self) -> usize {
        self.max_offset.abs() as usize
    }
    
    pub fn target_config(&self) -> &TargetTypeConfig {
        &self.target_config
    }
    
    fn align_offset(&self, offset: i32, alignment: usize) -> i32 {
        let alignment = alignment as i32;
        let misalignment = offset % alignment;
        if misalignment == 0 {
            offset
        } else {
            offset - misalignment
        }
    }
    
    pub fn reset(&mut self) {
        self.current_offset = 0;
        self.max_offset = 0;
        self.variable_layouts.clear();
        self.scope_stack.clear();
        self.scope_stack.push(0);
    }
    
    pub fn current_scope_variables(&self) -> Vec<&String> {
        let scope_start = *self.scope_stack.last().unwrap_or(&0);
        self.variable_layouts
            .iter()
            .filter(|(_, layout)| layout.offset >= scope_start)
            .map(|(name, _)| name)
            .collect()
    }
}

pub struct MemorySafetyChecker {
    lifetime_analyzer: LifetimeAnalyzer,
    stack_manager: StackFrameManager,
    _symbol_table: SymbolTable<MemoryLayout>,
}

impl MemorySafetyChecker {
    pub fn new() -> Self {
        Self::new_with_target_config(TargetTypeConfig::x86_64())
    }
    
    pub fn new_with_target_config(target_config: TargetTypeConfig) -> Self {
        Self {
            lifetime_analyzer: LifetimeAnalyzer::new(),
            stack_manager: StackFrameManager::new(target_config),
            _symbol_table: SymbolTable::new(),
        }
    }
    
    pub fn check_memory_safety(&mut self, statements: &[crate::parser::ast::Stmt]) -> Result<Vec<MemorySafetyWarning>, String> {
        let mut warnings = Vec::new();
        
        self.lifetime_analyzer.analyze_statements(statements)?;
        self.lifetime_analyzer.generate_lifetimes();
        
        warnings.extend(self.check_use_after_free()?);
        warnings.extend(self.check_double_free()?);
        warnings.extend(self.check_memory_leaks()?);
        warnings.extend(self.check_stack_overflow()?);
        
        Ok(warnings)
    }
    
    fn check_use_after_free(&self) -> Result<Vec<MemorySafetyWarning>, String> {
        let mut warnings = Vec::new();
        
        for (name, usage) in self.lifetime_analyzer.get_variable_usages() {
            let lifetime = usage.lifetime();
            
            for &usage_line in &usage.usage_lines {
                if usage_line > lifetime.end_line {
                    warnings.push(MemorySafetyWarning::UseAfterFree {
                        variable: name.clone(),
                        usage_line,
                        freed_line: lifetime.end_line,
                    });
                }
            }
        }
        
        Ok(warnings)
    }
    
    fn check_double_free(&self) -> Result<Vec<MemorySafetyWarning>, String> {
        Ok(Vec::new())
    }
    
    fn check_memory_leaks(&self) -> Result<Vec<MemorySafetyWarning>, String> {
        let mut warnings = Vec::new();
        
        for (name, usage) in self.lifetime_analyzer.get_variable_usages() {
            if usage.usage_lines.len() == 1 {
                warnings.push(MemorySafetyWarning::PotentialLeak {
                    variable: name.clone(),
                    allocation_line: usage.first_use,
                });
            }
        }
        
        Ok(warnings)
    }
    
    fn check_stack_overflow(&self) -> Result<Vec<MemorySafetyWarning>, String> {
        let mut warnings = Vec::new();
        
        const MAX_STACK_SIZE: usize = 1024 * 1024; // 1MB stack limit
        
        if self.stack_manager.frame_size() > MAX_STACK_SIZE {
            warnings.push(MemorySafetyWarning::StackOverflow {
                frame_size: self.stack_manager.frame_size(),
                limit: MAX_STACK_SIZE,
            });
        }
        
        Ok(warnings)
    }
    
    pub fn stack_manager(&self) -> &StackFrameManager {
        &self.stack_manager
    }
    
    pub fn stack_manager_mut(&mut self) -> &mut StackFrameManager {
        &mut self.stack_manager
    }
    
    pub fn lifetime_analyzer(&self) -> &LifetimeAnalyzer {
        &self.lifetime_analyzer
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MemorySafetyWarning {
    UseAfterFree {
        variable: String,
        usage_line: usize,
        freed_line: usize,
    },
    DoubleFree {
        variable: String,
        first_free: usize,
        second_free: usize,
    },
    PotentialLeak {
        variable: String,
        allocation_line: usize,
    },
    StackOverflow {
        frame_size: usize,
        limit: usize,
    },
    UnalignedAccess {
        variable: String,
        expected_alignment: usize,
        actual_alignment: usize,
    },
}

impl MemorySafetyWarning {
    pub fn severity(&self) -> MemorySafetySeverity {
        match self {
            MemorySafetyWarning::UseAfterFree { .. } => MemorySafetySeverity::Error,
            MemorySafetyWarning::DoubleFree { .. } => MemorySafetySeverity::Error,
            MemorySafetyWarning::StackOverflow { .. } => MemorySafetySeverity::Error,
            MemorySafetyWarning::PotentialLeak { .. } => MemorySafetySeverity::Warning,
            MemorySafetyWarning::UnalignedAccess { .. } => MemorySafetySeverity::Warning,
        }
    }
    
    pub fn message(&self) -> String {
        match self {
            MemorySafetyWarning::UseAfterFree { variable, usage_line, freed_line } => {
                format!("Variable '{}' used at line {} after being freed at line {}", variable, usage_line, freed_line)
            }
            MemorySafetyWarning::DoubleFree { variable, first_free, second_free } => {
                format!("Variable '{}' freed twice: first at line {}, then at line {}", variable, first_free, second_free)
            }
            MemorySafetyWarning::PotentialLeak { variable, allocation_line } => {
                format!("Variable '{}' allocated at line {} may not be properly freed", variable, allocation_line)
            }
            MemorySafetyWarning::StackOverflow { frame_size, limit } => {
                format!("Stack frame size {} bytes exceeds limit of {} bytes", frame_size, limit)
            }
            MemorySafetyWarning::UnalignedAccess { variable, expected_alignment, actual_alignment } => {
                format!("Variable '{}' has misaligned access: expected {}-byte alignment, got {}", variable, expected_alignment, actual_alignment)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MemorySafetySeverity {
    Error,
    Warning,
    Info,
}

impl Default for MemorySafetyChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Type, PrimitiveType};

    #[test]
    fn test_stack_frame_allocation() {
        let mut manager = StackFrameManager::new(TargetTypeConfig::x86_64());
        
        let int_type = Type::primitive(PrimitiveType::Int32);
        let layout1 = manager.allocate_variable("x".to_string(), &int_type);
        
        assert_eq!(layout1.strategy, AllocationStrategy::Stack);
        assert_eq!(layout1.size, 4);
        assert_eq!(layout1.alignment, 4);
        
        let layout2 = manager.allocate_variable("y".to_string(), &int_type);
        assert!(layout2.offset < layout1.offset); // Stack grows downward
    }
    
    #[test]
    fn test_scope_management() {
        let mut manager = StackFrameManager::new(TargetTypeConfig::x86_64());
        
        let int_type = Type::primitive(PrimitiveType::Int32);
        manager.allocate_variable("global".to_string(), &int_type);
        
        manager.enter_scope();
        manager.allocate_variable("local".to_string(), &int_type);
        
        assert!(manager.get_layout("global").is_some());
        assert!(manager.get_layout("local").is_some());
        
        let deallocated = manager.exit_scope().unwrap();
        assert_eq!(deallocated.len(), 1);
        assert_eq!(deallocated[0], "local");
        
        assert!(manager.get_layout("global").is_some());
        assert!(manager.get_layout("local").is_none());
    }
    
    #[test]
    fn test_memory_alignment() {
        let mut manager = StackFrameManager::new(TargetTypeConfig::x86_64());
        
        let char_type = Type::primitive(PrimitiveType::Char);
        let int_type = Type::primitive(PrimitiveType::Int32);
        let double_type = Type::primitive(PrimitiveType::Float64);
        
        let char_layout = manager.allocate_variable("c".to_string(), &char_type);
        let int_layout = manager.allocate_variable("i".to_string(), &int_type);
        let double_layout = manager.allocate_variable("d".to_string(), &double_type);
        
        assert_eq!(char_layout.alignment, 1);
        assert_eq!(int_layout.alignment, 4);
        assert_eq!(double_layout.alignment, 8);
        
        assert_eq!(char_layout.offset % char_layout.alignment as i32, 0);
        assert_eq!(int_layout.offset % int_layout.alignment as i32, 0);
        assert_eq!(double_layout.offset % double_layout.alignment as i32, 0);
    }
}
