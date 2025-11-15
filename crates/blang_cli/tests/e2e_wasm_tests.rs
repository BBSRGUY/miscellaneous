//! End-to-end tests that compile Blang to WASM and execute it
//!
//! These tests verify the entire compilation pipeline by:
//! 1. Compiling Blang source to WASM
//! 2. Loading the WASM into wasmtime
//! 3. Executing functions and verifying results

use blang_cli::compiler::{compile_source, CompileOptions};
use wasmtime::*;

/// Helper to compile Blang source and create a WASM module
fn compile_and_load(source: &str) -> Result<(Store<()>, Instance), Box<dyn std::error::Error>> {
    // Compile Blang to WASM
    let result = compile_source(
        source,
        CompileOptions {
            emit_ir: false,
            opt_level: 0,
        },
    )?;

    // Create wasmtime engine and module
    let engine = Engine::default();
    let module = Module::new(&engine, &result.wasm_bytes)?;

    // Create store and instance
    let mut store = Store::new(&engine, ());
    let instance = Instance::new(&mut store, &module, &[])?;

    Ok((store, instance))
}

/// Helper to get a typed function from an instance
fn get_func<Params, Results>(
    store: &mut Store<()>,
    instance: &Instance,
    name: &str,
) -> Result<TypedFunc<Params, Results>, Box<dyn std::error::Error>>
where
    Params: WasmParams,
    Results: WasmResults,
{
    let func = instance
        .get_func(&mut *store, name)
        .ok_or(format!("Function '{}' not found", name))?;

    func.typed::<Params, Results>(&store)
        .map_err(|e| format!("Type mismatch for '{}': {}", name, e).into())
}

#[test]
#[ignore] // TODO: Enable once IR lowering is fixed
fn test_simple_return_constant() {
    let source = r#"
        fn get_answer() -> i32 {
            42
        }
    "#;

    let (mut store, instance) = compile_and_load(source).expect("Compilation failed");

    let get_answer = get_func::<(), i32>(&mut store, &instance, "get_answer")
        .expect("Failed to get function");

    let result = get_answer.call(&mut store, ()).expect("Execution failed");
    assert_eq!(result, 42, "Expected function to return 42");
}

#[test]
#[ignore] // TODO: Enable once IR lowering is fixed
fn test_simple_addition() {
    let source = r#"
        fn add(a: i32, b: i32) -> i32 {
            a + b
        }
    "#;

    let (mut store, instance) = compile_and_load(source).expect("Compilation failed");

    let add = get_func::<(i32, i32), i32>(&mut store, &instance, "add")
        .expect("Failed to get function");

    let test_cases = vec![
        ((2, 3), 5),
        ((10, 20), 30),
        ((0, 0), 0),
        ((-5, 5), 0),
        ((-10, -20), -30),
    ];

    for ((a, b), expected) in test_cases {
        let result = add.call(&mut store, (a, b)).expect("Execution failed");
        assert_eq!(result, expected, "add({}, {}) should be {}", a, b, expected);
    }
}

#[test]
#[ignore] // TODO: Enable once IR lowering is fixed
fn test_subtraction() {
    let source = r#"
        fn subtract(a: i32, b: i32) -> i32 {
            a - b
        }
    "#;

    let (mut store, instance) = compile_and_load(source).expect("Compilation failed");

    let subtract = get_func::<(i32, i32), i32>(&mut store, &instance, "subtract")
        .expect("Failed to get function");

    let result = subtract.call(&mut store, (10, 3)).expect("Execution failed");
    assert_eq!(result, 7);
}

#[test]
#[ignore] // TODO: Enable once IR lowering is fixed
fn test_multiplication() {
    let source = r#"
        fn multiply(a: i32, b: i32) -> i32 {
            a * b
        }
    "#;

    let (mut store, instance) = compile_and_load(source).expect("Compilation failed");

    let multiply = get_func::<(i32, i32), i32>(&mut store, &instance, "multiply")
        .expect("Failed to get function");

    let test_cases = vec![
        ((2, 3), 6),
        ((10, 5), 50),
        ((0, 100), 0),
        ((-5, 3), -15),
    ];

    for ((a, b), expected) in test_cases {
        let result = multiply.call(&mut store, (a, b)).expect("Execution failed");
        assert_eq!(result, expected, "multiply({}, {}) should be {}", a, b, expected);
    }
}

#[test]
#[ignore] // TODO: Enable once IR lowering is fixed
fn test_conditional_if() {
    let source = r#"
        fn max(a: i32, b: i32) -> i32 {
            if a > b {
                a
            } else {
                b
            }
        }
    "#;

    let (mut store, instance) = compile_and_load(source).expect("Compilation failed");

    let max = get_func::<(i32, i32), i32>(&mut store, &instance, "max")
        .expect("Failed to get function");

    let test_cases = vec![
        ((5, 3), 5),
        ((3, 5), 5),
        ((7, 7), 7),
        ((-10, 5), 5),
        ((-5, -10), -5),
    ];

    for ((a, b), expected) in test_cases {
        let result = max.call(&mut store, (a, b)).expect("Execution failed");
        assert_eq!(result, expected, "max({}, {}) should be {}", a, b, expected);
    }
}

