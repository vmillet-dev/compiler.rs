use std::fs;
use std::path::PathBuf;
use std::process;

use clap::Parser;
use compiler_minic::codegen::{Codegen, TargetPlatform, parse_target_platform};
use compiler_minic::lexer::Lexer;
use compiler_minic::parser::Parser as MiniCParser;
use compiler_minic::ir::{IrGenerator, IrOptimizer};
use compiler_minic::semantic::{MemorySafetyChecker, MemorySafetySeverity};

/// MiniC Compiler - A simple C-like language compiler
#[derive(Parser)]
#[command(name = "minic")]
#[command(about = "A compiler for the MiniC language")]
#[command(version = "0.1.0")]
struct Cli {
    /// Input source file to compile
    #[arg(value_name = "FILE")]
    input: Option<PathBuf>,

    /// Target platform for code generation
    #[arg(short, long, default_value = "windows-x64")]
    target: String,

    /// Output directory for generated files
    #[arg(short, long, default_value = "build")]
    output_dir: PathBuf,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Skip memory safety checks
    #[arg(long)]
    skip_memory_checks: bool,

    /// Skip IR optimization
    #[arg(long)]
    skip_optimization: bool,
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run_compiler(cli) {
        eprintln!("Compilation failed: {}", e);
        process::exit(1);
    }
}

fn run_compiler(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    // Parse target platform
    let target_platform = parse_target_platform(&cli.target)
        .map_err(|_| format!("Invalid target platform: {}", cli.target))?;

    if cli.verbose {
        println!("Target platform: {:?}", target_platform);
        println!("Output directory: {:?}", cli.output_dir);
    }

    // Read source code
    let code = read_source_code(&cli)?;

    // Compile the code
    compile_code(&code, target_platform, &cli)
}

fn read_source_code(cli: &Cli) -> Result<String, Box<dyn std::error::Error>> {
    match &cli.input {
        Some(filename) => {
            if cli.verbose {
                println!("Compiling file: {:?}", filename);
            }
            fs::read_to_string(filename)
                .map_err(|e| format!("Error reading file '{:?}': {}", filename, e).into())
        }
        None => {
            if cli.verbose {
                println!("No file provided, using default code...");
            }
            Ok(get_default_code())
        }
    }
}

fn get_default_code() -> String {
    r#"
    int main() {
        int number = 42;
        float pi = 3.14159;
        char letter = 'A';

        int sum = number + 10;
        float area = pi * 5.0 * 5.0;

        if (number > 40) {
            println("Number is greater than 40: %d", number);
        }

        if (pi > 3.0) {
            println("Pi approximation: %.3f", pi);
        }

        int complex_calc = (number * 2) + (sum - 15);
        float ratio = area / (pi + 1.0);

        letter = 'Z';
        number = complex_calc;

        println("Final results:");
        println("Number: %d, Letter: %c", number, letter);
        println("Area: %.2f, Ratio: %.4f", area, ratio);

        if (complex_calc > 50) {
            if (letter == 'Z') {
                println("Complex condition met!");
            }
        }

        return 0;
    }

    int helper_function() {
        int local_var = 100;
        println("Helper function called with local: %d", local_var);
        return local_var;
    }

    float math_function() {
        float result = 2.718;
        if (result > 2.0) {
            result = result * 1.5;
        }
        return result;
    }
    "#.to_string()
}

