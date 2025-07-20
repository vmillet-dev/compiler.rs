use compiler_minic::lexer::Lexer;

fn main() {
    let code = r#"
    int main() {
        int x = 42;
        float y = 3.14;
        char c = 'a';

        if (x > 0) {
            return x + 1;
        }

        return 0;
    }
    "#;

    let mut lexer = Lexer::new(code);
    match lexer.tokenize() {
        Ok(tokens) => {
            for token in tokens {
                println!("{}", token);
            }
        }
        Err(e) => {
            eprintln!("Erreur de lexing: {}", e);
        }
    }
}
