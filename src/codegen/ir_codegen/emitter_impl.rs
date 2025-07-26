use crate::codegen::emitter::Emitter;
use super::IrCodegen;

// Implement the emitter traits for IrCodegen
impl Emitter for IrCodegen {
    fn emit_line(&mut self, line: &str) {
        self.output.push_str(line);
        self.output.push('\n');
    }

    fn emit_comment(&mut self, comment: &str) {
        self.emit_line(&format!("; {}", comment));
    }
}

// Helper methods for IrCodegen
impl IrCodegen {
    /// Emit a section header with clear visual separation
    pub fn emit_section_header(&mut self, title: &str) {
        self.emit_line("");
        self.emit_line(&format!("; {}", "=".repeat(60)));
        self.emit_line(&format!("; {}", title));
        self.emit_line(&format!("; {}", "=".repeat(60)));
        self.emit_line("");
    }

    /// Emit a subsection header for better organization
    pub fn emit_subsection_header(&mut self, title: &str) {
        self.emit_line("");
        self.emit_line(&format!("; {}", "-".repeat(40)));
        self.emit_line(&format!("; {}", title));
        self.emit_line(&format!("; {}", "-".repeat(40)));
    }

    /// Emit a stack layout summary for debugging
    pub fn emit_stack_layout_summary(&mut self) {
        self.emit_comment("STACK LAYOUT SUMMARY:");
        self.emit_comment("RBP+0  : Saved RBP (caller's frame pointer)");
        
        if self.locals.is_empty() && self.temp_locations.is_empty() {
            self.emit_comment("No local variables or temporaries allocated");
        } else {
            // Collect local variables info to avoid borrowing issues
            let locals_info: Vec<(String, i32)> = self.locals.iter()
                .map(|(name, &offset)| (name.clone(), offset))
                .collect();
            
            if !locals_info.is_empty() {
                self.emit_comment("Local variables:");
                for (name, offset) in locals_info {
                    self.emit_comment(&format!("RBP{:3} : {}", offset, name));
                }
            }
            
            // Collect temp variables info to avoid borrowing issues
            let temps_info: Vec<(usize, i32)> = self.temp_locations.iter()
                .map(|(&temp_id, &offset)| (temp_id, offset))
                .collect();
            
            if !temps_info.is_empty() {
                self.emit_comment("Temporary variables:");
                for (temp_id, offset) in temps_info {
                    self.emit_comment(&format!("RBP{:3} : %t{}", offset, temp_id));
                }
            }
        }
        self.emit_line("");
    }
}