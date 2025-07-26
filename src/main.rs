use std::{env, fs};
use compiler_minic::codegen::{IrCodegen, TargetPlatform, parse_target_platform};
use compiler_minic::lexer::Lexer;
use compiler_minic::parser::Parser;
use compiler_minic::ir::{IrGenerator, IrOptimizer};
use compiler_minic::semantic::{MemorySafetyChecker, MemorySafetySeverity};

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // Parse target platform from command line arguments
    let target_platform = args.iter()
        .position(|arg| arg == "--target")
        .and_then(|i| args.get(i + 1))
        .and_then(|target_str| parse_target_platform(target_str).ok())
        .unwrap_or(TargetPlatform::WindowsX64);
    
    println!("Target platform: {:?}", target_platform);
    
    // Find the filename (first non-flag argument that's not a target value)
    let mut skip_next = false;
    let filename = args.iter().skip(1).find(|arg| {
        if skip_next {
            skip_next = false;
            return false;
        }
        if *arg == "--target" {
            skip_next = true;
            return false;
        }
        !arg.starts_with("--")
    });
    
    let code = if let Some(filename) = filename {
        // File argument provided
        match fs::read_to_string(filename) {
            Ok(content) => {
                println!("Compiling file: {}", filename);
                content
            }
            Err(e) => {
                eprintln!("Error reading file '{}': {}", filename, e);
                return;
            }
        }
    } else {
        // No file argument, use default code
        println!("No file provided, using default code...");
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
    };

    let mut lexer = Lexer::new(&code);
    match lexer.tokenize() {
        Ok(tokens) => {
            let mut parser = Parser::new(tokens);
            let ast = parser.parse();
            
            for error in parser.get_errors() {
                eprintln!("Parser error: {}", error);
            }
            
            let mut memory_checker = MemorySafetyChecker::new();
            match memory_checker.check_memory_safety(&ast) {
                Ok(warnings) => {
                    for warning in warnings {
                        match warning.severity() {
                            MemorySafetySeverity::Error => {
                                eprintln!("Memory safety error: {}", warning.message());
                            }
                            MemorySafetySeverity::Warning => {
                                println!("Memory safety warning: {}", warning.message());
                            }
                            MemorySafetySeverity::Info => {
                                println!("Memory safety info: {}", warning.message());
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Memory safety analysis error: {}", e);
                }
            }
            
            // Generate IR from AST
            let mut ir_generator = IrGenerator::new();
            let ir_program = match ir_generator.generate(&ast) {
                Ok(program) => program,
                Err(e) => {
                    eprintln!("IR generation failed: {:?}", e);
                    return;
                }
            };

            // Save IR to file for inspection
            match fs::write("output.ir", format!("{}", ir_program)) {
                Ok(_) => println!("IR code saved to output.ir"),
                Err(e) => eprintln!("Error writing IR file: {}", e),
            }

            // Optimize IR
            let mut optimizer = IrOptimizer::new();
            let optimized_ir = optimizer.optimize(ir_program);

            // Save optimized IR to file
            match fs::write("output_optimized.ir", format!("{}", optimized_ir)) {
                Ok(_) => println!("Optimized IR code saved to output_optimized.ir"),
                Err(e) => eprintln!("Error writing optimized IR file: {}", e),
            }

            // Generate assembly from IR with target platform
            let ir_codegen = IrCodegen::new_with_target(target_platform);
            let asm_code = ir_codegen.generate(&optimized_ir);

            match fs::write("output.asm", asm_code) {
                Ok(_) => println!("Assembly code (from IR) saved to output.asm"),
                Err(e) => eprintln!("Error writing assembly file: {}", e),
            }
        }
        Err(e) => {
            eprintln!("Lexing error: {}", e);
        }
    }
}
