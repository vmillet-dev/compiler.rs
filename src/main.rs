use std::{env, fs};
use compiler_minic::codegen::Codegen;
use compiler_minic::lexer::Lexer;
use compiler_minic::parser::Parser;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let code = if args.len() > 1 {
        // File argument provided
        let filename = &args[1];
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

        println("Testing simple println with different types:");
        println(number);
        println(pi);
        println(letter);

        println("Testing with expressions:");
        println(number * 2);

        return 0;
    }
    "#.to_string()
    };

    let mut lexer = Lexer::new(&code);
    match lexer.tokenize() {
        Ok(tokens) => {
            let mut parser = Parser::new(tokens);
            let ast = parser.parse();
            let codegen = Codegen::new();
            let asm_code = codegen.generate(&ast);

            match fs::write("output.asm", asm_code) {
                Ok(_) => println!("Code assembleur sauvegardé dans output.asm"),
                Err(e) => eprintln!("Erreur lors de l'écriture du fichier: {}", e),
            }
        }
        Err(e) => {
            eprintln!("Erreur de lexing: {}", e);
        }
    }
}
