//! Compile command implementation

use crate::compiler::{compile_file, CompileOptions};
use crate::error::CliError;
use anyhow::Result;
use std::path::PathBuf;
use tracing::info;

/// Run the compile command
pub fn run(input: PathBuf, output: PathBuf, emit_ir: bool, opt_level: u8) -> Result<()> {
    // Validate optimization level
    if opt_level > 3 {
        return Err(CliError::InvalidOptLevel(opt_level).into());
    }

    // Check input file exists
    if !input.exists() {
        return Err(CliError::ReadSource {
            path: input.clone(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"),
        }
        .into());
    }

    info!(
        "Compiling {} -> {}",
        input.display(),
        output.display()
    );

    // Create options
    let options = CompileOptions { emit_ir, opt_level };

    // Compile
    compile_file(&input, &output, options)?;

    println!("✓ Compiled {} to {}", input.display(), output.display());

    if emit_ir {
        let ir_path = output.with_extension("ir.txt");
        println!("✓ Emitted IR to {}", ir_path.display());
    }

    Ok(())
}
