# Compiler MiniC

A simple C-like language compiler written in Rust that compiles to x86-64 assembly.

## Overview

This project implements a complete compiler pipeline for a subset of the C programming language, featuring:
- **Lexical Analysis** (Tokenization)
- **Syntax Analysis** (Parsing to AST)
- **Code Generation** (x86-64 Assembly output)

The compiler can handle basic C constructs including variables, functions, control flow, and basic I/O operations.

## Project Structure

```
compiler-minic/
├── src/
│   ├── main.rs              # Entry point and example usage
│   ├── lib.rs               # Library root with module declarations
│   ├── error/               # Error handling
│   │   ├── mod.rs           # Error module exports
│   │   └── error.rs         # CompilerError enum and implementations
│   ├── lexer/               # Lexical analysis
│   │   ├── mod.rs           # Lexer module exports
│   │   ├── lexer.rs         # Lexer implementation
│   │   └── token.rs         # Token types and definitions
│   ├── parser/              # Syntax analysis
│   │   ├── mod.rs           # Parser module exports
│   │   ├── parser.rs        # Recursive descent parser
│   │   └── ast.rs           # Abstract Syntax Tree definitions
│   └── codegen/             # Code generation
│       ├── mod.rs           # Codegen module exports
│       └── codegen.rs       # x86-64 assembly code generator
├── .github/
│   └── workflows/
│       └── ci.yml           # GitHub Actions CI/CD pipeline
├── Cargo.toml               # Rust project configuration
├── .gitignore               # Git ignore rules
└── output.asm               # Generated assembly output (created at runtime)
```

## Components

### 1. Error Handling (`src/error/`)

**`error.rs`**: Defines comprehensive error types for all compiler phases:
- `LexError`: Lexical analysis errors (invalid characters, malformed tokens)
- `ParseError`: Syntax errors (unexpected tokens, malformed expressions)
- `SemanticError`: Semantic analysis errors (type mismatches, undefined variables)
- `CodegenError`: Code generation errors
- `IoError`: File I/O errors

Each error includes line and column information for precise error reporting.

### 2. Lexical Analysis (`src/lexer/`)

**`token.rs`**: Defines the token types supported by the language:
- **Literals**: `Integer(i64)`, `Float(f64)`, `String(String)`, `Char(char)`
- **Keywords**: `int`, `float`, `char`, `void`, `if`, `else`, `while`, `for`, `return`, `println`
- **Operators**: Arithmetic (`+`, `-`, `*`, `/`), comparison (`==`, `!=`, `<`, `>`), logical (`&&`, `||`)
- **Delimiters**: Parentheses, braces, brackets, semicolons, commas
- **Identifiers**: Variable and function names

**`lexer.rs`**: Implements the lexical analyzer that:
- Converts source code into a stream of tokens
- Handles whitespace and comments (both `//` and `/* */` styles)
- Tracks line and column numbers for error reporting
- Supports string literals with escape sequences
- Recognizes keywords vs identifiers

### 3. Syntax Analysis (`src/parser/`)

**`ast.rs`**: Defines the Abstract Syntax Tree node types:
- **Expressions (`Expr`)**:
  - Literals: `Integer`, `Float`, `Char`, `String`
  - `Identifier`: Variable references
  - `Binary`: Binary operations with left/right operands and operator
  - `Call`: Function calls with callee and arguments
- **Statements (`Stmt`)**:
  - `ExprStmt`: Expression statements
  - `VarDecl`: Variable declarations with type and optional initializer
  - `Return`: Return statements
  - `If`: Conditional statements with condition and then-branch
  - `Block`: Statement blocks
  - `Function`: Function definitions
  - `PrintStmt`: Print statements with format string and arguments

**`parser.rs`**: Implements a recursive descent parser that:
- Converts token stream into an AST
- Handles operator precedence correctly
- Supports function definitions and calls
- Parses control flow statements (`if`, `return`)
- Includes comprehensive unit tests for all language constructs

### 4. Code Generation (`src/codegen/`)

