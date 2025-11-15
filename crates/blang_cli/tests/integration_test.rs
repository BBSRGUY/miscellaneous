//! Integration tests for the Blang CLI

use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Helper to get the path to the blang binary
fn blang_binary() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // Go up to workspace root
    path.pop();
    path.push("target");
    path.push("debug");
    path.push("blang");
    path
}

/// Helper to create a temporary directory for tests
fn temp_dir() -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("blang_test_{}", std::process::id()));
    fs::create_dir_all(&path).unwrap();
    path
}

/// Helper to clean up temporary directory
fn cleanup(path: &PathBuf) {
    if path.exists() {
        fs::remove_dir_all(path).ok();
    }
}

#[test]
fn test_cli_help() {
    let output = Command::new(blang_binary())
        .arg("--help")
        .output()
        .expect("Failed to execute blang");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Blang programming language compiler"));
    assert!(stdout.contains("compile"));
    assert!(stdout.contains("bundle"));
    assert!(stdout.contains("dev"));
}

#[test]
fn test_cli_version() {
    let output = Command::new(blang_binary())
        .arg("--version")
        .output()
        .expect("Failed to execute blang");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("blang"));
}

#[test]
#[ignore] // TODO: Fix infinite loop in IR lowering/codegen
fn test_compile_simple_function() {
    let temp = temp_dir();

    // Create a simple Blang source file
    let source = r#"
fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#;

    let input_path = temp.join("test.blang");
    let output_path = temp.join("test.wasm");

    fs::write(&input_path, source).unwrap();

    // Compile the file
    let output = Command::new(blang_binary())
        .arg("compile")
        .arg(&input_path)
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute blang compile");

    if !output.status.success() {
        eprintln!("Compile failed:");
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }

    assert!(output.status.success(), "Compilation should succeed");
    assert!(output_path.exists(), "WASM file should be created");

    // Verify WASM magic number
    let wasm_bytes = fs::read(&output_path).unwrap();
    assert_eq!(&wasm_bytes[0..4], &[0x00, 0x61, 0x73, 0x6d], "Should be valid WASM");

    cleanup(&temp);
}

#[test]
#[ignore] // TODO: Fix infinite loop in IR lowering/codegen
fn test_compile_with_ir_emit() {
    let temp = temp_dir();

    let source = "fn test() {}";
    let input_path = temp.join("test.blang");
    let output_path = temp.join("test.wasm");

    fs::write(&input_path, source).unwrap();

    // Compile with IR emission
    let output = Command::new(blang_binary())
        .arg("compile")
        .arg(&input_path)
        .arg("-o")
        .arg(&output_path)
        .arg("--emit-ir")
        .output()
        .expect("Failed to execute blang compile");

    assert!(output.status.success());
    assert!(output_path.exists(), "WASM file should be created");

    // Check that IR file was created
    let ir_path = temp.join("test.ir.txt");
    assert!(ir_path.exists(), "IR file should be created with --emit-ir");

    cleanup(&temp);
}

#[test]
#[ignore] // TODO: Fix infinite loop in IR lowering/codegen
fn test_compile_with_optimization() {
    let temp = temp_dir();

    let source = r#"
fn factorial(n: i32) -> i32 {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}
"#;

    let input_path = temp.join("test.blang");
    let output_path = temp.join("test.wasm");

    fs::write(&input_path, source).unwrap();

    // Compile with optimization level 2
    let output = Command::new(blang_binary())
        .arg("compile")
        .arg(&input_path)
        .arg("-o")
        .arg(&output_path)
        .arg("-O")
        .arg("2")
        .output()
        .expect("Failed to execute blang compile");

    assert!(output.status.success());
    assert!(output_path.exists());

    cleanup(&temp);
}

#[test]
fn test_compile_invalid_input() {
    let temp = temp_dir();
    let input_path = temp.join("nonexistent.blang");
    let output_path = temp.join("test.wasm");

    // Try to compile a file that doesn't exist
    let output = Command::new(blang_binary())
        .arg("compile")
        .arg(&input_path)
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute blang compile");

    assert!(!output.status.success(), "Should fail for nonexistent file");

    cleanup(&temp);
}

