# MiniC Compiler CLI Usage

The MiniC compiler has been refactored with a clean, modern CLI interface using `clap`.

## Basic Usage

```bash
# Compile with default settings (uses built-in example code)
cargo run

# Compile a specific file
cargo run -- input.c

# Show help
cargo run -- --help
```

## Command Line Options

### Input File
```bash
# Compile a specific source file
cargo run -- test_simple.c
```

### Target Platform
```bash
# Specify target platform (default: windows-x64)
cargo run -- --target linux-x64 input.c
cargo run -- --target windows-x64 input.c
cargo run -- -t macos-arm64 input.c
```

### Output Directory
```bash
# Specify output directory (default: current directory)
cargo run -- --output-dir ./build input.c
cargo run -- -o ./output input.c
```

### Verbose Output
```bash
# Enable detailed compilation output
cargo run -- --verbose input.c
cargo run -- -v input.c
```

### Skip Options
```bash
# Skip memory safety checks
cargo run -- --skip-memory-checks input.c

# Skip IR optimization
cargo run -- --skip-optimization input.c

# Combine multiple options
cargo run -- --verbose --skip-memory-checks --skip-optimization --output-dir ./build input.c
```

## Examples

### Basic compilation with verbose output:
```bash
cargo run -- --verbose test_simple.c
```

### Cross-compilation for Linux with custom output directory:
```bash
cargo run -- --target linux-x64 --output-dir ./linux_build --verbose input.c
```

### Fast compilation (skip optimizations and memory checks):
```bash
cargo run -- --skip-optimization --skip-memory-checks --output-dir ./debug input.c
```

## Output Files

The compiler generates the following files in the output directory:

- `output.ir` - Intermediate representation (IR) code
- `output_optimized.ir` - Optimized IR code (if optimization is enabled)
- `output.asm` - Generated assembly code

## Improvements Made

1. **Clean CLI Interface**: Using `clap` for professional command-line argument parsing
2. **Better Error Handling**: Proper error propagation with descriptive messages
3. **Modular Code Structure**: Separated concerns into focused functions
4. **Automatic Directory Creation**: Output directories are created automatically
5. **Flexible Options**: Skip memory checks or optimization for faster compilation
6. **Verbose Mode**: Detailed output for debugging and monitoring compilation progress
7. **Clippy Compliance**: Applied Clippy suggestions for better Rust code quality

## Code Quality Improvements

- Used `&[T]` instead of `&Vec<T>` for function parameters (more idiomatic)
- Applied inline format arguments for better performance
- Proper error handling with `Result` types
- Separated compilation phases into individual functions
- Added comprehensive CLI documentation and help text