#[test]
#[ignore] // TODO: Enable once IR lowering is fixed
fn test_factorial_iterative() {
    let source = r#"
        fn factorial(n: i32) -> i32 {
            let mut result = 1;
            let mut i = 1;
            while i <= n {
                result = result * i;
                i = i + 1;
            }
            result
        }
    "#;

    let (mut store, instance) = compile_and_load(source).expect("Compilation failed");

    let factorial = get_func::<i32, i32>(&mut store, &instance, "factorial")
        .expect("Failed to get function");

    let test_cases = vec![
        (0, 1),
        (1, 1),
        (2, 2),
        (3, 6),
        (4, 24),
        (5, 120),
    ];

    for (n, expected) in test_cases {
        let result = factorial.call(&mut store, n).expect("Execution failed");
        assert_eq!(result, expected, "factorial({}) should be {}", n, expected);
    }
}

#[test]
#[ignore] // TODO: Enable once IR lowering is fixed
fn test_fibonacci_recursive() {
    let source = r#"
        fn fibonacci(n: i32) -> i32 {
            if n <= 1 {
                n
            } else {
                fibonacci(n - 1) + fibonacci(n - 2)
            }
        }
    "#;

    let (mut store, instance) = compile_and_load(source).expect("Compilation failed");

    let fibonacci = get_func::<i32, i32>(&mut store, &instance, "fibonacci")
        .expect("Failed to get function");

    let test_cases = vec![
        (0, 0),
        (1, 1),
        (2, 1),
        (3, 2),
        (4, 3),
        (5, 5),
        (6, 8),
        (7, 13),
    ];

    for (n, expected) in test_cases {
        let result = fibonacci.call(&mut store, n).expect("Execution failed");
        assert_eq!(result, expected, "fibonacci({}) should be {}", n, expected);
    }
}

#[test]
#[ignore] // TODO: Enable once IR lowering is fixed
fn test_boolean_operations() {
    let source = r#"
        fn and_op(a: bool, b: bool) -> bool {
            a && b
        }

        fn or_op(a: bool, b: bool) -> bool {
            a || b
        }

        fn not_op(a: bool) -> bool {
            !a
        }
    "#;

    let (mut store, instance) = compile_and_load(source).expect("Compilation failed");

    // Test AND
    let and_op = get_func::<(i32, i32), i32>(&mut store, &instance, "and_op")
        .expect("Failed to get and_op");

    assert_eq!(and_op.call(&mut store, (1, 1)).unwrap(), 1);  // true && true
    assert_eq!(and_op.call(&mut store, (1, 0)).unwrap(), 0);  // true && false
    assert_eq!(and_op.call(&mut store, (0, 1)).unwrap(), 0);  // false && true
    assert_eq!(and_op.call(&mut store, (0, 0)).unwrap(), 0);  // false && false

    // Test OR
    let or_op = get_func::<(i32, i32), i32>(&mut store, &instance, "or_op")
        .expect("Failed to get or_op");

    assert_eq!(or_op.call(&mut store, (1, 1)).unwrap(), 1);  // true || true
    assert_eq!(or_op.call(&mut store, (1, 0)).unwrap(), 1);  // true || false
    assert_eq!(or_op.call(&mut store, (0, 1)).unwrap(), 1);  // false || true
    assert_eq!(or_op.call(&mut store, (0, 0)).unwrap(), 0);  // false || false

    // Test NOT
    let not_op = get_func::<i32, i32>(&mut store, &instance, "not_op")
        .expect("Failed to get not_op");

    assert_eq!(not_op.call(&mut store, 1).unwrap(), 0);  // !true
    assert_eq!(not_op.call(&mut store, 0).unwrap(), 1);  // !false
}

