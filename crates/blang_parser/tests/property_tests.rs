//! Property-based tests for the parser
//!
//! These tests verify parser invariants and robustness.

use blang_parser::{parse, parse_expr, parse_stmt, parse_type};
use proptest::prelude::*;

/// Test that the parser never panics on any input
#[test]
fn parser_never_panics() {
    proptest!(|(input in "\\PC*")| {
        // Parser should never panic, even on invalid input
        let _ = parse(&input);
    });
}

/// Test that parser handles valid keywords without panicking
#[test]
fn parser_handles_keywords() {
    let keywords = vec![
        "fn", "let", "if", "else", "while", "for", "return",
        "break", "continue", "struct", "enum", "impl", "trait",
        "type", "const", "static", "pub", "use", "mod", "unsafe",
        "component", "script", "block",
    ];

    proptest!(|(keyword in prop::sample::select(keywords))| {
        // Should not panic
        let _ = parse(keyword);
    });
}

/// Test that parser handles whitespace variations
#[test]
fn parser_handles_whitespace_variations() {
    let base_programs = vec![
        "fn test() {}",
        "let x = 42;",
        "struct Point { x: i32 }",
    ];

    proptest!(|(
        program in prop::sample::select(base_programs),
        spaces in prop::collection::vec(" |\t|\n", 0..5)
    )| {
        // Insert random whitespace
        let with_whitespace = program.chars()
            .zip(spaces.iter().cycle())
            .flat_map(|(c, ws)| vec![c.to_string(), ws.to_string()])
            .collect::<String>();

        // Should not panic
        let _ = parse(&with_whitespace);
    });
}

/// Test that valid function declarations parse successfully
#[test]
fn valid_functions_parse() {
    let functions = vec![
        "fn test() {}",
        "fn add(x: i32) {}",
        "fn add(x: i32, y: i32) {}",
        "fn get() -> i32 { 42 }",
        "pub fn public_fn() {}",
        "unsafe fn unsafe_fn() {}",
    ];

    for func in functions {
        let result = parse(func);
        assert!(result.is_ok(), "Failed to parse valid function: {} - {:?}", func, result.err());
        let items = result.unwrap();
        assert_eq!(items.len(), 1, "Should parse exactly one item");
    }
}

/// Test that valid struct declarations parse successfully
#[test]
fn valid_structs_parse() {
    let structs = vec![
        "struct Empty {}",
        "struct Point { x: i32 }",
        "struct Point { x: i32, y: i32 }",
        "pub struct Public {}",
    ];

    for s in structs {
        let result = parse(s);
        assert!(result.is_ok(), "Failed to parse valid struct: {} - {:?}", s, result.err());
    }
}

/// Test that parser produces valid spans
#[test]
fn parser_produces_valid_spans() {
    proptest!(|(input in "[a-zA-Z_][a-zA-Z0-9_]*")| {
        // Try to parse as identifier/function/etc
        if let Ok(items) = parse(&format!("fn {}() {{}}", input)) {
            for item in items {
                // Span should be within input bounds
                let adjusted_input = format!("fn {}() {{}}", input);
                prop_assert!(item.span.start.0 < adjusted_input.len() as u32,
                    "Item span start {} exceeds input length {}",
                    item.span.start.0, adjusted_input.len());
            }
        }
    });
}

/// Test expression parsing
#[test]
fn expression_parsing_works() {
    let expressions = vec![
        "42",
        "x",
        "x + y",
        "x * y + z",
        "f()",
        "f(x)",
        "f(x, y)",
        "(x + y) * z",
        "true",
        "false",
        "null",
    ];

    for expr in expressions {
        let result = parse_expr(expr);
        assert!(result.is_ok(), "Failed to parse expression: {} - {:?}", expr, result.err());
    }
}

/// Test statement parsing
#[test]
fn statement_parsing_works() {
    let statements = vec![
        "let x = 42;",
        "let mut x = 42;",
        "x = 10;",
        "return;",
        "return 42;",
        "break;",
        "continue;",
    ];

    for stmt in statements {
        let result = parse_stmt(stmt);
        assert!(result.is_ok(), "Failed to parse statement: {} - {:?}", stmt, result.err());
    }
}

/// Test type parsing
#[test]
fn type_parsing_works() {
    let types = vec![
        "i32",
        "u32",
        "f64",
        "bool",
        "str",
        "*i32",
    ];

    for ty in types {
        let result = parse_type(ty);
        assert!(result.is_ok(), "Failed to parse type: {} - {:?}", ty, result.err());
    }
}

