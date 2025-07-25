use crate::types::{TypeKind, PrimitiveType};

#[derive(Debug, Clone, PartialEq)]
pub struct TargetTypeConfig {
    pub pointer_size: usize,
    pub default_alignment: usize,
    pub stack_alignment: usize,
}

impl TargetTypeConfig {
    pub fn x86_64() -> Self {
        Self {
            pointer_size: 8,
            default_alignment: 8,
            stack_alignment: 16,
        }
    }
    
    pub fn size_of(&self, type_kind: &TypeKind) -> usize {
        match type_kind {
            TypeKind::Primitive(prim) => match prim {
                PrimitiveType::Void => 0,
                PrimitiveType::Bool => 1,
                PrimitiveType::Int8 | PrimitiveType::UInt8 | PrimitiveType::Char => 1,
                PrimitiveType::Int16 | PrimitiveType::UInt16 => 2,
                PrimitiveType::Int32 | PrimitiveType::UInt32 | PrimitiveType::Float32 => 4,
                PrimitiveType::Int64 | PrimitiveType::UInt64 | PrimitiveType::Float64 => 8,
                PrimitiveType::String => self.pointer_size, // Pointer to string data
            },
            TypeKind::Pointer(_) => self.pointer_size,
            TypeKind::Array(element_type, count) => {
                self.size_of(&element_type.kind) * count
            }
            TypeKind::Function(_) => self.pointer_size, // Function pointer
            TypeKind::Struct(s) => {
                let mut total_size = 0;
                for (_, field_type) in &s.fields {
                    let field_size = self.size_of(&field_type.kind);
                    let field_alignment = self.alignment_of(&field_type.kind);
                    total_size = self.align_offset(total_size, field_alignment);
                    total_size += field_size;
                }
                self.align_offset(total_size, self.default_alignment)
            }
            TypeKind::Union(u) => {
                u.variants.iter()
                    .map(|(_, variant_type)| self.size_of(&variant_type.kind))
                    .max()
                    .unwrap_or(0)
            }
            TypeKind::Enum(_) => 4, // 32-bit enum by default
            TypeKind::Generic(_) => self.pointer_size, // Default for generic types
        }
    }
    
    pub fn alignment_of(&self, type_kind: &TypeKind) -> usize {
        match type_kind {
            TypeKind::Primitive(prim) => match prim {
                PrimitiveType::Void => 1,
                PrimitiveType::Bool => 1,
                PrimitiveType::Int8 | PrimitiveType::UInt8 | PrimitiveType::Char => 1,
                PrimitiveType::Int16 | PrimitiveType::UInt16 => 2,
                PrimitiveType::Int32 | PrimitiveType::UInt32 | PrimitiveType::Float32 => 4,
                PrimitiveType::Int64 | PrimitiveType::UInt64 | PrimitiveType::Float64 => 8,
                PrimitiveType::String => self.pointer_size,
            },
            TypeKind::Pointer(_) => self.pointer_size,
            TypeKind::Array(element_type, _) => self.alignment_of(&element_type.kind),
            TypeKind::Function(_) => self.pointer_size,
            TypeKind::Struct(s) => {
                s.fields.iter()
                    .map(|(_, field_type)| self.alignment_of(&field_type.kind))
                    .max()
                    .unwrap_or(1)
            }
            TypeKind::Union(u) => {
                u.variants.iter()
                    .map(|(_, variant_type)| self.alignment_of(&variant_type.kind))
                    .max()
                    .unwrap_or(1)
            }
            TypeKind::Enum(_) => 4,
            TypeKind::Generic(_) => self.default_alignment,
        }
    }
    
    pub fn align_offset(&self, offset: usize, alignment: usize) -> usize {
        (offset + alignment - 1) & !(alignment - 1)
    }
}

impl Default for TargetTypeConfig {
    fn default() -> Self {
        Self::x86_64()
    }
}
