use compiler_minic::{lexer::Lexer, parser::Parser, ir::generator::IrGenerator, codegen::{Codegen, IrCodegen}};

#[cfg(test)]
mod ir_integration_tests {
    use super::*;

    fn compile_both_ways(source: &str) -> (String, String, String, String) {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().expect("Tokenization failed");
        let mut parser = Parser::new(tokens);
        let ast = parser.parse();

        let direct_codegen = Codegen::new();
        let direct_asm = direct_codegen.generate(&ast);

        let mut ir_generator = IrGenerator::new();
        let ir_program = ir_generator.generate(&ast);
        let ir_output = format!("{}", ir_program);
        
        let ir_codegen = IrCodegen::new();
        let ir_asm = ir_codegen.generate(&ir_program);

        (direct_asm, ir_asm, ir_output, source.to_string())
    }

    fn validate_ir_structure(ir_output: &str, expected_elements: &[&str]) {
        for element in expected_elements {
            assert!(ir_output.contains(element), 
                "IR output missing expected element: {}\nFull IR:\n{}", element, ir_output);
        }
    }

    fn validate_asm_structure(asm_output: &str, expected_instructions: &[&str]) {
        for instruction in expected_instructions {
            assert!(asm_output.contains(instruction),
                "Assembly output missing expected instruction: {}\nFull ASM:\n{}", instruction, asm_output);
        }
    }

    #[test]
    fn test_simple_variable_declaration() {
        let source = r#"
int main() {
    int x = 42;
    return x;
}
"#;

        let (direct_asm, ir_asm, ir_output, _) = compile_both_ways(source);

        validate_ir_structure(&ir_output, &[
            "define i32 @main()",
            "%x = alloca i32",
            "store i32 42, %x",
            "%t0 = load i32, %x",
            "ret i32 %t0"
        ]);

        validate_asm_structure(&direct_asm, &["mov", "rbp"]);
        validate_asm_structure(&ir_asm, &["mov", "rbp"]);

        assert!(!direct_asm.is_empty());
        assert!(!ir_asm.is_empty());
        assert!(direct_asm.contains("main:"));
        assert!(ir_asm.contains("main:"));
    }

    #[test]
    fn test_binary_arithmetic_operations() {
        let source = r#"
int main() {
    int a = 10;
    int b = 20;
    int sum = a + b;
    int diff = a - b;
    int prod = a * b;
    return sum;
}
"#;

        let (direct_asm, ir_asm, ir_output, _) = compile_both_ways(source);

        validate_ir_structure(&ir_output, &[
            "%a = alloca i32",
            "%b = alloca i32", 
            "%sum = alloca i32",
            "add i32",
            "sub i32", 
            "mul i32"
        ]);

        validate_asm_structure(&direct_asm, &["add", "sub", "imul"]);
        validate_asm_structure(&ir_asm, &["add", "sub", "imul"]);
    }

    #[test]
    fn test_conditional_statements() {
        let source = r#"
int main() {
    int x = 15;
    if (x > 10) {
        x = x + 5;
    }
    return x;
}
"#;

        let (direct_asm, ir_asm, ir_output, _) = compile_both_ways(source);

        validate_ir_structure(&ir_output, &[
            "gt i32",
            "br %t1, label %if_then",
            "if_then_0:",
            "if_end_1:"
        ]);

        validate_asm_structure(&direct_asm, &["cmp"]);
        validate_asm_structure(&ir_asm, &["setg", "je"]);
    }

    #[test]
    fn test_print_statements() {
        let source = r#"
int main() {
    int value = 42;
    println("Value is:");
    println(value);
    return 0;
}
"#;

        let (direct_asm, ir_asm, ir_output, _) = compile_both_ways(source);

        validate_ir_structure(&ir_output, &[
            "print \"str_",
            "@str_0 = constant str \"Value is:\""
        ]);

        validate_asm_structure(&direct_asm, &["call     printf"]);
        validate_asm_structure(&ir_asm, &["call     printf"]);
    }

    #[test]
    fn test_function_with_multiple_types() {
        let source = r#"
int main() {
    int number = 42;
    float pi = 3.14;
    char letter = 'A';
    println(number);
    println(pi);
    println(letter);
    return 0;
}
"#;

        let (direct_asm, ir_asm, ir_output, _) = compile_both_ways(source);

        validate_ir_structure(&ir_output, &[
            "%number = alloca i32",
            "%pi = alloca f64", 
            "%letter = alloca i8",
            "store i32 42",
            "store f64 3.14",
            "store i8 'A'"
        ]);

        assert!(direct_asm.contains("dword") || direct_asm.contains("qword"));
        assert!(ir_asm.contains("dword") || ir_asm.contains("qword"));
    }

    #[test]
    fn test_complex_expressions() {
        let source = r#"
int main() {
    int a = 5;
    int b = 10;
    int result = (a + b) * 2 - a;
    return result;
}
"#;

        let (direct_asm, ir_asm, ir_output, _) = compile_both_ways(source);

        validate_ir_structure(&ir_output, &[
            "%t0 =",
            "%t1 =", 
            "%t2 =",
            "add i32",
            "mul i32",
            "sub i32"
        ]);

        validate_asm_structure(&direct_asm, &["add", "imul", "sub"]);
        validate_asm_structure(&ir_asm, &["add", "imul", "sub"]);
    }

    #[test]
    fn test_unary_operations() {
        let source = r#"
int main() {
    int x = 10;
    int neg = -x;
    return neg;
}
"#;

        let (direct_asm, ir_asm, ir_output, _) = compile_both_ways(source);

        validate_ir_structure(&ir_output, &[
            "neg i32"
        ]);

        validate_asm_structure(&direct_asm, &["neg"]);
        validate_asm_structure(&ir_asm, &["neg"]);
    }

    #[test]
    fn test_string_literals() {
        let source = r#"
int main() {
    println("Hello, World!");
    println("Testing string literals");
    return 0;
}
"#;

        let (direct_asm, ir_asm, ir_output, _) = compile_both_ways(source);

        validate_ir_structure(&ir_output, &[
            "@str_0 = constant str \"Hello, World!\"",
            "@str_1 = constant str \"Testing string literals\""
        ]);

        assert!(direct_asm.contains("section .data") || direct_asm.contains("Hello, World!"));
        assert!(ir_asm.contains("section .data") || ir_asm.contains("Hello, World!"));
    }
}
