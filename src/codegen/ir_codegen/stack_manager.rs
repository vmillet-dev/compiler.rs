use crate::ir::{IrFunction, IrInstruction, IrValue, IrType};
use super::IrCodegen;

impl IrCodegen {
    /// Calculate the stack space needed for a function
    pub fn calculate_stack_space(&mut self, function: &IrFunction) -> i32 {
        let mut space = 32; // Shadow space for Windows x64 ABI
        
        // Allocate space for local variables
        for (name, ir_type) in &function.local_vars {
            let size = self.get_type_size(ir_type);
            space += size;
            self.locals.insert(name.clone(), -space);
        }
        
        // Allocate space for temporary variables
        let mut _temp_count = 0;
        for instruction in &function.instructions {
            if let Some(temp_id) = self.extract_temp_id(instruction) {
                if !self.temp_locations.contains_key(&temp_id) {
                    _temp_count += 1;
                    space += 8; // Assume 8 bytes for all temps
                    self.temp_locations.insert(temp_id, -space);
                }
            }
        }
        
        // Align to 16 bytes
        (space + 15) & !15
    }

    /// Extract temporary variable ID from instruction if present
    pub fn extract_temp_id(&self, instruction: &IrInstruction) -> Option<usize> {
        match instruction {
            IrInstruction::BinaryOp { dest, .. } |
            IrInstruction::UnaryOp { dest, .. } |
            IrInstruction::Load { dest, .. } |
            IrInstruction::Move { dest, .. } => {
                if let IrValue::Temp(id) = dest {
                    Some(*id)
                } else {
                    None
                }
            }
            IrInstruction::Call { dest: Some(dest), .. } => {
                if let IrValue::Temp(id) = dest {
                    Some(*id)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Get the size in bytes for an IR type
    pub fn get_type_size(&self, ir_type: &IrType) -> i32 {
        match ir_type {
            IrType::Int => 4,
            IrType::Float => 8,
            IrType::Char => 1,
            IrType::String => 8, // Pointer size
            IrType::Void => 0,
            IrType::Pointer(_) => 8,
        }
    }
}