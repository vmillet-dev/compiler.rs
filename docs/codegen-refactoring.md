# Code Generation Module Refactoring

## Overview

The `codegen` module has been completely refactored to provide a clean, well-organized, and maintainable code generation pipeline. The new architecture separates concerns and provides better abstractions for different aspects of code generation.

## New Architecture

### Core Structure

```
src/codegen/
├── core/                    # Core abstractions and interfaces
│   ├── mod.rs              # Module exports
│   ├── instruction.rs      # Assembly instruction definitions
│   ├── emitter.rs          # Code emission traits
│   └── target.rs           # Target platform abstractions
├── backends/               # Different backend implementations
│   ├── mod.rs              # Backend exports
│   └── ir_backend.rs       # IR-based code generation backend
├── generators/             # Specialized code generators
│   ├── mod.rs              # Generator exports
│   ├── function.rs         # Function-specific generation
│   ├── instruction.rs      # Instruction generation utilities
│   ├── operation.rs        # Operation generation utilities
│   └── call.rs             # Function call generation
├── utils/                  # Utility modules
│   ├── mod.rs              # Utility exports
│   ├── stack_manager.rs    # Stack layout management
│   ├── register_allocator.rs # Register allocation
│   └── formatter.rs        # Assembly formatting utilities
├── targets/                # Target-specific implementations
│   ├── mod.rs              # Target exports
│   ├── windows_x64.rs      # Windows x64 target
│   ├── linux_x64.rs        # Linux x64 target
│   └── macos_x64.rs        # macOS x64 target
└── [legacy modules]        # Backward compatibility
```

## Key Improvements

### 1. Separation of Concerns

- **Core**: Defines fundamental abstractions (instructions, emitters, targets)
- **Backends**: Implements different compilation strategies (IR-based, AST-based)
- **Generators**: Specialized generators for different code aspects
- **Utils**: Reusable utilities for stack management, register allocation, etc.
- **Targets**: Platform-specific implementations

### 2. Target Platform Abstraction

The new `Target` trait provides a clean abstraction for different platforms:

```rust
pub trait Target {
    fn platform(&self) -> TargetPlatform;
    fn calling_convention(&self) -> CallingConvention;
    fn assembly_directives(&self) -> Vec<String>;
    fn function_prologue(&self) -> Vec<String>;
    fn function_epilogue(&self) -> Vec<String>;
    fn parameter_registers(&self) -> Vec<Register>;
    // ... and more
}
```

Supported platforms:
- **Windows x64**: Microsoft x64 calling convention
- **Linux x64**: System V ABI calling convention  
- **macOS x64**: Apple x64 ABI calling convention

### 3. Improved Code Emission

The new emitter traits provide better abstractions:

```rust
pub trait Emitter {
    fn emit_line(&mut self, line: &str);
    fn emit_comment(&mut self, comment: &str);
}

pub trait CodeEmitter: Emitter {
    fn emit_instruction(&mut self, instruction: Instruction, operands: Vec<Operand>);
    fn emit_instruction_with_size(&mut self, instruction: Instruction, size: Size, operands: Vec<Operand>);
}

pub trait CodeEmitterWithComment: CodeEmitter {
    fn emit_instruction_with_comment(&mut self, instruction: Instruction, operands: Vec<Operand>, comment: Option<&str>);
    fn emit_instruction_with_size_and_comment(&mut self, instruction: Instruction, size: Size, operands: Vec<Operand>, comment: Option<&str>);
    // ... and more
}
```

### 4. Modular Code Generation

Specialized generators handle different aspects:

- **FunctionGenerator**: Function prologues, epilogues, and setup
- **InstructionGenerator**: Individual IR instruction translation
- **OperationGenerator**: Arithmetic and logical operations
- **CallGenerator**: Function calls with proper calling conventions

### 5. Better Resource Management

- **StackManager**: Handles stack layout and variable allocation
- **RegisterAllocator**: Manages register allocation and tracking
- **AssemblyFormatter**: Provides consistent assembly formatting

## Usage Examples

### Basic Usage

```rust
use crate::codegen::{IrBackend, TargetPlatform};

let mut backend = IrBackend::new_with_target(TargetPlatform::WindowsX64);
backend.set_ir_program(ir_program);
let assembly = backend.generate();
```

### Custom Target

```rust
use crate::codegen::{create_target, TargetPlatform};

let target = create_target(TargetPlatform::LinuxX64);
let prologue = target.function_prologue();
let param_regs = target.parameter_registers();
```

### Using Utilities

```rust
use crate::codegen::{StackManager, RegisterAllocator};

let mut stack_manager = StackManager::new();
let offset = stack_manager.allocate_local("var1".to_string(), TokenType::Int);

let mut reg_allocator = RegisterAllocator::new();
let reg = reg_allocator.allocate_register("temp1".to_string());
```

## Backward Compatibility

The refactoring maintains backward compatibility by:

1. **Legacy Module Exports**: Old modules are still available but marked as legacy
2. **Re-exports**: Main types are re-exported from the root module
3. **Gradual Migration**: The old `IrCodegen` is still available alongside the new `IrBackend`

## Migration Path

To migrate from the old system to the new one:

1. **Replace IrCodegen with IrBackend**:
   ```rust
   // Old
   let ir_codegen = IrCodegen::new_with_target(target_platform);
   let asm_code = ir_codegen.generate(ir_program);
   
   // New
   let mut ir_backend = IrBackend::new_with_target(target_platform);
   ir_backend.set_ir_program(ir_program);
   let asm_code = ir_backend.generate();
   ```

2. **Use new target abstractions**:
   ```rust
   // Old
   let target = WindowsX64Target;
   
   // New
   let target = create_target(TargetPlatform::WindowsX64);
   ```

3. **Leverage new utilities**:
   ```rust
   // Use StackManager for stack layout
   // Use RegisterAllocator for register management
   // Use specialized generators for different code aspects
   ```

## Benefits

1. **Maintainability**: Clear separation of concerns makes the code easier to understand and modify
2. **Extensibility**: Easy to add new targets, backends, or generators
3. **Testability**: Modular design allows for better unit testing
4. **Reusability**: Utilities can be shared across different backends
5. **Performance**: Better resource management and optimization opportunities
6. **Documentation**: Self-documenting code with clear abstractions

## Future Enhancements

The new architecture enables several future improvements:

1. **Additional Backends**: AST-based backend, LLVM backend
2. **More Targets**: ARM64, RISC-V, WebAssembly
3. **Advanced Optimizations**: Register allocation algorithms, instruction scheduling
4. **Better Debugging**: Source maps, debug information generation
5. **Code Analysis**: Static analysis, profiling integration

## Conclusion

The refactored code generation module provides a solid foundation for the MiniC compiler's code generation pipeline. It offers better organization, maintainability, and extensibility while maintaining backward compatibility with existing code.