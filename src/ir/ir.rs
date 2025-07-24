use crate::lexer::TokenType;
use std::fmt;

/// IR Value types - represents the type system in IR
#[derive(Debug, Clone, PartialEq)]
pub enum IrType {
    Int,
    Float,
    Char,
    String,
    Void,
    Pointer(Box<IrType>),
}

impl fmt::Display for IrType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IrType::Int => write!(f, "i32"),
            IrType::Float => write!(f, "f64"),
            IrType::Char => write!(f, "i8"),
            IrType::String => write!(f, "str"),
            IrType::Void => write!(f, "void"),
            IrType::Pointer(inner) => write!(f, "{}*", inner),
        }
    }
}

impl From<TokenType> for IrType {
    fn from(token_type: TokenType) -> Self {
        match token_type {
            TokenType::Int => IrType::Int,
            TokenType::FloatType => IrType::Float,
            TokenType::CharType => IrType::Char,
            TokenType::Void => IrType::Void,
            _ => IrType::Void, // Default fallback
        }
    }
}

/// IR Values - represents operands in IR instructions
#[derive(Debug, Clone, PartialEq)]
pub enum IrValue {
    /// Immediate integer constant
    IntConstant(i64),
    /// Immediate float constant
    FloatConstant(f64),
    /// Immediate character constant
    CharConstant(char),
    /// String literal with label
    StringConstant(String),
    /// Local variable reference
    Local(String),
    /// Temporary variable (generated during IR generation)
    Temp(usize),
    /// Function parameter
    Parameter(String),
    /// Global variable
    Global(String),
}

impl fmt::Display for IrValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IrValue::IntConstant(i) => write!(f, "{}", i),
            IrValue::FloatConstant(fl) => write!(f, "{}", fl),
            IrValue::CharConstant(c) => write!(f, "'{}'", c),
            IrValue::StringConstant(s) => write!(f, "\"{}\"", s),
            IrValue::Local(name) => write!(f, "%{}", name),
            IrValue::Temp(id) => write!(f, "%t{}", id),
            IrValue::Parameter(name) => write!(f, "%{}", name),
            IrValue::Global(name) => write!(f, "@{}", name),
        }
    }
}

impl Eq for IrValue {}

impl std::hash::Hash for IrValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            IrValue::IntConstant(i) => {
                0u8.hash(state);
                i.hash(state);
            }
            IrValue::FloatConstant(f) => {
                1u8.hash(state);
                f.to_bits().hash(state);
            }
            IrValue::CharConstant(c) => {
                2u8.hash(state);
                c.hash(state);
            }
            IrValue::StringConstant(s) => {
                3u8.hash(state);
                s.hash(state);
            }
            IrValue::Local(name) => {
                4u8.hash(state);
                name.hash(state);
            }
            IrValue::Temp(id) => {
                5u8.hash(state);
                id.hash(state);
            }
            IrValue::Parameter(name) => {
                6u8.hash(state);
                name.hash(state);
            }
            IrValue::Global(name) => {
                7u8.hash(state);
                name.hash(state);
            }
        }
    }
}

/// Binary operations in IR
#[derive(Debug, Clone, PartialEq)]
pub enum IrBinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    // Comparison operations
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    // Logical operations
    And,
    Or,
}

impl fmt::Display for IrBinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IrBinaryOp::Add => write!(f, "add"),
            IrBinaryOp::Sub => write!(f, "sub"),
            IrBinaryOp::Mul => write!(f, "mul"),
            IrBinaryOp::Div => write!(f, "div"),
            IrBinaryOp::Mod => write!(f, "mod"),
            IrBinaryOp::Eq => write!(f, "eq"),
            IrBinaryOp::Ne => write!(f, "ne"),
            IrBinaryOp::Lt => write!(f, "lt"),
            IrBinaryOp::Le => write!(f, "le"),
            IrBinaryOp::Gt => write!(f, "gt"),
            IrBinaryOp::Ge => write!(f, "ge"),
            IrBinaryOp::And => write!(f, "and"),
            IrBinaryOp::Or => write!(f, "or"),
        }
    }
}

impl From<TokenType> for IrBinaryOp {
    fn from(token_type: TokenType) -> Self {
        match token_type {
            TokenType::Plus => IrBinaryOp::Add,
            TokenType::Minus => IrBinaryOp::Sub,
            TokenType::Multiply => IrBinaryOp::Mul,
            TokenType::Divide => IrBinaryOp::Div,
            TokenType::Modulo => IrBinaryOp::Mod,
            TokenType::Equal => IrBinaryOp::Eq,
            TokenType::NotEqual => IrBinaryOp::Ne,
            TokenType::LessThan => IrBinaryOp::Lt,
            TokenType::LessEqual => IrBinaryOp::Le,
            TokenType::GreaterThan => IrBinaryOp::Gt,
            TokenType::GreaterEqual => IrBinaryOp::Ge,
            TokenType::LogicalAnd => IrBinaryOp::And,
            TokenType::LogicalOr => IrBinaryOp::Or,
            _ => panic!("Invalid binary operator: {:?}", token_type),
        }
    }
}

/// Unary operations in IR
#[derive(Debug, Clone, PartialEq)]
pub enum IrUnaryOp {
    Neg,
    Not,
}

impl fmt::Display for IrUnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IrUnaryOp::Neg => write!(f, "neg"),
            IrUnaryOp::Not => write!(f, "not"),
        }
    }
}

/// IR Instructions - the core of our intermediate representation
#[derive(Debug, Clone, PartialEq)]
pub enum IrInstruction {
    /// Variable declaration: alloca type name
    Alloca {
        var_type: IrType,
        name: String,
    },
    
