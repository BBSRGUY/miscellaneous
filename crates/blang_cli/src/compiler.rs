//! Compiler pipeline implementation

use crate::error::{CliError, CliResult};
use blang_codegen_wasm::compile_to_wasm;
use blang_ir::{IrLowerer, Module};
use blang_parser::parse;
use std::fs;
use std::path::Path;
use tracing::info;

/// Compilation options
pub struct CompileOptions {
    pub emit_ir: bool,
    pub opt_level: u8,
}

/// Complete compilation result
pub struct CompilationResult {
    pub wasm_bytes: Vec<u8>,
    pub ir_module: Option<Module>,
}

/// Run the full compilation pipeline: source → WASM
pub fn compile_source(source: &str, options: CompileOptions) -> CliResult<CompilationResult> {
    // 1. Parse
    info!("Parsing source code...");
    let items = parse(source).map_err(|e| CliError::Parse {
        location: format!("{:?}", e.span),
        message: format!("{}", e.kind),
    })?;

    info!("Parsed {} top-level items", items.len());

    // 2. Type checking (placeholder - would integrate blang_typeck when available)
    info!("Type checking...");
    // TODO: Integrate type checker when implemented

    // 3. Lower to IR
    info!("Lowering to IR...");
    let mut lowerer = IrLowerer::new("main".to_string());
    lowerer
        .lower_items(&items)
        .map_err(|e| CliError::IrLowering(e.to_string()))?;

    let mut ir_module = lowerer.into_module();

    // 4. Optimize IR (if requested)
    if options.opt_level > 0 {
        info!("Optimizing IR (level {})...", options.opt_level);
        blang_ir::optimize_module(&mut ir_module);
    }

    // 5. Validate IR
    info!("Validating IR...");
    blang_ir::validate_module(&ir_module)
        .map_err(|e| CliError::IrLowering(e.to_string()))?;

    // 6. Generate WASM
    info!("Generating WebAssembly...");
    let wasm_bytes =
        compile_to_wasm(&ir_module).map_err(|e| CliError::Codegen(e.to_string()))?;

    info!("Generated {} bytes of WASM", wasm_bytes.len());

    Ok(CompilationResult {
        wasm_bytes,
        ir_module: if options.emit_ir {
            Some(ir_module)
        } else {
            None
        },
    })
}

/// Compile a file to WASM
pub fn compile_file(
    input_path: &Path,
    output_path: &Path,
    options: CompileOptions,
) -> CliResult<()> {
    // Read source file
    info!("Reading source file: {}", input_path.display());
    let source = fs::read_to_string(input_path).map_err(|e| CliError::ReadSource {
        path: input_path.to_path_buf(),
        source: e,
    })?;

    // Compile
    let result = compile_source(&source, options)?;

    // Write WASM output
    info!("Writing WASM to: {}", output_path.display());
    fs::write(output_path, &result.wasm_bytes).map_err(|e| CliError::WriteOutput {
        path: output_path.to_path_buf(),
        source: e,
    })?;

    // Write IR if requested
    if let Some(ir_module) = result.ir_module {
        let ir_path = output_path.with_extension("ir.txt");
        info!("Writing IR to: {}", ir_path.display());
        fs::write(&ir_path, format!("{}", ir_module)).map_err(|e| CliError::WriteOutput {
            path: ir_path,
            source: e,
        })?;
    }

    info!("Compilation successful!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_simple_function() {
        let source = r#"
            fn add(a: i32, b: i32) -> i32 {
                a + b
            }
        "#;

        let options = CompileOptions {
            emit_ir: false,
            opt_level: 0,
        };

        let result = compile_source(source, options);
        assert!(result.is_ok(), "Compilation failed: {:?}", result.err());

        let result = result.unwrap();
        assert!(!result.wasm_bytes.is_empty());
        assert_eq!(&result.wasm_bytes[0..4], &[0x00, 0x61, 0x73, 0x6d]); // WASM magic
    }

    #[test]
    fn test_compile_with_ir_emit() {
        let source = "fn test() {}";

        let options = CompileOptions {
            emit_ir: true,
            opt_level: 0,
        };

        let result = compile_source(source, options).unwrap();
        assert!(result.ir_module.is_some());
    }
}
