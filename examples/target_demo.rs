use compiler_minic::codegen::{Codegen};
use compiler_minic::codegen::targets::TargetPlatform;
use compiler_minic::ir::{IrProgram, IrFunction, IrInstruction, IrValue, IrType};

fn main() {
    // Create a simple IR program
    let program = IrProgram {
        functions: vec![
            IrFunction {
                name: "main".to_string(),
                return_type: IrType::Int,
                parameters: vec![],
                local_vars: vec![],
                instructions: vec![
                    IrInstruction::Print {
                        format_string: IrValue::StringConstant("hello_msg".to_string()),
                        args: vec![],
                    },
                    IrInstruction::Return {
                        value: Some(IrValue::IntConstant(0)),
                        var_type: IrType::Int,
                    },
                ],
            }
        ],
        global_strings: vec![
            ("hello_msg".to_string(), "Hello, World!".to_string()),
        ],
    };

    println!("=== WINDOWS X64 TARGET ===");
    let windows_codegen = Codegen::new_with_target(TargetPlatform::WindowsX64);
    let windows_asm = windows_codegen.generate(&program);
    println!("{}", windows_asm);

    println!("\n=== LINUX X64 TARGET ===");
    let linux_codegen = Codegen::new_with_target(TargetPlatform::LinuxX64);
    let linux_asm = linux_codegen.generate(&program);
    println!("{}", linux_asm);

    println!("\n=== MACOS X64 TARGET ===");
    let macos_codegen = Codegen::new_with_target(TargetPlatform::MacOSX64);
    let macos_asm = macos_codegen.generate(&program);
    println!("{}", macos_asm);
}