    /// Load from memory: load type dest, src
    Load {
        dest: IrValue,
        src: IrValue,
        var_type: IrType,
    },
    
    /// Store to memory: store type value, dest
    Store {
        value: IrValue,
        dest: IrValue,
        var_type: IrType,
    },
    
    /// Binary operation: binop type dest, op, left, right
    BinaryOp {
        dest: IrValue,
        op: IrBinaryOp,
        left: IrValue,
        right: IrValue,
        var_type: IrType,
    },
    
    /// Unary operation: unop type dest, op, operand
    UnaryOp {
        dest: IrValue,
        op: IrUnaryOp,
        operand: IrValue,
        var_type: IrType,
    },
    
    /// Function call: call type dest, func, args
    Call {
        dest: Option<IrValue>,
        func: String,
        args: Vec<IrValue>,
        return_type: IrType,
    },
    
    /// Conditional branch: br condition, true_label, false_label
    Branch {
        condition: IrValue,
        true_label: String,
        false_label: String,
    },
    
    /// Unconditional jump: jmp label
    Jump {
        label: String,
    },
    
    /// Label definition: label:
    Label {
        name: String,
    },
    
    /// Return statement: ret type value
    Return {
        value: Option<IrValue>,
        var_type: IrType,
    },
    
    /// Print statement (built-in): print format, args
    Print {
        format_string: IrValue,
        args: Vec<IrValue>,
    },
    
    /// Move/Copy operation: mov type dest, src
    Move {
        dest: IrValue,
        src: IrValue,
        var_type: IrType,
    },
    
    /// Type conversion: convert dest_type dest, src_type src
    Convert {
        dest: IrValue,
        dest_type: IrType,
        src: IrValue,
        src_type: IrType,
    },
    
    /// Comment for debugging
    Comment {
        text: String,
    },
}

impl fmt::Display for IrInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IrInstruction::Alloca { var_type, name } => {
                write!(f, "  %{} = alloca {}", name, var_type)
            }
            IrInstruction::Load { dest, src, var_type } => {
                write!(f, "  {} = load {}, {}", dest, var_type, src)
            }
            IrInstruction::Store { value, dest, var_type } => {
                write!(f, "  store {} {}, {}", var_type, value, dest)
            }
            IrInstruction::BinaryOp { dest, op, left, right, var_type } => {
                write!(f, "  {} = {} {} {}, {}", dest, op, var_type, left, right)
            }
            IrInstruction::UnaryOp { dest, op, operand, var_type } => {
                write!(f, "  {} = {} {} {}", dest, op, var_type, operand)
            }
            IrInstruction::Call { dest, func, args, return_type } => {
                let args_str = args.iter()
                    .map(|arg| format!("{}", arg))
                    .collect::<Vec<_>>()
                    .join(", ");
                if let Some(dest_val) = dest {
                    write!(f, "  {} = call {} {}({})", dest_val, return_type, func, args_str)
                } else {
                    write!(f, "  call {} {}({})", return_type, func, args_str)
                }
            }
            IrInstruction::Branch { condition, true_label, false_label } => {
                write!(f, "  br {}, label %{}, label %{}", condition, true_label, false_label)
            }
            IrInstruction::Jump { label } => {
                write!(f, "  jmp label %{}", label)
            }
            IrInstruction::Label { name } => {
                write!(f, "{}:", name)
            }
            IrInstruction::Return { value, var_type } => {
                if let Some(val) = value {
                    write!(f, "  ret {} {}", var_type, val)
                } else {
                    write!(f, "  ret {}", var_type)
                }
            }
            IrInstruction::Print { format_string, args } => {
                let args_str = args.iter()
                    .map(|arg| format!("{}", arg))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "  print {}, [{}]", format_string, args_str)
            }
            IrInstruction::Move { dest, src, var_type } => {
                write!(f, "  {} = mov {} {}", dest, var_type, src)
            }
            IrInstruction::Convert { dest, dest_type, src, src_type } => {
                write!(f, "  {} = convert {} {} to {}", dest, src_type, src, dest_type)
            }
            IrInstruction::Comment { text } => {
                write!(f, "  ; {}", text)
            }
        }
    }
}

/// IR Function representation
#[derive(Debug, Clone)]
pub struct IrFunction {
    pub name: String,
    pub return_type: IrType,
    pub parameters: Vec<(String, IrType)>,
    pub instructions: Vec<IrInstruction>,
    pub local_vars: Vec<(String, IrType)>,
}

impl fmt::Display for IrFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Function signature
        let params_str = self.parameters.iter()
            .map(|(name, ty)| format!("{} %{}", ty, name))
            .collect::<Vec<_>>()
            .join(", ");
        
        writeln!(f, "define {} @{}({}) {{", self.return_type, self.name, params_str)?;
        
        // Instructions
        for instruction in &self.instructions {
            writeln!(f, "{}", instruction)?;
        }
        
        writeln!(f, "}}")?;
        Ok(())
    }
}

/// Complete IR Program
#[derive(Debug, Clone)]
pub struct IrProgram {
    pub functions: Vec<IrFunction>,
    pub global_strings: Vec<(String, String)>, // (label, content)
}

impl fmt::Display for IrProgram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "; IR Program Generated by Mini-C Compiler")?;
        writeln!(f, "")?;
        
        // Global strings
        if !self.global_strings.is_empty() {
            writeln!(f, "; Global string constants")?;
            for (label, content) in &self.global_strings {
                writeln!(f, "@{} = constant str \"{}\"", label, content)?;
            }
            writeln!(f, "")?;
        }
        
        // Functions
        for function in &self.functions {
            writeln!(f, "{}", function)?;
        }
        
        Ok(())
    }
}