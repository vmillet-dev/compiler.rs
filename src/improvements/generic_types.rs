
use std::collections::HashMap;

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
    Integer(IntegerType),
    Float(FloatType),
    Boolean,
    Character,
    Void,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IntegerType {
    pub signed: bool,
    pub width: u8, // 8, 16, 32, 64 bits
}

#[derive(Debug, Clone, PartialEq)]
pub struct FloatType {
    pub precision: FloatPrecision,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FloatPrecision {
    Single,  // 32-bit
    Double,  // 64-bit
    Extended, // 80-bit or higher
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
    pub name: Option<String>,
    pub fields: Vec<FieldType>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnionType {
    pub name: Option<String>,
    pub variants: Vec<FieldType>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumType {
    pub name: Option<String>,
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldType {
    pub name: String,
    pub field_type: Type,
    pub offset: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariant {
    pub name: String,
    pub value: Option<i64>,
}

pub struct TypeChecker {
    type_environment: HashMap<String, Type>,
    generic_constraints: HashMap<String, Vec<TypeConstraint>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeConstraint {
    Implements(String), // Trait/interface name
    SizeAtLeast(usize),
    SizeAtMost(usize),
    Numeric,
    Comparable,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            type_environment: HashMap::new(),
            generic_constraints: HashMap::new(),
        }
    }

    pub fn check_type_compatibility(&self, expected: &Type, actual: &Type) -> bool {
        match (&expected.kind, &actual.kind) {
            (TypeKind::Primitive(p1), TypeKind::Primitive(p2)) => {
                self.check_primitive_compatibility(p1, p2)
            }
            (TypeKind::Pointer(t1), TypeKind::Pointer(t2)) => {
                self.check_type_compatibility(t1, t2)
            }
            (TypeKind::Generic(name), _) => {
                self.check_generic_constraint(name, actual)
            }
            _ => expected == actual,
        }
    }

    fn check_primitive_compatibility(&self, p1: &PrimitiveType, p2: &PrimitiveType) -> bool {
        match (p1, p2) {
            (PrimitiveType::Integer(i1), PrimitiveType::Integer(i2)) => {
                i1.signed == i2.signed || i1.width >= i2.width
            }
            (PrimitiveType::Float(_), PrimitiveType::Integer(_)) => true, // int to float
            (PrimitiveType::Float(f1), PrimitiveType::Float(f2)) => {
                match (f1.precision, f2.precision) {
                    (FloatPrecision::Double, FloatPrecision::Single) => true,
                    (FloatPrecision::Extended, _) => true,
                    _ => f1 == f2,
                }
            }
            _ => p1 == p2,
        }
    }

    fn check_generic_constraint(&self, generic_name: &str, actual_type: &Type) -> bool {
        if let Some(constraints) = self.generic_constraints.get(generic_name) {
            constraints.iter().all(|constraint| {
                self.satisfies_constraint(actual_type, constraint)
            })
        } else {
            true // No constraints means any type is acceptable
        }
    }

    fn satisfies_constraint(&self, type_: &Type, constraint: &TypeConstraint) -> bool {
        match constraint {
            TypeConstraint::Numeric => matches!(
                type_.kind,
                TypeKind::Primitive(PrimitiveType::Integer(_)) |
                TypeKind::Primitive(PrimitiveType::Float(_))
            ),
            TypeConstraint::Comparable => {
                !matches!(type_.kind, TypeKind::Function(_))
            }
            TypeConstraint::SizeAtLeast(min_size) => {
                type_.size_hint.map_or(false, |size| size >= *min_size)
            }
            TypeConstraint::SizeAtMost(max_size) => {
                type_.size_hint.map_or(true, |size| size <= *max_size)
            }
            TypeConstraint::Implements(_trait_name) => {
                false
            }
        }
    }

    pub fn add_generic_constraint(&mut self, generic_name: String, constraint: TypeConstraint) {
        self.generic_constraints
            .entry(generic_name)
            .or_insert_with(Vec::new)
            .push(constraint);
    }
}

impl Default for TypeQualifiers {
    fn default() -> Self {
        Self {
            is_const: false,
            is_volatile: false,
            is_restrict: false,
        }
    }
}

impl Type {
    pub fn int32() -> Self {
        Self {
            kind: TypeKind::Primitive(PrimitiveType::Integer(IntegerType {
                signed: true,
                width: 32,
            })),
            qualifiers: TypeQualifiers::default(),
            size_hint: Some(4),
        }
    }

    pub fn float64() -> Self {
        Self {
            kind: TypeKind::Primitive(PrimitiveType::Float(FloatType {
                precision: FloatPrecision::Double,
            })),
            qualifiers: TypeQualifiers::default(),
            size_hint: Some(8),
        }
    }

    pub fn char_type() -> Self {
        Self {
            kind: TypeKind::Primitive(PrimitiveType::Character),
            qualifiers: TypeQualifiers::default(),
            size_hint: Some(1),
        }
    }

    pub fn void_type() -> Self {
        Self {
            kind: TypeKind::Primitive(PrimitiveType::Void),
            qualifiers: TypeQualifiers::default(),
            size_hint: Some(0),
        }
    }

    pub fn pointer_to(target: Type) -> Self {
        Self {
            kind: TypeKind::Pointer(Box::new(target)),
            qualifiers: TypeQualifiers::default(),
            size_hint: Some(8), // 64-bit pointer
        }
    }

    pub fn is_numeric(&self) -> bool {
        matches!(
            self.kind,
            TypeKind::Primitive(PrimitiveType::Integer(_)) |
            TypeKind::Primitive(PrimitiveType::Float(_))
        )
    }

    pub fn is_integral(&self) -> bool {
        matches!(self.kind, TypeKind::Primitive(PrimitiveType::Integer(_)))
    }

    pub fn is_floating_point(&self) -> bool {
        matches!(self.kind, TypeKind::Primitive(PrimitiveType::Float(_)))
    }
}