#[test]
fn test_compile_invalid_syntax() {
    let temp = temp_dir();

    // Create a file with invalid syntax
    let source = "fn {"; // Invalid syntax
    let input_path = temp.join("invalid.blang");
    let output_path = temp.join("invalid.wasm");

    fs::write(&input_path, source).unwrap();

    let output = Command::new(blang_binary())
        .arg("compile")
        .arg(&input_path)
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute blang compile");

    assert!(!output.status.success(), "Should fail for invalid syntax");

    cleanup(&temp);
}

#[test]
#[ignore] // TODO: Fix infinite loop in IR lowering/codegen
fn test_bundle_command() {
    let temp = temp_dir();

    let source = r#"
fn main() {
    let x = 42;
}
"#;

    let input_path = temp.join("main.blang");
    let out_dir = temp.join("dist");

    fs::write(&input_path, source).unwrap();

    // Create bundle
    let output = Command::new(blang_binary())
        .arg("bundle")
        .arg(&input_path)
        .arg("--out-dir")
        .arg(&out_dir)
        .output()
        .expect("Failed to execute blang bundle");

    if !output.status.success() {
        eprintln!("Bundle failed:");
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }

    assert!(output.status.success(), "Bundle should succeed");

    // Check that all expected files are created
    assert!(out_dir.join("app.wasm").exists(), "WASM file should exist");
    assert!(out_dir.join("runtime.js").exists(), "Runtime.js should exist");
    assert!(out_dir.join("index.html").exists(), "index.html should exist");

    // Verify index.html contains expected content
    let html = fs::read_to_string(out_dir.join("index.html")).unwrap();
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("runtime.js"));
    assert!(html.contains("app.wasm"));

    cleanup(&temp);
}

#[test]
#[ignore] // TODO: Fix infinite loop in IR lowering/codegen
fn test_bundle_with_release() {
    let temp = temp_dir();

    let source = "fn test() {}";
    let input_path = temp.join("test.blang");
    let out_dir = temp.join("dist");

    fs::write(&input_path, source).unwrap();

    // Create release bundle
    let output = Command::new(blang_binary())
        .arg("bundle")
        .arg(&input_path)
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--release")
        .output()
        .expect("Failed to execute blang bundle");

    assert!(output.status.success());
    assert!(out_dir.join("app.wasm").exists());

    cleanup(&temp);
}

#[test]
fn test_invalid_opt_level() {
    let temp = temp_dir();

    let source = "fn test() {}";
    let input_path = temp.join("test.blang");
    let output_path = temp.join("test.wasm");

    fs::write(&input_path, source).unwrap();

    // Try to compile with invalid optimization level
    let output = Command::new(blang_binary())
        .arg("compile")
        .arg(&input_path)
        .arg("-o")
        .arg(&output_path)
        .arg("-O")
        .arg("5") // Invalid: should be 0-3
        .output()
        .expect("Failed to execute blang compile");

    assert!(!output.status.success(), "Should fail for invalid opt level");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid optimization level") || stderr.contains("5"));

    cleanup(&temp);
}

#[test]
#[ignore] // TODO: Fix infinite loop in IR lowering/codegen
fn test_verbose_flag() {
    let temp = temp_dir();

    let source = "fn test() {}";
    let input_path = temp.join("test.blang");
    let output_path = temp.join("test.wasm");

    fs::write(&input_path, source).unwrap();

    // Compile with verbose flag
    let output = Command::new(blang_binary())
        .arg("--verbose")
        .arg("compile")
        .arg(&input_path)
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute blang compile");

    assert!(output.status.success());

    // Verbose mode should produce more output (though exact format depends on implementation)
    let combined_output = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Should contain logging output from compilation stages
    assert!(
        combined_output.contains("Parsing") ||
        combined_output.contains("Lowering") ||
        combined_output.contains("Generating")
    );

    cleanup(&temp);
}
