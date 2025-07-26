# Target Interface Documentation

The Mini-C compiler now supports multiple target platforms through a flexible target interface. This allows you to generate platform-specific assembly code for different operating systems and architectures.

## Supported Targets

### Windows x64
- **Platform**: `TargetPlatform::WindowsX64`
- **Calling Convention**: Microsoft x64
- **Assembly Format**: NASM-compatible x86-64
- **External Functions**: `printf`, `exit`
- **Global Symbols**: `main`

### Linux x64
- **Platform**: `TargetPlatform::LinuxX64`
- **Calling Convention**: System V ABI
- **Assembly Format**: NASM-compatible x86-64
- **External Functions**: `printf`, `exit`
- **Global Symbols**: `main`
- **Startup Code**: Includes `_start` entry point with system call exit

### macOS x64
- **Platform**: `TargetPlatform::MacOSX64`
- **Calling Convention**: Apple x64 ABI (System V-like)
- **Assembly Format**: NASM-compatible x86-64
- **External Functions**: `_printf`, `_exit` (with underscore prefix)
- **Global Symbols**: `_main` (with underscore prefix)

## Usage

### Command Line Interface

You can specify the target platform using the `--target` flag:

```bash
# Compile for Windows (default)
cargo run -- --target windows input.c

# Compile for Linux
cargo run -- --target linux input.c

# Compile for macOS
cargo run -- --target macos input.c
```

### Programmatic Usage

```rust
use compiler_minic::codegen::{IrCodegen, TargetPlatform};

// Create code generator for specific target
let codegen = IrCodegen::new_with_target(TargetPlatform::LinuxX64);

// Generate assembly
let assembly = codegen.generate(&ir_program);
```

### Target Selection

You can parse target strings using the helper function:

```rust
use compiler_minic::codegen::parse_target_platform;

let target = parse_target_platform("linux").unwrap();
// Returns TargetPlatform::LinuxX64
```

Supported target strings:
- Windows: `"windows"`, `"win"`, `"windows-x64"`, `"win64"`
- Linux: `"linux"`, `"linux-x64"`, `"linux64"`
- macOS: `"macos"`, `"darwin"`, `"macos-x64"`, `"darwin-x64"`

## Target Interface

The `Target` trait defines platform-specific behavior:

```rust
pub trait Target {
    // Platform identification
    fn platform(&self) -> TargetPlatform;
    fn calling_convention(&self) -> CallingConvention;
    
    // Assembly generation
    fn assembly_directives(&self) -> Vec<String>;
    fn data_section_header(&self) -> String;
    fn text_section_header(&self) -> String;
    
    // Function conventions
    fn function_prologue(&self) -> Vec<String>;
    fn function_epilogue(&self) -> Vec<String>;
    fn parameter_registers(&self) -> Vec<Register>;
    fn return_register(&self) -> Register;
    
    // Platform-specific formatting
    fn format_string_literal(&self, label: &str, content: &str) -> String;
    fn format_function_call(&self, function_name: &str) -> Vec<String>;
    
    // Type information
    fn type_info(&self, type_name: &str) -> (usize, usize); // (size, alignment)
}
```

## Key Differences Between Targets

### Symbol Naming
- **Windows/Linux**: Standard names (`main`, `printf`)
- **macOS**: Underscore prefix (`_main`, `_printf`)

### Calling Conventions
- **Windows**: Microsoft x64 calling convention
  - Parameters: RCX, RDX, R8, R9
  - Return: RAX
- **Linux**: System V ABI
  - Parameters: RDI, RSI, RDX, RCX, R8, R9 (simplified in current implementation)
  - Return: RAX
- **macOS**: Apple x64 ABI (System V-like)
  - Similar to Linux but with underscore prefixes

### Startup Code
- **Windows/macOS**: No special startup code needed
- **Linux**: Includes `_start` entry point that calls `main` and exits via system call

## Adding New Targets

To add support for a new target platform:

1. Add the platform to `TargetPlatform` enum
2. Add calling convention to `CallingConvention` enum if needed
3. Create a new target implementation struct
4. Implement the `Target` trait
5. Update the `create_target` factory function
6. Update the `parse_target_platform` function

Example:

```rust
pub struct Arm64Target;

impl Target for Arm64Target {
    fn platform(&self) -> TargetPlatform {
        TargetPlatform::Arm64
    }
    
    fn calling_convention(&self) -> CallingConvention {
        CallingConvention::AAPCS64
    }
    
    // ... implement other methods
}
```

## Examples

See `examples/target_demo.rs` for a complete example of compiling the same source code for multiple targets and comparing the output.

## Testing

The target interface is tested through the existing integration tests, which automatically use the default Windows target. The interface ensures backward compatibility while enabling cross-platform code generation.