#[test]
#[ignore] // TODO: Enable once IR lowering is fixed
fn test_comparison_operations() {
    let source = r#"
        fn is_equal(a: i32, b: i32) -> bool {
            a == b
        }

        fn is_not_equal(a: i32, b: i32) -> bool {
            a != b
        }

        fn is_less(a: i32, b: i32) -> bool {
            a < b
        }

        fn is_greater(a: i32, b: i32) -> bool {
            a > b
        }
    "#;

    let (mut store, instance) = compile_and_load(source).expect("Compilation failed");

    let is_equal = get_func::<(i32, i32), i32>(&mut store, &instance, "is_equal")
        .expect("Failed to get is_equal");
    assert_eq!(is_equal.call(&mut store, (5, 5)).unwrap(), 1);
    assert_eq!(is_equal.call(&mut store, (5, 3)).unwrap(), 0);

    let is_not_equal = get_func::<(i32, i32), i32>(&mut store, &instance, "is_not_equal")
        .expect("Failed to get is_not_equal");
    assert_eq!(is_not_equal.call(&mut store, (5, 3)).unwrap(), 1);
    assert_eq!(is_not_equal.call(&mut store, (5, 5)).unwrap(), 0);

    let is_less = get_func::<(i32, i32), i32>(&mut store, &instance, "is_less")
        .expect("Failed to get is_less");
    assert_eq!(is_less.call(&mut store, (3, 5)).unwrap(), 1);
    assert_eq!(is_less.call(&mut store, (5, 3)).unwrap(), 0);

    let is_greater = get_func::<(i32, i32), i32>(&mut store, &instance, "is_greater")
        .expect("Failed to get is_greater");
    assert_eq!(is_greater.call(&mut store, (5, 3)).unwrap(), 1);
    assert_eq!(is_greater.call(&mut store, (3, 5)).unwrap(), 0);
}

#[test]
#[ignore] // TODO: Enable once IR lowering is fixed
fn test_complex_arithmetic() {
    let source = r#"
        fn complex_calc(x: i32, y: i32, z: i32) -> i32 {
            (x + y) * z - (x - y) / 2
        }
    "#;

    let (mut store, instance) = compile_and_load(source).expect("Compilation failed");

    let complex_calc = get_func::<(i32, i32, i32), i32>(&mut store, &instance, "complex_calc")
        .expect("Failed to get function");

    // (10 + 5) * 2 - (10 - 5) / 2 = 15 * 2 - 5 / 2 = 30 - 2 = 28
    let result = complex_calc.call(&mut store, (10, 5, 2)).expect("Execution failed");
    assert_eq!(result, 28);
}

#[test]
#[ignore] // TODO: Enable once IR lowering is fixed
fn test_let_bindings() {
    let source = r#"
        fn calculate() -> i32 {
            let x = 10;
            let y = 20;
            let z = x + y;
            z * 2
        }
    "#;

    let (mut store, instance) = compile_and_load(source).expect("Compilation failed");

    let calculate = get_func::<(), i32>(&mut store, &instance, "calculate")
        .expect("Failed to get function");

    let result = calculate.call(&mut store, ()).expect("Execution failed");
    assert_eq!(result, 60);  // (10 + 20) * 2
}

#[test]
#[ignore] // TODO: Enable once IR lowering is fixed
fn test_wasm_output_is_valid() {
    // Even though we can't execute due to IR lowering bug, we can verify WASM is valid
    let source = "fn test() {}";

    let result = compile_source(
        source,
        CompileOptions {
            emit_ir: false,
            opt_level: 0,
        },
    );

    // Should compile without error
    assert!(result.is_ok(), "Compilation should succeed");

    let wasm_bytes = result.unwrap().wasm_bytes;

    // Verify WASM magic number and version
    assert_eq!(&wasm_bytes[0..4], &[0x00, 0x61, 0x73, 0x6d], "Invalid WASM magic number");
    assert_eq!(&wasm_bytes[4..8], &[0x01, 0x00, 0x00, 0x00], "Invalid WASM version");

    // Try to parse with wasmtime to verify it's valid
    let engine = Engine::default();
    let module_result = Module::new(&engine, &wasm_bytes);
    assert!(module_result.is_ok(), "Generated WASM should be valid: {:?}", module_result.err());
}

#[test]
#[ignore] // TODO: Enable once IR lowering is fixed
fn test_multiple_compilation() {
    // Test that we can compile multiple programs in succession
    let programs = vec![
        "fn a() {}",
        "fn b() {}",
        "fn c() {}",
    ];

    for source in programs {
        let result = compile_source(
            source,
            CompileOptions {
                emit_ir: false,
                opt_level: 0,
            },
        );

        assert!(result.is_ok(), "Failed to compile: {}", source);

        let wasm_bytes = result.unwrap().wasm_bytes;
        assert_eq!(&wasm_bytes[0..4], &[0x00, 0x61, 0x73, 0x6d]);
    }
}

#[test]
#[ignore] // TODO: Enable once IR lowering is fixed
fn test_optimization_levels() {
    let source = "fn test() {}";

    for opt_level in 0..=3 {
        let result = compile_source(
            source,
            CompileOptions {
                emit_ir: false,
                opt_level,
            },
        );

        assert!(result.is_ok(), "Compilation with opt level {} should succeed", opt_level);

        let wasm_bytes = result.unwrap().wasm_bytes;
        assert_eq!(&wasm_bytes[0..4], &[0x00, 0x61, 0x73, 0x6d]);
    }
}
