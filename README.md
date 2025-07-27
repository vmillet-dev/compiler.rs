# MiniC Compiler

A modern, educational compiler for a C-like programming language, written in Rust. This project demonstrates the complete compilation pipeline from source code to executable x86-64 assembly, featuring lexical analysis, parsing, intermediate representation, optimization, and code generation.

## Overview

MiniC is a subset of the C programming language designed for educational purposes and compiler development learning. The compiler implements a complete toolchain that transforms C-like source code into optimized x86-64 assembly code through a sophisticated multi-stage pipeline.

**Key Features:**
- **Complete Compilation Pipeline**: Lexing → Parsing → Semantic Analysis → IR Generation → Optimization → Code Generation
- **Cross-Platform Support**: Windows, Linux, and macOS target platforms
- **Memory Safety Analysis**: Static analysis to detect potential memory safety issues
- **IR-Based Optimization**: Constant folding, dead code elimination, and copy propagation
- **Professional CLI**: Modern command-line interface with comprehensive options
- **Comprehensive Testing**: Unit and integration tests covering the entire pipeline

## Installation

### Prerequisites

- **Rust**: Version 1.88.0 or later ([Install Rust](https://rustup.rs/))
- **NASM**: Netwide Assembler for assembly compilation
- **GCC**: GNU Compiler Collection for linking

#### Installing Dependencies

**Windows:**
```bash
# Install NASM via Chocolatey
choco install nasm

# Install GCC via MSYS2 or use Visual Studio Build Tools
```

**Linux (Ubuntu/Debian):**
```bash
sudo apt-get update
sudo apt-get install -y nasm gcc
```

**macOS:**
```bash
brew install nasm gcc
```

### Building the Compiler

```bash
# Clone the repository
git clone <repository-url>
cd compiler-minic

# Build the project
cargo build --release

# Run tests to verify installation
cargo test
```

## Usage

### Basic Commands

```bash
# Compile with default example code
cargo run

# Compile a specific source file
cargo run -- input.c

# Show help and all available options
cargo run -- --help
```

### Command Line Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `<FILE>` | | Input source file to compile | Built-in example |
| `--target` | `-t` | Target platform (windows-x64, linux-x64, macos-x64, macos-arm64) | windows-x64 |
| `--output-dir` | `-o` | Output directory for generated files | build |
| `--verbose` | `-v` | Enable detailed compilation output | false |
| `--skip-memory-checks` | | Skip memory safety analysis | false |
| `--skip-optimization` | | Skip IR optimization passes | false |

### Examples

**Basic compilation with verbose output:**
```bash
cargo run -- --verbose program.c
```

**Cross-compilation for Linux:**
```bash
cargo run -- --target linux-x64 --output-dir ./linux_build program.c
```

**Fast development build (skip optimizations):**
```bash
cargo run -- --skip-optimization --skip-memory-checks --verbose program.c
```

**Complete workflow with custom output:**
```bash
cargo run -- --target windows-x64 --output-dir ./release --verbose program.c
```

### Assembly and Execution

After compilation, assemble and link the generated code:

**Windows:**
```bash
nasm -f win64 build/output.asm -o build/output.obj
gcc -o build/output.exe build/output.obj -lmsvcrt
./build/output.exe
```

**Linux:**
```bash
nasm -f elf64 build/output.asm -o build/output.o
gcc -o build/output build/output.o -no-pie
./build/output
```

**macOS:**
```bash
nasm -f macho64 build/output.asm -o build/output.o
gcc -o build/output build/output.o
./build/output
```

## Project Structure

```
compiler-minic/
├── src/                          # Core compiler implementation
│   ├── main.rs                   # CLI entry point and compilation orchestration
│   ├── lib.rs                    # Library root with module declarations
│   ├── error/                    # Comprehensive error handling system
│   │   ├── mod.rs                # Error module exports
│   │   └── error.rs              # CompilerError enum with detailed error types
│   ├── lexer/                    # Lexical analysis (tokenization)
│   │   ├── mod.rs                # Lexer module exports
│   │   ├── lexer.rs              # Tokenizer implementation with position tracking
│   │   └── token.rs              # Token definitions and types
│   ├── parser/                   # Syntax analysis and AST construction
│   │   ├── mod.rs                # Parser module exports
│   │   ├── parser.rs             # Recursive descent parser implementation
│   │   └── ast.rs                # Abstract Syntax Tree node definitions
│   ├── semantic/                 # Static analysis and memory safety
│   │   ├── mod.rs                # Semantic analysis module exports
│   │   └── memory_safety.rs     # Memory safety checker implementation
│   ├── ir/                       # Intermediate Representation
│   │   ├── mod.rs                # IR module exports
│   │   ├── ir.rs                 # IR instruction set and data structures
│   │   ├── generator.rs          # AST to IR translation
│   │   └── optimizer.rs          # IR optimization passes
│   ├── codegen/                  # Code generation to x86-64 assembly
│   │   ├── mod.rs                # Code generation module exports and re-exports
│   │   ├── codegen.rs            # Main code generator implementation
│   │   ├── core/                 # Core abstractions and target implementations
│   │   │   ├── mod.rs            # Core module exports
│   │   │   ├── emitter.rs        # Assembly instruction emission traits
│   │   │   ├── instruction.rs    # x86-64 instruction and operand definitions
│   │   │   └── targets/          # Target platform implementations
│   │   │       ├── mod.rs        # Target module exports
│   │   │       ├── base.rs       # Base target trait and common functionality
│   │   │       ├── windows.rs    # Windows x86-64 target implementation
│   │   │       ├── linux.rs      # Linux x86-64 target implementation
│   │   │       └── macos.rs      # macOS x86-64/ARM64 target implementations
│   │   ├── utils/                # Code generation utilities
│   │   │   ├── mod.rs            # Utils module exports
│   │   │   ├── formatter.rs      # Assembly instruction formatting
│   │   │   ├── stack_manager.rs  # Stack frame management
│   │   │   └── register_allocator.rs # Register allocation utilities
│   │   └── generators/           # Specialized code generators
│   │       ├── mod.rs            # Generators module exports
│   │       ├── instruction.rs    # IR instruction code generation
│   │       ├── operation.rs      # Operation code generation
│   │       ├── value.rs          # Value handling and conversion
│   │       ├── function.rs       # Function prologue/epilogue generation
│   │       └── call.rs           # Function call code generation
│   └── types/                    # Type system and definitions
│       ├── mod.rs                # Type system module exports
│       └── types.rs              # Type definitions and utilities
├── tests/                        # Comprehensive test suite
│   └── integration_tests.rs      # End-to-end compilation pipeline tests
├── docs/                         # Documentation and guides
│   ├── CLI_USAGE.md              # Detailed CLI usage guide
│   ├── IR_IMPLEMENTATION.md      # IR design and implementation details
│   ├── TARGET_INTERFACE.md       # Target platform interface documentation
│   └── COMPILER_REVIEW.md        # Architecture and design decisions
├── examples/                     # Usage examples and demonstrations
│   ├── codegen_usage.rs          # Code generation API examples
│   └── target_demo.rs            # Target platform demonstration
├── .github/                      # CI/CD configuration
│   └── workflows/
│       └── ci.yml                # GitHub Actions workflow
├── Cargo.toml                    # Rust project configuration
└── README.md                     # This file
```

## Architecture

### Compilation Pipeline

The MiniC compiler follows a traditional multi-pass architecture:

```
Source Code → Lexer → Parser → Semantic Analysis → IR Generation → Optimization → Code Generation → Assembly
```

1. **Lexical Analysis**: Converts source text into tokens with position tracking
2. **Syntax Analysis**: Builds an Abstract Syntax Tree (AST) using recursive descent parsing
3. **Semantic Analysis**: Performs memory safety checks and static analysis
4. **IR Generation**: Translates AST to platform-independent intermediate representation
5. **Optimization**: Applies optimization passes (constant folding, dead code elimination)
6. **Code Generation**: Produces target-specific x86-64 assembly code

### Design Decisions

**Error Handling Strategy:**
- Comprehensive error types covering all compilation phases
- Position tracking for precise error reporting
- Graceful error recovery where possible
- Rust's `Result` type for safe error propagation

**AST Design:**
- Separate enums for expressions and statements
- `Box<T>` for recursive structures to enable heap allocation
- `PartialEq` derivation for testing and comparison
- Immutable design for thread safety

**Intermediate Representation:**
- SSA-like form with temporary variables
- Platform-independent instruction set
- Optimization-friendly structure
- Type information preservation

**Code Generation:**
- Target-specific assembly generation
- System V ABI compliance for function calls
- Stack-based variable storage
- NASM-compatible output format

**Memory Management:**
- Stack allocation for local variables
- No dynamic memory allocation in generated code
- Static analysis for memory safety verification
- Rust's ownership system for compiler memory safety

## Supported Language Features

### Data Types
- `int`: 64-bit signed integers
- `float`: 64-bit IEEE 754 floating-point numbers
- `char`: Single ASCII characters
- `void`: For functions with no return value

### Operators
- **Arithmetic**: `+`, `-`, `*`, `/` (with proper precedence)
- **Comparison**: `==`, `!=`, `<`, `<=`, `>`, `>=`
- **Logical**: `&&`, `||`, `!`
- **Unary**: `-` (negation), `!` (logical not)

### Control Flow
- `if` statements with optional `else` branches
- `return` statements with optional values
- Function definitions and calls
- Block statements with proper scoping

### Built-in Functions
- `println()`: Formatted output with newline
- Format specifiers: `%d` (integers), `%f` (floats), `%c` (characters)

### Example Program
```c
int main() {
    int x = 42;
    float pi = 3.14159;
    char grade = 'A';
    
    if (x > 40) {
        println("Number is: %d", x);
        println("Pi approximation: %.3f", pi);
        println("Grade: %c", grade);
    }
    
    int result = x * 2 + 10;
    return result;
}

float calculate_area() {
    float radius = 5.0;
    float pi = 3.14159;
    return pi * radius * radius;
}
```

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test integration_tests

# Run tests with verbose output
cargo test -- --nocapture
```

### Test Coverage

**Unit Tests:**
- Lexer: Token recognition, position tracking, error handling
- Parser: AST construction, operator precedence, error recovery
- IR: Instruction generation, optimization passes
- Code Generation: Assembly output, register allocation

**Integration Tests:**
- Complete compilation pipeline validation
- Cross-platform assembly generation
- Optimization effectiveness verification
- Error handling across all phases

## Contributing

This project serves as an educational resource for understanding compiler construction. Contributions are welcome in the following areas:

- **Language Features**: Additional operators, control structures, data types
- **Optimizations**: Advanced optimization passes, register allocation improvements
- **Target Platforms**: Additional architecture support (ARM64, RISC-V)
- **Error Handling**: Better error messages and recovery strategies
- **Documentation**: Examples, tutorials, and architectural guides

## License

This project is an educational compiler implementation demonstrating fundamental concepts of programming language design and implementation. It is provided as-is for learning and research purposes.

---

**Note**: This compiler is designed for educational purposes and may not be suitable for production use. It demonstrates core compiler concepts and serves as a foundation for learning about language implementation.
