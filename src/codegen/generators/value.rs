use crate::ir::{IrValue, IrType};
use crate::codegen::core::{Operand, Register, Size};
use crate::codegen::backend::IrBackend;

impl IrBackend {
    /// Convert IR value to assembly operand
    pub fn ir_value_to_operand(&self, value: &IrValue) -> Operand {
        match value {
            IrValue::IntConstant(i) => Operand::Immediate(*i),
            IrValue::FloatConstant(_f) => {
                panic!("Float constants cannot be used as immediate operands - must be pre-loaded into memory")
            }
            IrValue::CharConstant(c) => Operand::Immediate(*c as i64),
            IrValue::StringConstant(label) => Operand::Label(label.clone()),
            IrValue::Local(name) => {
                let offset = self.locals.get(name).copied().unwrap_or(0);
                Operand::Memory { base: Register::Rbp, offset }
            }
            IrValue::Temp(id) => {
                let offset = self.temp_locations.get(id).copied().unwrap_or(0);
                Operand::Memory { base: Register::Rbp, offset }
            }
            IrValue::Parameter(_name) => {
                // Parameters would be at positive offsets from RBP
                let offset = 16; // Simplified - would need proper parameter handling
                Operand::Memory { base: Register::Rbp, offset }
            }
            IrValue::Global(name) => Operand::Label(name.clone()),
        }
    }

    /// Convert IR type to assembly size
    pub fn ir_type_to_size(&self, ir_type: &IrType) -> Size {
        match ir_type {
            IrType::Int => Size::Dword,
            IrType::Float => Size::Qword,
            IrType::Char => Size::Byte,
            IrType::String => Size::Qword,
            IrType::Void => Size::Qword,
            IrType::Pointer(_) => Size::Qword,
        }
    }

    /// Convert IR value to string for comments
    pub fn ir_value_to_string(&self, value: &IrValue) -> String {
        match value {
            IrValue::IntConstant(i) => i.to_string(),
            IrValue::FloatConstant(f) => f.to_string(),
            IrValue::CharConstant(c) => format!("'{}'", c),
            IrValue::StringConstant(label) => format!("@{}", label),
            IrValue::Local(name) => format!("%{}", name),
            IrValue::Temp(id) => format!("%t{}", id),
            IrValue::Parameter(name) => format!("%{}", name),
            IrValue::Global(name) => format!("@{}", name),
        }
    }
}