use crate::codegen::instruction::Register;

#[derive(Debug, Clone)]
pub struct CallingConvention {
    pub name: String,
    pub stack_alignment: usize,
    pub shadow_space_size: usize,
    pub integer_registers: Vec<Register>,
    pub float_registers: Vec<Register>,
    pub return_register: Register,
}

impl CallingConvention {
    pub fn windows_x64() -> Self {
        Self {
            name: "Windows x64".to_string(),
            stack_alignment: 16,
            shadow_space_size: 32,
            integer_registers: vec![
                Register::Rcx,
                Register::Rdx, 
                Register::R8,
                Register::R9,
            ],
            float_registers: vec![
                Register::Xmm0,
                Register::Xmm1,
                Register::Xmm2,
                Register::Xmm3,
            ],
            return_register: Register::Rax,
        }
    }
    
    pub fn system_v_x64() -> Self {
        Self {
            name: "System V x64".to_string(),
            stack_alignment: 16,
            shadow_space_size: 0,
            integer_registers: vec![
                Register::Rdx,  // Using available registers only
                Register::Rcx,
                Register::R8,
                Register::R9,
            ],
            float_registers: vec![
                Register::Xmm0,
                Register::Xmm1,
                Register::Xmm2,
                Register::Xmm3,
            ],
            return_register: Register::Rax,
        }
    }
    
    pub fn get_integer_register(&self, index: usize) -> Option<Register> {
        self.integer_registers.get(index).copied()
    }
    
    pub fn get_float_register(&self, index: usize) -> Option<Register> {
        self.float_registers.get(index).copied()
    }
    
    pub fn max_register_args(&self) -> usize {
        self.integer_registers.len().min(self.float_registers.len())
    }
}

#[derive(Debug, Clone)]
pub struct FunctionCallGenerator {
    calling_convention: CallingConvention,
}

impl FunctionCallGenerator {
    pub fn new(calling_convention: CallingConvention) -> Self {
        Self { calling_convention }
    }
    
    pub fn windows_x64() -> Self {
        Self::new(CallingConvention::windows_x64())
    }
    
    pub fn calling_convention(&self) -> &CallingConvention {
        &self.calling_convention
    }
    
    pub fn generate_stack_alignment(&self) -> Vec<String> {
        let mut instructions = Vec::new();
        let alignment = self.calling_convention.stack_alignment;
        
        instructions.push(format!("    ; Align stack to {}-byte boundary", alignment));
        instructions.push(format!("    and     rsp, ~{}            ; Force alignment", alignment - 1));
        
        if self.calling_convention.shadow_space_size > 0 {
            instructions.push(format!("    sub     rsp, {}             ; Allocate shadow space", 
                self.calling_convention.shadow_space_size));
        }
        
        instructions
    }
    
    pub fn generate_stack_cleanup(&self) -> Vec<String> {
        let mut instructions = Vec::new();
        
        if self.calling_convention.shadow_space_size > 0 {
            instructions.push(format!("    add     rsp, {}             ; Deallocate shadow space", 
                self.calling_convention.shadow_space_size));
        }
        
        instructions
    }
    
    pub fn generate_argument_passing(&self, args: &[String], arg_types: &[String]) -> Vec<String> {
        let mut instructions = Vec::new();
        
        for (i, (arg, arg_type)) in args.iter().zip(arg_types.iter()).enumerate() {
            if i >= self.calling_convention.max_register_args() {
                instructions.push(format!("    ; Stack argument {}: {} (not implemented)", i, arg));
                continue;
            }
            
            match arg_type.as_str() {
                "int" | "integer" => {
                    if let Some(reg) = self.calling_convention.get_integer_register(i) {
                        instructions.push(format!("    mov     {}, {}              ; Integer argument {}", 
                            reg.to_string().to_lowercase(), arg, i));
                    }
                }
                "float" | "double" => {
                    if let Some(reg) = self.calling_convention.get_float_register(i) {
                        instructions.push(format!("    movsd   {}, {}              ; Float argument {}", 
                            reg.to_string().to_lowercase(), arg, i));
                        
                        if self.calling_convention.name.contains("Windows") {
                            if let Some(int_reg) = self.calling_convention.get_integer_register(i) {
                                instructions.push(format!("    movq    {}, {}              ; Copy to integer register", 
                                    int_reg.to_string().to_lowercase(), reg.to_string().to_lowercase()));
                            }
                        }
                    }
                }
                "char" => {
                    if let Some(reg) = self.calling_convention.get_integer_register(i) {
                        instructions.push(format!("    movzx   {}, {}              ; Character argument {}", 
                            reg.to_string().to_lowercase(), arg, i));
                    }
                }
                _ => {
                    instructions.push(format!("    ; Unknown argument type: {} for arg {}", arg_type, i));
                }
            }
        }
        
        instructions
    }
}
