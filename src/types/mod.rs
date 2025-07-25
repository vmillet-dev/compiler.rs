use crate::lexer::TokenType;

pub mod target_config;
use target_config::TargetTypeConfig;

#[derive(Debug, Clone, PartialEq)]
pub struct Type {
    pub kind: TypeKind,
    pub qualifiers: TypeQualifiers,
    pub size_hint: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeKind {
    Primitive(PrimitiveType),
    Pointer(Box<Type>),
    Array(Box<Type>, usize),
    Function(FunctionType),
    Struct(StructType),
    Union(UnionType),
    Enum(EnumType),
    Generic(String), // For generic type parameters
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrimitiveType {
    Void,
    Bool,
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float32,
    Float64,
    Char,
    String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeQualifiers {
    pub is_const: bool,
    pub is_volatile: bool,
    pub is_restrict: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionType {
    pub return_type: Box<Type>,
    pub parameters: Vec<Type>,
    pub is_variadic: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructType {
    pub name: String,
    pub fields: Vec<(String, Type)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnionType {
    pub name: String,
    pub variants: Vec<(String, Type)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumType {
    pub name: String,
    pub variants: Vec<String>,
}

impl Type {
    pub fn primitive(prim: PrimitiveType) -> Self {
        Type {
            kind: TypeKind::Primitive(prim),
            qualifiers: TypeQualifiers::default(),
            size_hint: None,
        }
    }

    pub fn pointer(target: Type) -> Self {
        Type {
            kind: TypeKind::Pointer(Box::new(target)),
            qualifiers: TypeQualifiers::default(),
            size_hint: None, // Let target config determine pointer size
        }
    }

    pub fn array(element: Type, size: usize) -> Self {
        Type {
            kind: TypeKind::Array(Box::new(element), size),
            qualifiers: TypeQualifiers::default(),
            size_hint: None,
        }
    }
    
    pub fn generic(name: String) -> Self {
        Type {
            kind: TypeKind::Generic(name),
            qualifiers: TypeQualifiers::default(),
            size_hint: None,
        }
    }
    
    pub fn function(return_type: Type, parameters: Vec<Type>, is_variadic: bool) -> Self {
        Type {
            kind: TypeKind::Function(FunctionType {
                return_type: Box::new(return_type),
                parameters,
                is_variadic,
            }),
            qualifiers: TypeQualifiers::default(),
            size_hint: None,
        }
    }

    pub fn is_compatible_with(&self, other: &Type) -> bool {
        self.is_compatible_with_substitutions(other, &std::collections::HashMap::new())
    }
    
    pub fn is_compatible_with_substitutions(&self, other: &Type, substitutions: &std::collections::HashMap<String, Type>) -> bool {
        match (&self.kind, &other.kind) {
            (TypeKind::Primitive(a), TypeKind::Primitive(b)) => a == b,
            (TypeKind::Pointer(a), TypeKind::Pointer(b)) => a.is_compatible_with_substitutions(b, substitutions),
            (TypeKind::Array(a, size_a), TypeKind::Array(b, size_b)) => {
                size_a == size_b && a.is_compatible_with_substitutions(b, substitutions)
            }
            (TypeKind::Function(a), TypeKind::Function(b)) => {
                a.return_type.is_compatible_with_substitutions(&b.return_type, substitutions) &&
                a.parameters.len() == b.parameters.len() &&
                a.parameters.iter().zip(&b.parameters).all(|(p1, p2)| p1.is_compatible_with_substitutions(p2, substitutions)) &&
                a.is_variadic == b.is_variadic
            }
            (TypeKind::Generic(name), _) => {
                if let Some(substituted) = substitutions.get(name) {
                    substituted.is_compatible_with_substitutions(other, substitutions)
                } else {
                    true // Generic types are compatible with anything if not constrained
                }
            }
            (_, TypeKind::Generic(name)) => {
                if let Some(substituted) = substitutions.get(name) {
                    self.is_compatible_with_substitutions(substituted, substitutions)
                } else {
                    true // Generic types are compatible with anything if not constrained
                }
            }
            _ => false,
        }
    }
    
    pub fn is_generic(&self) -> bool {
        match &self.kind {
            TypeKind::Generic(_) => true,
            TypeKind::Pointer(inner) => inner.is_generic(),
            TypeKind::Array(inner, _) => inner.is_generic(),
            TypeKind::Function(func) => {
                func.return_type.is_generic() || func.parameters.iter().any(|p| p.is_generic())
            }
            _ => false,
        }
    }

    pub fn to_token_type(&self) -> Option<TokenType> {
        match &self.kind {
            TypeKind::Primitive(PrimitiveType::Void) => Some(TokenType::Void),
            TypeKind::Primitive(PrimitiveType::Int32) => Some(TokenType::Int),
            TypeKind::Primitive(PrimitiveType::Float64) => Some(TokenType::FloatType),
            TypeKind::Primitive(PrimitiveType::Char) => Some(TokenType::CharType),
            _ => None,
        }
    }

    pub fn size(&self) -> usize {
        self.size_with_config(&TargetTypeConfig::default())
    }
    
    pub fn size_with_config(&self, config: &TargetTypeConfig) -> usize {
        if let Some(hint) = self.size_hint {
            return hint;
        }
        config.size_of(&self.kind)
    }
    
    pub fn alignment(&self) -> usize {
        self.alignment_with_config(&TargetTypeConfig::default())
    }
    
    pub fn alignment_with_config(&self, config: &TargetTypeConfig) -> usize {
        config.alignment_of(&self.kind)
    }
}

impl Default for TypeQualifiers {
    fn default() -> Self {
        TypeQualifiers {
            is_const: false,
            is_volatile: false,
            is_restrict: false,
        }
    }
}

impl From<TokenType> for Type {
    fn from(token_type: TokenType) -> Self {
        match token_type {
            TokenType::Void => Type::primitive(PrimitiveType::Void),
            TokenType::Int => Type::primitive(PrimitiveType::Int32),
            TokenType::FloatType => Type::primitive(PrimitiveType::Float64),
            TokenType::CharType => Type::primitive(PrimitiveType::Char),
            _ => Type::primitive(PrimitiveType::Int32), // Default fallback
        }
    }
}

pub struct TypeChecker {
    pub constraints: std::collections::HashMap<String, Vec<TypeConstraint>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeConstraint {
    Trait(String),
    Subtype(Type),
    Size(usize),
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            constraints: std::collections::HashMap::new(),
        }
    }

    pub fn add_constraint(&mut self, type_param: String, constraint: TypeConstraint) {
        self.constraints.entry(type_param).or_insert_with(Vec::new).push(constraint);
    }

    pub fn check_constraints(&self, type_param: &str, concrete_type: &Type) -> bool {
        if let Some(constraints) = self.constraints.get(type_param) {
            for constraint in constraints {
                if !self.satisfies_constraint(concrete_type, constraint) {
                    return false;
                }
            }
        }
        true
    }

    fn satisfies_constraint(&self, concrete_type: &Type, constraint: &TypeConstraint) -> bool {
        match constraint {
            TypeConstraint::Size(expected_size) => concrete_type.size() == *expected_size,
            TypeConstraint::Subtype(parent) => concrete_type.is_compatible_with(parent),
            TypeConstraint::Trait(_) => true, // Simplified for now
        }
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}
