use compiler_minic::lexer::Lexer;
use compiler_minic::parser::Parser;

fn main() {
    let code = r#"
    int main() {
        int x = 42;
        float y = 3.14;
        char c = 'a';

        if (x > 0) {
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
            println!("{:#?}", ast);
        }
        Err(e) => {
            eprintln!("Erreur de lexing: {}", e);
        }
    }
}