fn compile_code(
    code: &str,
    target_platform: TargetPlatform,
    cli: &Cli,
) -> Result<(), Box<dyn std::error::Error>> {

    // Tokenization
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize()
        .map_err(|e| format!("Lexing error: {}", e))?;

    if cli.verbose {
        println!("Tokenization completed successfully");
    }

    // Parsing
    let mut parser = MiniCParser::new(tokens);
    let ast = parser.parse();

    // Check for parser errors
    let parser_errors = parser.get_errors();
    if !parser_errors.is_empty() {
        for error in parser_errors {
            eprintln!("Parser error: {}", error);
        }
        return Err("Parsing failed with errors".into());
    }

    if cli.verbose {
        println!("Parsing completed successfully");
    }

    // Memory safety analysis (if not skipped)
    if !cli.skip_memory_checks {
        run_memory_safety_analysis(&ast, cli.verbose)?;
    }

    // IR generation
    let ir_program = generate_ir(&ast, cli.verbose)?;

    // Save IR to file
    save_ir_to_file(&ir_program, &cli.output_dir, "output.ir", cli.verbose)?;

    // IR optimization (if not skipped)
    let final_ir = if cli.skip_optimization {
        if cli.verbose {
            println!("Skipping IR optimization");
        }
        ir_program
    } else {
        let optimized_ir = optimize_ir(ir_program, cli.verbose)?;
        save_ir_to_file(&optimized_ir, &cli.output_dir, "output_optimized.ir", cli.verbose)?;
        optimized_ir
    };

    // Code generation
    generate_assembly(&final_ir, target_platform, &cli.output_dir, cli.verbose)?;

    if cli.verbose {
        println!("Compilation completed successfully!");
    }

    Ok(())
}

fn run_memory_safety_analysis(
    ast: &[compiler_minic::parser::ast::Stmt],
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        println!("Running memory safety analysis...");
    }

    let mut memory_checker = MemorySafetyChecker::new();
    let warnings = memory_checker.check_memory_safety(ast)
        .map_err(|e| format!("Memory safety analysis error: {}", e))?;

    for warning in warnings {
        match warning.severity() {
            MemorySafetySeverity::Error => {
                eprintln!("Memory safety error: {}", warning.message());
            }
            MemorySafetySeverity::Warning => {
                println!("Memory safety warning: {}", warning.message());
            }
            MemorySafetySeverity::Info => {
                if verbose {
                    println!("Memory safety info: {}", warning.message());
                }
            }
        }
    }

    Ok(())
}

fn generate_ir(
    ast: &[compiler_minic::parser::ast::Stmt],
    verbose: bool,
) -> Result<compiler_minic::ir::IrProgram, Box<dyn std::error::Error>> {
    if verbose {
        println!("Generating IR...");
    }

    let mut ir_generator = IrGenerator::new();
    ir_generator.generate(ast)
        .map_err(|e| format!("IR generation failed: {e:?}").into())
}

fn optimize_ir(
    ir_program: compiler_minic::ir::IrProgram,
    verbose: bool,
) -> Result<compiler_minic::ir::IrProgram, Box<dyn std::error::Error>> {
    if verbose {
        println!("Optimizing IR...");
    }

    let mut optimizer = IrOptimizer::new();
    Ok(optimizer.optimize(ir_program))
}

fn save_ir_to_file(
    ir_program: &compiler_minic::ir::IrProgram,
    output_dir: &PathBuf,
    filename: &str,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create output directory if it doesn't exist
    fs::create_dir_all(output_dir)
        .map_err(|e| format!("Error creating output directory '{output_dir:?}': {e}"))?;

    let output_path = output_dir.join(filename);
    fs::write(&output_path, format!("{ir_program}"))
        .map_err(|e| format!("Error writing IR file '{output_path:?}': {e}"))?;

    if verbose {
        println!("IR code saved to {output_path:?}");
    }

    Ok(())
}

fn generate_assembly(
    ir_program: &compiler_minic::ir::IrProgram,
    target_platform: TargetPlatform,
    output_dir: &PathBuf,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        println!("Generating assembly code...");
    }

    // Create output directory if it doesn't exist
    fs::create_dir_all(output_dir)
        .map_err(|e| format!("Error creating output directory '{output_dir:?}': {e}"))?;

    let ir_codegen = Codegen::new_with_target(target_platform);
    let asm_code = ir_codegen.generate(ir_program);

    let output_path = output_dir.join("output.asm");
    fs::write(&output_path, asm_code)
        .map_err(|e| format!("Error writing assembly file '{output_path:?}': {e}"))?;

    if verbose {
        println!("Assembly code saved to {output_path:?}");
    }

    Ok(())
}
