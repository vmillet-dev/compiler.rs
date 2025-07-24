use crate::types::Type;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Public,
    Private,
    Protected,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Mutability {
    Mutable,
    Immutable,
}

#[derive(Debug, Clone)]
pub struct Symbol<T> {
    pub name: String,
    pub symbol_type: Type,
    pub value: T,
    pub visibility: Visibility,
    pub mutability: Mutability,
    pub scope_level: usize,
    pub line: usize,
    pub column: usize,
}

impl<T> Symbol<T> {
    pub fn new(
        name: String,
        symbol_type: Type,
        value: T,
        visibility: Visibility,
        mutability: Mutability,
        scope_level: usize,
        line: usize,
        column: usize,
    ) -> Self {
        Self {
            name,
            symbol_type,
            value,
            visibility,
            mutability,
            scope_level,
            line,
            column,
        }
    }
    
    pub fn is_accessible_from(&self, current_scope: usize) -> bool {
        match self.visibility {
            Visibility::Public => true,
            Visibility::Private => self.scope_level == current_scope,
            Visibility::Protected => self.scope_level <= current_scope,
        }
    }
    
    pub fn can_modify(&self) -> bool {
        self.mutability == Mutability::Mutable
    }
}

pub struct SymbolTable<T> {
    scopes: Vec<HashMap<String, Symbol<T>>>,
    current_scope: usize,
}

impl<T> SymbolTable<T> {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()], // Global scope
            current_scope: 0,
        }
    }
    
    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
        self.current_scope += 1;
    }
    
    pub fn exit_scope(&mut self) -> Result<(), String> {
        if self.current_scope == 0 {
            return Err("Cannot exit global scope".to_string());
        }
        
        self.scopes.pop();
        self.current_scope -= 1;
        Ok(())
    }
    
    pub fn insert(&mut self, symbol: Symbol<T>) -> Result<(), String> {
        let current_scope = &mut self.scopes[self.current_scope];
        
        if current_scope.contains_key(&symbol.name) {
            return Err(format!("Symbol '{}' already exists in current scope", symbol.name));
        }
        
        current_scope.insert(symbol.name.clone(), symbol);
        Ok(())
    }
    
    pub fn lookup(&self, name: &str) -> Option<&Symbol<T>> {
        for scope_level in (0..=self.current_scope).rev() {
            if let Some(symbol) = self.scopes[scope_level].get(name) {
                if symbol.is_accessible_from(self.current_scope) {
                    return Some(symbol);
                }
            }
        }
        None
    }
    
    pub fn lookup_mut(&mut self, name: &str) -> Option<&mut Symbol<T>> {
        let current_scope = self.current_scope;
        
        let mut target_scope = None;
        for scope_level in (0..=current_scope).rev() {
            if let Some(symbol) = self.scopes[scope_level].get(name) {
                if symbol.is_accessible_from(current_scope) {
                    target_scope = Some(scope_level);
                    break;
                }
            }
        }
        
        if let Some(scope_level) = target_scope {
            self.scopes[scope_level].get_mut(name)
        } else {
            None
        }
    }
    
    pub fn exists_in_current_scope(&self, name: &str) -> bool {
        self.scopes[self.current_scope].contains_key(name)
    }
    
    pub fn current_scope_symbols(&self) -> Vec<&Symbol<T>> {
        self.scopes[self.current_scope].values().collect()
    }
    
    pub fn accessible_symbols(&self) -> Vec<&Symbol<T>> {
        let mut symbols = Vec::new();
        
        for scope_level in 0..=self.current_scope {
            for symbol in self.scopes[scope_level].values() {
                if symbol.is_accessible_from(self.current_scope) {
                    symbols.push(symbol);
                }
            }
        }
        
        symbols
    }
    
    pub fn current_scope_level(&self) -> usize {
        self.current_scope
    }
    
    pub fn check_shadowing(&self, name: &str) -> Vec<&Symbol<T>> {
        let mut shadowed = Vec::new();
        
        for scope_level in 0..self.current_scope {
            if let Some(symbol) = self.scopes[scope_level].get(name) {
                shadowed.push(symbol);
            }
        }
        
        shadowed
    }
    
    pub fn remove(&mut self, name: &str) -> Option<Symbol<T>> {
        self.scopes[self.current_scope].remove(name)
    }
    
    pub fn clear_current_scope(&mut self) {
        self.scopes[self.current_scope].clear();
    }
    
    pub fn total_symbols(&self) -> usize {
        self.scopes.iter().map(|scope| scope.len()).sum()
    }
}