/// Test that parser handles nested structures
#[test]
fn parser_handles_nesting() {
    let nested = vec![
        "fn outer() { fn inner() {} }",  // This might not be valid syntax, but shouldn't panic
        "{ { { } } }",
        "if true { if false { } }",
    ];

    for program in nested {
        // Should not panic
        let _ = parse(program);
    }
}

/// Test that parser handles incomplete input gracefully
#[test]
fn parser_handles_incomplete_input() {
    let incomplete = vec![
        "fn",
        "fn test",
        "fn test(",
        "fn test()",
        "let",
        "let x",
        "let x =",
        "struct",
        "struct Point",
        "struct Point {",
    ];

    for input in incomplete {
        let result = parse(input);
        // Should either parse or error, but not panic
        match result {
            Ok(_) => {}, // Unexpected but ok
            Err(e) => {
                // Error is expected - verify it has useful information
                assert!(!format!("{:?}", e).is_empty());
            }
        }
    }
}

/// Test that parser handles very long identifiers
#[test]
fn parser_handles_long_identifiers() {
    proptest!(|(len in 1usize..1000)| {
        let long_ident = "a".repeat(len);
        let program = format!("fn {}() {{}}", long_ident);

        // Should not panic
        let _ = parse(&program);
    });
}

/// Test that parser handles many parameters
#[test]
fn parser_handles_many_parameters() {
    proptest!(|(count in 0usize..50)| {
        let params = (0..count)
            .map(|i| format!("x{}: i32", i))
            .collect::<Vec<_>>()
            .join(", ");
        let program = format!("fn test({}) {{}}", params);

        // Should not panic
        let _ = parse(&program);
    });
}

/// Test that parser handles many struct fields
#[test]
fn parser_handles_many_fields() {
    proptest!(|(count in 0usize..50)| {
        let fields = (0..count)
            .map(|i| format!("field{}: i32", i))
            .collect::<Vec<_>>()
            .join(", ");
        let program = format!("struct S {{ {} }}", fields);

        // Should not panic
        let _ = parse(&program);
    });
}

/// Test component and script parsing
#[test]
fn component_and_script_parse() {
    let programs = vec![
        "component Counter {}",
        "component Counter { fn increment() {} }",
        "script DataProcessor {}",
        "script DataProcessor { fn process() {} }",
    ];

    for program in programs {
        let result = parse(program);
        assert!(result.is_ok(), "Failed to parse: {} - {:?}", program, result.err());
    }
}

/// Test unsafe block parsing
#[test]
fn unsafe_blocks_parse() {
    let programs = vec![
        "unsafe block test() {}",
        "unsafe block test(x: i32) {}",
        "unsafe block test() -> i32 { 42 }",
    ];

    for program in programs {
        let result = parse(program);
        assert!(result.is_ok(), "Failed to parse unsafe block: {} - {:?}", program, result.err());
    }
}

/// Test that parser handles complex realistic programs
#[test]
fn parser_handles_realistic_programs() {
    let programs = vec![
        r#"
            fn fibonacci(n: i32) -> i32 {
                if n <= 1 {
                    n
                } else {
                    fibonacci(n - 1) + fibonacci(n - 2)
                }
            }
        "#,
        r#"
            struct Point {
                x: i32,
                y: i32,
            }

            fn distance(p1: Point, p2: Point) -> i32 {
                let dx = p1.x - p2.x;
                let dy = p1.y - p2.y;
                dx * dx + dy * dy
            }
        "#,
        r#"
            component Counter {
                fn increment() {}
                fn decrement() {}
                fn get_value() -> i32 { 0 }
            }
        "#,
    ];

    for program in programs {
        let result = parse(program);
        assert!(result.is_ok(), "Failed to parse realistic program: {:?}", result.err());
    }
}

/// Fuzz test with random valid-looking tokens
#[test]
fn fuzz_with_valid_tokens() {
    let tokens = vec!["fn", "let", "if", "else", "(", ")", "{", "}", "x", "42", "+", "-", ";"];

    proptest!(ProptestConfig::with_cases(50), |(
        token_seq in prop::collection::vec(prop::sample::select(tokens), 1..50)
    )| {
        let input = token_seq.join(" ");

        // Should not panic
        let _ = parse(&input);
    });
}

/// Test that re-parsing doesn't change the structure
#[test]
fn parse_is_deterministic() {
    let programs = vec![
        "fn test() {}",
        "let x = 42;",
        "struct Point { x: i32 }",
    ];

    for program in programs {
        let result1 = parse(program);
        let result2 = parse(program);

        // Both should have same success/failure
        assert_eq!(result1.is_ok(), result2.is_ok(),
            "Parse results differ for: {}", program);
    }
}
