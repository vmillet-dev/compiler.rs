# Intermediate Representation (IR) Implementation

## Overview

This document describes the IR (Intermediate Representation) implementation added to the Mini-C compiler. The IR serves as an intermediate step between the Abstract Syntax Tree (AST) and assembly code generation.

## Architecture

The compiler now supports two compilation pipelines:

1. **Direct AST-to-Assembly**: `cargo run [file.c]`
2. **IR-based compilation**: `cargo run [file.c] --ir`

## IR Components

### 1. IR Types (`src/ir/ir.rs`)
- `IrType`: Represents data types (Int, Float, Char, Void)
- `IrValue`: Represents operands (constants, locals, temporaries, parameters, globals)
- `IrBinaryOp`: Binary operations (Add, Sub, Mul, Div, etc.)
- `IrUnaryOp`: Unary operations (Neg, Not)
- `IrInstruction`: IR instruction set
- `IrFunction`: Function representation in IR
- `IrProgram`: Complete program representation

### 2. IR Generation (`src/ir/generator.rs`)
- `IrGenerator`: Converts AST to IR
- Handles variable declarations, expressions, statements
- Manages temporary variables and labels
- Generates structured control flow

### 3. IR Optimization (`src/ir/optimizer.rs`)
- `IrOptimizer`: Performs basic optimizations
- Currently implements constant folding
- Extensible for additional optimizations

### 4. IR-to-Assembly Code Generation (`src/codegen/ir_codegen.rs`)
- `IrCodegen`: Converts IR to x86-64 assembly
- Handles register allocation and stack management
- Generates Windows x64 calling convention compliant code

## IR Instruction Set

### Memory Operations
- `Alloca`: Allocate stack space for variables
- `Store`: Store value to memory location
- `Load`: Load value from memory location
- `Move`: Move value between locations

### Arithmetic Operations
- `BinaryOp`: Binary arithmetic and logical operations
- `UnaryOp`: Unary operations (negation, logical not)

### Control Flow
- `Branch`: Conditional branching
- `Jump`: Unconditional jump
- `Label`: Jump target
- `Return`: Function return

### Function Operations
- `Call`: Function call
- `Print`: Built-in print function

## Example IR Output

For the input C code:
```c
int main() {
    int x = 10;
    int y = 20;
    int result = x + y * 2;
    
    if (result > 30) {
        println("Result is greater than 30");
        println(result);
    }
    
    return result;
}
```

The generated IR is:
```llvm
define i32 @main() {
entry:
  %x = alloca i32
  store i32 10, %x
  %y = alloca i32
  store i32 20, %y
  %result = alloca i32
  %t0 = load i32, %x
  %t1 = load i32, %y
  %t2 = mul i32 %t1, 2
  %t3 = add i32 %t0, %t2
  store i32 %t3, %result
  %t4 = load i32, %result
  %t5 = gt i32 %t4, 30
  br %t5, label %if_then_0, label %if_end_1
if_then_0:
  print "str_0", []
  %t6 = load i32, %result
  print "str_1", [%t6]
  jmp label %if_end_1
if_end_1:
  %t7 = load i32, %result
  ret i32 %t7
}
```

## Benefits of IR

1. **Separation of Concerns**: Separates language semantics from target architecture
2. **Optimization Opportunities**: Enables machine-independent optimizations
3. **Multiple Backends**: Easy to add support for different target architectures
4. **Debugging**: IR provides a human-readable intermediate representation
5. **Analysis**: Enables sophisticated program analysis

## Usage

### Compile with IR pipeline:
```bash
cargo run test_ir.c --ir
```

This generates:
- `output.ir`: Original IR code
- `output_optimized.ir`: Optimized IR code
- `output_ir.asm`: Generated assembly from IR

### Assemble and link:
```bash
nasm -f win64 output_ir.asm -o output.o
gcc -o output.exe output.o
./output.exe
```

## Future Enhancements

1. **Advanced Optimizations**:
   - Dead code elimination
   - Common subexpression elimination
   - Loop optimizations
   - Register allocation improvements

2. **Additional IR Instructions**:
   - Array operations
   - Structure operations
   - Function pointers

3. **Multiple Backends**:
   - ARM64 support
   - RISC-V support
   - LLVM backend integration

4. **Analysis Passes**:
   - Type checking at IR level
   - Memory safety analysis
   - Performance profiling integration