impl<T> Default for SymbolTable<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub type VariableSymbolTable = SymbolTable<i32>;

pub type FunctionSymbolTable = SymbolTable<FunctionInfo>;

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub parameters: Vec<(String, Type)>,
    pub return_type: Type,
    pub is_extern: bool,
    pub body_analyzed: bool,
}

impl FunctionInfo {
    pub fn new(parameters: Vec<(String, Type)>, return_type: Type, is_extern: bool) -> Self {
        Self {
            parameters,
            return_type,
            is_extern,
            body_analyzed: false,
        }
    }
    
    pub fn parameter_count(&self) -> usize {
        self.parameters.len()
    }
    
    pub fn parameter_type(&self, index: usize) -> Option<&Type> {
        self.parameters.get(index).map(|(_, t)| t)
    }
    
    pub fn parameter_name(&self, index: usize) -> Option<&str> {
        self.parameters.get(index).map(|(n, _)| n.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Type, PrimitiveType};

    #[test]
    fn test_symbol_table_basic_operations() {
        let mut table = SymbolTable::<i32>::new();
        
        let symbol = Symbol::new(
            "x".to_string(),
            Type::primitive(PrimitiveType::Int32),
            42,
            Visibility::Public,
            Mutability::Mutable,
            0,
            1,
            1,
        );
        
        assert!(table.insert(symbol).is_ok());
        assert!(table.lookup("x").is_some());
        assert!(table.lookup("y").is_none());
    }
    
    #[test]
    fn test_symbol_table_scoping() {
        let mut table = SymbolTable::<i32>::new();
        
        let global_symbol = Symbol::new(
            "global".to_string(),
            Type::primitive(PrimitiveType::Int32),
            1,
            Visibility::Public,
            Mutability::Mutable,
            0,
            1,
            1,
        );
        table.insert(global_symbol).unwrap();
        
        table.enter_scope();
        
        let local_symbol = Symbol::new(
            "local".to_string(),
            Type::primitive(PrimitiveType::Int32),
            2,
            Visibility::Private,
            Mutability::Mutable,
            1,
            2,
            1,
        );
        table.insert(local_symbol).unwrap();
        
        assert!(table.lookup("global").is_some());
        assert!(table.lookup("local").is_some());
        
        table.exit_scope().unwrap();
        
        assert!(table.lookup("global").is_some());
        assert!(table.lookup("local").is_none());
    }
    
    #[test]
    fn test_symbol_shadowing() {
        let mut table = SymbolTable::<i32>::new();
        
        let global_x = Symbol::new(
            "x".to_string(),
            Type::primitive(PrimitiveType::Int32),
            1,
            Visibility::Public,
            Mutability::Mutable,
            0,
            1,
            1,
        );
        table.insert(global_x).unwrap();
        
        table.enter_scope();
        let local_x = Symbol::new(
            "x".to_string(),
            Type::primitive(PrimitiveType::Int32),
            2,
            Visibility::Private,
            Mutability::Mutable,
            1,
            2,
            1,
        );
        table.insert(local_x).unwrap();
        
        let found = table.lookup("x").unwrap();
        assert_eq!(found.value, 2);
        
        let shadowed = table.check_shadowing("x");
        assert_eq!(shadowed.len(), 1);
        assert_eq!(shadowed[0].value, 1);
    }
}