**`codegen.rs`**: Implements x86-64 assembly code generation:
- **Instruction Set**: Defines x86-64 instructions (`mov`, `add`, `sub`, `call`, etc.)
- **Register Management**: Handles x86-64 registers (`rax`, `rbp`, `rsp`, etc.)
- **Operand Types**: Immediate values, registers, memory locations, labels
- **Code Generation**:
  - Function prologues and epilogues
  - Variable storage on the stack
  - Expression evaluation with proper register usage
  - Control flow (conditional jumps)
  - Function calls with proper calling conventions
  - Built-in `println` function support

### 5. Main Application (`src/main.rs`)

Demonstrates the complete compilation pipeline:
1. **Input**: C-like source code (embedded as string literal)
2. **Lexing**: Tokenizes the source code
3. **Parsing**: Builds an AST from tokens
4. **Code Generation**: Generates x86-64 assembly
5. **Output**: Writes assembly to `output.asm`

Example input program:
```c
int main() {
    int x = 42;
    float y = 3.14;
    char c = 'a';
    println("Hello, world!\n");
    println("The integer is %d, the float is %f, and the char is %c.\n", x, y, c);
    
    if (x > 0) {
        println(x + 1);
        println("x is positive.\n");
        return x + 1;
    }
    
    return 0;
}
```

## Supported Language Features

### Data Types
- `int`: 64-bit signed integers
- `float`: 64-bit floating-point numbers
- `char`: Single characters
- `void`: For functions with no return value

### Operators
- **Arithmetic**: `+`, `-`, `*`, `/`
- **Comparison**: `==`, `!=`, `<`, `<=`, `>`, `>=`
- **Logical**: `&&`, `||`, `!`
- **Unary**: `-` (negation)

### Control Flow
- `if` statements with optional `else`
- `return` statements
- Function definitions and calls

### Built-in Functions
- `println()`: Print with newline support
- Format string support for `%d` (integers), `%f` (floats), `%c` (characters)

## Building and Running

### Prerequisites
- Rust 1.88.0 or later
- NASM (Netwide Assembler) for assembling the output
- GCC or compatible linker for creating executables

### Build
```bash
cargo build
```

### Run the Compiler
```bash
cargo run
```
This generates `output.asm` containing the x86-64 assembly code.

### Assemble and Link (Windows)
```bash
nasm -f win64 output.asm -o output.obj
gcc -o output.exe output.obj -lmsvcrt
./output.exe
```

### Run Tests
```bash
cargo test
```

## CI/CD Pipeline

The project includes a GitHub Actions workflow (`.github/workflows/ci.yml`) that:
1. **Builds** the project on Windows
2. **Runs tests** to ensure correctness
3. **Compiles and executes** the generated assembly to verify end-to-end functionality

The pipeline installs NASM and GCC, compiles the generated assembly, and executes the resulting binary.

## Architecture Decisions

### Error Handling
- Uses Rust's `Result` type for comprehensive error handling
- Custom `CompilerError` enum covers all compilation phases
- Line/column tracking for precise error reporting

### AST Design
- Expressions and statements are separate enums
- `Box<Expr>` used for recursive structures to enable heap allocation
- `PartialEq` derived for easy testing and comparison

### Code Generation
- Targets x86-64 architecture with System V ABI
- Uses stack-based variable storage
- Implements proper function calling conventions
- Generates NASM-compatible assembly syntax

### Testing
- Comprehensive unit tests for each compiler phase
- Tests cover both positive cases and error conditions
- Integration tests verify end-to-end compilation

## Future Enhancements

Potential areas for expansion:
- **More data types**: Arrays, structs, pointers
- **Advanced control flow**: `while`, `for` loops
- **Function parameters**: Currently functions take no parameters
- **Optimization**: Basic optimizations like constant folding
- **Better error recovery**: Continue parsing after errors
- **Symbol table**: Proper variable scoping and type checking
- **More target architectures**: ARM64, RISC-V support

## License

This project is a educational compiler implementation demonstrating the fundamental concepts of language design and implementation.