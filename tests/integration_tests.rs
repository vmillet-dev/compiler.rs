use compiler_minic::{lexer::Lexer, parser::Parser, ir::generator::IrGenerator, codegen::{Codegen}};

#[cfg(test)]
mod ir_integration_tests {
    use super::*;

    fn compile_both_ways(source: &str) -> (String, String, String, String) {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().expect("Tokenization failed");
        let mut parser = Parser::new(tokens);
        let ast = parser.parse();

        let mut ir_generator = IrGenerator::new();
        let ir_program = ir_generator.generate(&ast).expect("IR generation should succeed");
        let ir_output = format!("{}", ir_program);
        
        let ir_codegen = Codegen::new();
        let ir_asm = ir_codegen.generate(&ir_program);

        // For now, we only have IR-based compilation, so we return the same assembly for both
        // The first return value is kept for backward compatibility but is the same as the second
        (ir_asm.clone(), ir_asm, ir_output, source.to_string())
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

    #[test]
    fn test_assignment_expressions() {
        let source = r#"
int main() {
    int x = 10;
    int y = 20;
    x = y;
    y = x + 5;
    return x;
}
"#;

        let (direct_asm, ir_asm, ir_output, _) = compile_both_ways(source);

        validate_ir_structure(&ir_output, &[
            "%x = alloca i32",
            "%y = alloca i32",
            "store i32",
            "load i32"
        ]);

        validate_asm_structure(&direct_asm, &["mov"]);
        validate_asm_structure(&ir_asm, &["mov"]);
    }

    #[test]
    fn test_nested_expressions() {
        let source = r#"
int main() {
    int a = 2;
    int b = 3;
    int c = 4;
    int result = ((a + b) * c) - (a * (b + c));
    return result;
}
"#;

        let (direct_asm, ir_asm, ir_output, _) = compile_both_ways(source);

        validate_ir_structure(&ir_output, &[
            "%t0 =",
            "%t1 =",
            "%t2 =",
            "%t3 =",
            "%t4 =",
            "add i32",
            "mul i32",
            "sub i32"
        ]);

        validate_asm_structure(&direct_asm, &["add", "imul", "sub"]);
        validate_asm_structure(&ir_asm, &["add", "imul", "sub"]);
    }

    #[test]
    fn test_mixed_data_types() {
        let source = r#"
int main() {
    int i = 42;
    float f = 3.14;
    char c = 'X';
    int sum = i + (int)f;
    return sum;
}
"#;

        let (direct_asm, ir_asm, ir_output, _) = compile_both_ways(source);

        validate_ir_structure(&ir_output, &[
            "%i = alloca i32",
            "%f = alloca f64",
            "%c = alloca i8",
            "store i32 42",
            "store f64 3.14",
            "store i8 'X'"
        ]);

        assert!(direct_asm.contains("dword") || direct_asm.contains("qword"));
        assert!(ir_asm.contains("dword") || ir_asm.contains("qword"));
    }

    #[test]
    fn test_block_statements() {
        let source = r#"
int main() {
    int x = 10;
    {
        int y = 20;
        x = x + y;
    }
    return x;
}
"#;

        let (direct_asm, ir_asm, ir_output, _) = compile_both_ways(source);

        validate_ir_structure(&ir_output, &[
            "%x = alloca i32",
            "store i32 10",
            "store i32 20"
        ]);

        validate_asm_structure(&direct_asm, &["mov", "add"]);
        validate_asm_structure(&ir_asm, &["mov", "add"]);
    }

    #[test]
    fn test_multiple_return_paths() {
        let source = r#"
int main() {
    int x = 15;
    if (x > 10) {
        return x + 1;
    }
    return x - 1;
}
"#;

        let (direct_asm, ir_asm, ir_output, _) = compile_both_ways(source);

        validate_ir_structure(&ir_output, &[
            "gt i32",
            "br %t1, label %if_then",
            "ret i32",
            "if_then_0:",
            "if_end_1:"
        ]);

        validate_asm_structure(&direct_asm, &["cmp", "ret"]);
        validate_asm_structure(&ir_asm, &["setg", "je", "ret"]);
    }

    #[test]
    fn test_comparison_operators() {
        let source = r#"
int main() {
    int a = 10;
    int b = 20;
    int eq = (a == b);
    int ne = (a != b);
    int lt = (a < b);
    int le = (a <= b);
    int gt = (a > b);
    int ge = (a >= b);
    return eq + ne + lt + le + gt + ge;
}
"#;

        let (direct_asm, ir_asm, ir_output, _) = compile_both_ways(source);

        validate_ir_structure(&ir_output, &[
            "eq i32",
            "ne i32", 
            "lt i32",
            "le i32",
            "gt i32",
            "ge i32"
        ]);

        validate_asm_structure(&direct_asm, &["cmp"]);
        validate_asm_structure(&ir_asm, &["cmp"]);
    }

    #[test]
    fn test_format_string_with_multiple_args() {
        let source = r#"
int main() {
    int num = 42;
    float pi = 3.14;
    char letter = 'A';
    println("Number: %d, Pi: %f, Letter: %c", num, pi, letter);
    return 0;
}
"#;

        let (direct_asm, ir_asm, ir_output, _) = compile_both_ways(source);

        validate_ir_structure(&ir_output, &[
            "print \"str_",
            "@str_0 = constant str \"Number: %d, Pi: %f, Letter: %c\""
        ]);

        validate_asm_structure(&direct_asm, &["call     printf"]);
        validate_asm_structure(&ir_asm, &["call     printf"]);
    }

    #[test]
    fn test_expression_statements() {
        let source = r#"
int main() {
    int x = 5;
    x + 10;
    x * 2;
    -x;
    return x;
}
"#;

        let (direct_asm, ir_asm, ir_output, _) = compile_both_ways(source);

        validate_ir_structure(&ir_output, &[
            "%x = alloca i32",
            "store i32 5",
            "add i32",
            "mul i32",
            "neg i32"
        ]);

        validate_asm_structure(&direct_asm, &["mov"]);
        validate_asm_structure(&ir_asm, &["mov"]);
    }

    #[test]
    fn test_variable_shadowing() {
        let source = r#"
int main() {
    int x = 10;
    {
        int x = 20;
        println(x);
    }
    println(x);
    return 0;
}
"#;

        let (direct_asm, ir_asm, ir_output, _) = compile_both_ways(source);

        validate_ir_structure(&ir_output, &[
            "%x = alloca i32",
            "store i32 10",
            "store i32 20"
        ]);

        validate_asm_structure(&direct_asm, &["mov"]);
        validate_asm_structure(&ir_asm, &["mov"]);
    }

    #[test]
    fn test_function_with_parameters() {
        let source = r#"
int add(int a, int b) {
    return a + b;
}
"#;

        let (direct_asm, ir_asm, ir_output, _) = compile_both_ways(source);

        validate_ir_structure(&ir_output, &[
            "define i32 @add(i32 %a, i32 %b)",
            "load i32, %a",
            "load i32, %b",
            "add i32"
        ]);

        validate_asm_structure(&ir_asm, &["add:", "mov", "add"]);
    }

    #[test]
    fn test_logical_and_operator() {
        let source = r#"
int main() {
    int a = 1;
    int b = 0;
    int result = a && b;
    return result;
}
"#;

        let (direct_asm, ir_asm, ir_output, _) = compile_both_ways(source);

        validate_ir_structure(&ir_output, &[
            "and_false_",
            "and_end_",
            "and_eval_right_",
            "br %t"
        ]);

        validate_asm_structure(&ir_asm, &["je", "jmp"]);
    }

    #[test]
    fn test_logical_or_operator() {
        let source = r#"
int main() {
    int a = 0;
    int b = 1;
    int result = a || b;
    return result;
}
"#;

        let (direct_asm, ir_asm, ir_output, _) = compile_both_ways(source);

        validate_ir_structure(&ir_output, &[
            "or_true_",
            "or_end_",
            "or_eval_right_",
            "br %t"
        ]);

        validate_asm_structure(&ir_asm, &["je", "jmp"]);
    }

    #[test]
    fn test_complex_logical_expression() {
        let source = r#"
int main() {
    int a = 1;
    int b = 0;
    int c = 1;
    int result = (a && b) || c;
    return result;
}
"#;

        let (direct_asm, ir_asm, ir_output, _) = compile_both_ways(source);

        validate_ir_structure(&ir_output, &[
            "and_false_",
            "and_end_",
            "or_true_",
            "or_end_"
        ]);

        validate_asm_structure(&ir_asm, &["je", "jmp"]);
    }

    #[test]
    fn test_while_loop() {
        let source = r#"
int main() {
    int i = 0;
    while (i < 5) {
        i = i + 1;
    }
    return i;
}
"#;

        let (direct_asm, ir_asm, ir_output, _) = compile_both_ways(source);

        validate_ir_structure(&ir_output, &[
            "loop_start_",
            "loop_end_",
            "loop_body_",
            "br %t",
            "jmp label %loop_start_"
        ]);

        validate_asm_structure(&ir_asm, &["loop_start_", "loop_end_", "je", "jmp"]);
    }

    #[test]
    fn test_for_loop() {
        let source = r#"
int main() {
    int sum = 0;
    for (int i = 0; i < 3; i = i + 1) {
        sum = sum + i;
    }
    return sum;
}
"#;

        let (direct_asm, ir_asm, ir_output, _) = compile_both_ways(source);

        validate_ir_structure(&ir_output, &[
            "for_start_",
            "for_end_",
            "for_continue_",
            "for_body_",
            "br %t",
            "jmp label %for_start_"
        ]);

        validate_asm_structure(&ir_asm, &["for_start_", "for_end_", "for_continue_", "je", "jmp"]);
    }

    #[test]
    fn test_nested_loops() {
        let source = r#"
int main() {
    int sum = 0;
    for (int i = 0; i < 2; i = i + 1) {
        int j = 0;
        while (j < 2) {
            sum = sum + 1;
            j = j + 1;
        }
    }
    return sum;
}
"#;

        let (direct_asm, ir_asm, ir_output, _) = compile_both_ways(source);

        validate_ir_structure(&ir_output, &[
            "for_start_",
            "for_end_",
            "loop_start_",
            "loop_end_",
            "br %t",
            "jmp label"
        ]);

        validate_asm_structure(&ir_asm, &["for_start_", "loop_start_", "je", "jmp"]);
    }

    #[test]
    fn test_break_and_continue() {
        let source = r#"
int main() {
    int i = 0;
    while (i < 10) {
        i = i + 1;
        if (i == 3) {
            continue;
        }
        if (i == 7) {
            break;
        }
    }
    return i;
}
"#;

        let (direct_asm, ir_asm, ir_output, _) = compile_both_ways(source);

        validate_ir_structure(&ir_output, &[
            "loop_start_",
            "loop_end_",
            "jmp label %loop_start_",
            "jmp label %loop_end_"
        ]);

        validate_asm_structure(&ir_asm, &["loop_start_", "loop_end_", "jmp"]);
    }

    #[test]
    fn test_function_parameters_with_logical_operators() {
        let source = r#"
int test(int x, int y) {
    return x && y;
}
"#;

        let (direct_asm, ir_asm, ir_output, _) = compile_both_ways(source);

        validate_ir_structure(&ir_output, &[
            "define i32 @test(i32 %x, i32 %y)",
            "load i32, %x",
            "load i32, %y",
            "and_false_",
            "and_end_"
        ]);

        validate_asm_structure(&ir_asm, &["test:", "je", "jmp"]);
    }
}
