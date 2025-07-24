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
│   ├── ir/                  # Intermediate Representation
│   │   ├── mod.rs           # IR module exports
│   │   ├── ir.rs            # IR instruction definitions
│   │   ├── generator.rs     # AST to IR translation
│   │   └── optimizer.rs     # IR optimization passes
│   └── codegen/             # Code generation
│       ├── mod.rs           # Codegen module exports
│       ├── codegen.rs       # Direct x86-64 assembly code generator
│       ├── ir_codegen.rs    # IR to x86-64 assembly code generator
│       ├── analyzer.rs      # AST analysis for variable types
│       ├── expression.rs    # Expression code generation
│       ├── statement.rs     # Statement code generation
│       ├── emitter.rs       # Assembly instruction emission
│       └── instruction.rs   # x86-64 instruction definitions
├── tests/
│   └── integration_tests.rs # Integration tests for compilation pipeline
├── .github/
│   └── workflows/
│       └── ci.yml           # GitHub Actions CI/CD pipeline
├── Cargo.toml               # Rust project configuration
├── .gitignore               # Git ignore rules
├── output.asm               # Generated assembly output (direct compilation)
└── output_ir.asm            # Generated assembly output (IR compilation)
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

### 4. Intermediate Representation (`src/ir/`)

**`ir.rs`**: Defines the IR instruction set and data structures:
- **IR Instructions**: Load, store, arithmetic operations, control flow, function calls
- **IR Values**: Temporary variables, constants, memory locations
- **IR Types**: Integer, float, char, void type representations
- **IR Program**: Complete program representation with functions and basic blocks

**`generator.rs`**: Implements AST to IR translation:
- Converts AST expressions to IR instruction sequences
- Handles variable declarations and assignments
- Translates control flow statements to IR branches
- Manages temporary variable allocation
- Supports function definitions and calls

**`optimizer.rs`**: Implements IR optimization passes:
- **Constant Folding**: Evaluates constant expressions at compile time
- **Dead Code Elimination**: Removes unreachable code and unused variables
- **Copy Propagation**: Replaces variable copies with direct references
- **Basic Block Optimization**: Optimizes within basic blocks

### 5. Code Generation (`src/codegen/`)

**`codegen.rs`**: Implements direct x86-64 assembly code generation:
- **Instruction Set**: Defines x86-64 instructions (`mov`, `add`, `sub`, `call`, etc.)
- **Register Management**: Handles x86-64 registers (`rax`, `rbp`, `rsp`, etc.)
- **Operand Types**: Immediate values, registers, memory locations, labels
- **Direct Code Generation**: AST to assembly without intermediate representation

**`ir_codegen.rs`**: Implements IR to x86-64 assembly translation:
- Converts optimized IR instructions to x86-64 assembly
- Handles register allocation for IR temporary variables
- Implements proper calling conventions for IR function calls
- Manages stack layout for IR variable storage

**`analyzer.rs`**: Provides AST analysis utilities:
- Variable type inference and collection
- Format string analysis for print statements
- Symbol table management for code generation

**`expression.rs`**, **`statement.rs`**: Modular code generation:
- Expression evaluation with proper register usage
- Statement translation (declarations, control flow, returns)
- Function prologues and epilogues
- Built-in `println` function support

**`emitter.rs`**, **`instruction.rs`**: Assembly output infrastructure:
- Structured assembly instruction emission
- x86-64 instruction and operand definitions
- Comment generation for readable assembly output

### 6. Main Application (`src/main.rs`)

Demonstrates the complete compilation pipeline with two modes:

**Direct Compilation (default)**:
1. **Input**: C-like source code (embedded as string literal or file)
2. **Lexing**: Tokenizes the source code
3. **Parsing**: Builds an AST from tokens
4. **Direct Code Generation**: Generates x86-64 assembly directly from AST
5. **Output**: Writes assembly to `output.asm`

**IR-Based Compilation (--ir flag)**:
1. **Input**: C-like source code (embedded as string literal or file)
2. **Lexing**: Tokenizes the source code
3. **Parsing**: Builds an AST from tokens
4. **IR Generation**: Converts AST to intermediate representation
5. **IR Optimization**: Applies optimization passes (constant folding, dead code elimination)
6. **IR Code Generation**: Translates optimized IR to x86-64 assembly
7. **Output**: Writes assembly to `output_ir.asm`

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

#### Direct Compilation (Default)
```bash
cargo run
```
This generates `output.asm` containing the x86-64 assembly code using direct AST-to-assembly translation.

#### IR-Based Compilation
```bash
cargo run -- --ir
```
This generates `output_ir.asm` containing the x86-64 assembly code using an intermediate representation (IR) pipeline:
1. **AST → IR**: Converts the Abstract Syntax Tree to intermediate representation
2. **IR Optimization**: Performs optimizations like constant folding and dead code elimination  
3. **IR → Assembly**: Translates optimized IR to x86-64 assembly

The IR pipeline provides better optimization opportunities and cleaner code generation architecture.

### Assemble and Link (Windows)
```bash
nasm -f win64 output.asm -o output.obj
gcc -o output.exe output.obj -lmsvcrt
./output.exe
```

### Run Tests

#### Unit Tests
```bash
cargo test
```
Runs unit tests for lexer, parser, and code generation components.

#### Integration Tests
```bash
cargo test --test integration_tests
```
Runs comprehensive integration tests that validate the complete compilation pipeline by:
- Compiling actual C code snippets through both direct and IR-based compilation paths
- Validating IR output structure and instruction patterns
- Verifying generated assembly contains expected instructions
- Ensuring both compilation modes produce functionally equivalent results

The integration test suite covers:
- Variable declarations and assignments
- Binary and unary arithmetic operations
- Conditional statements and control flow
- Print statements with format strings
- Multiple data types (int, float, char)
- Complex nested expressions
- Block statements and variable scoping
- Comparison operators
- Expression statements

**Integration Test Structure:**
Each test compiles a minimal C code snippet and validates:
1. **IR Structure**: Checks for expected IR instructions (e.g., `%x = alloca i32`, `add i32`, `br %t1`)
2. **Assembly Output**: Verifies both direct and IR-generated assembly contain expected x86-64 instructions
3. **Functional Equivalence**: Ensures both compilation paths produce working assembly code

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
