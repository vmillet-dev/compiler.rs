use std::fs;
use compiler_minic::codegen::Codegen;
use compiler_minic::lexer::Lexer;
use compiler_minic::parser::Parser;

fn main() {
    let code = r#"
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

        /* Commentaire bloc */
        // Commentaire ligne

        return 0;
    }
    "#;

    let mut lexer = Lexer::new(code);
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
