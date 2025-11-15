//! Tests for unsafe block parsing

use blang_parser::parse;

#[test]
fn test_parse_simple_unsafe_block() {
    let source = r#"unsafe block add(a: i32, b: i32) -> i32 {
    a + b
}"#;

    let result = parse(source);

    if let Err(ref e) = result {
        eprintln!("Parse error: {:?}", e);
        eprintln!("Source: {}", source);
    }

    assert!(result.is_ok(), "Parser failed: {:?}", result.err());
    let items = result.unwrap();
    assert_eq!(items.len(), 1);

    match &items[0].kind {
        blang_ast::ItemKind::UnsafeBlock(unsafe_block) => {
            assert_eq!(unsafe_block.name.name, "add");
            assert_eq!(unsafe_block.params.len(), 2);
            assert!(unsafe_block.return_ty.is_some());
        }
        _ => panic!("Expected UnsafeBlock, got {:?}", items[0].kind),
    }
}

#[test]
fn test_parse_unsafe_block_no_params() {
    let source = r#"
        unsafe block get_constant() -> i32 {
            42
        }
    "#;

    let result = parse(source);

    assert!(result.is_ok());
    let items = result.unwrap();
    assert_eq!(items.len(), 1);

    match &items[0].kind {
        blang_ast::ItemKind::UnsafeBlock(unsafe_block) => {
            assert_eq!(unsafe_block.name.name, "get_constant");
            assert_eq!(unsafe_block.params.len(), 0);
        }
        _ => panic!("Expected UnsafeBlock"),
    }
}

#[test]
fn test_parse_unsafe_block_no_return() {
    let source = r#"
        unsafe block do_work(x: i32) {
            let y = x + 1;
        }
    "#;

    let result = parse(source);

    assert!(result.is_ok());
    let items = result.unwrap();
    assert_eq!(items.len(), 1);

    match &items[0].kind {
        blang_ast::ItemKind::UnsafeBlock(unsafe_block) => {
            assert_eq!(unsafe_block.name.name, "do_work");
            assert_eq!(unsafe_block.params.len(), 1);
            assert!(unsafe_block.return_ty.is_none());
        }
        _ => panic!("Expected UnsafeBlock"),
    }
}

#[test]
fn test_parse_unsafe_block_with_pointer() {
    let source = r#"
        unsafe block ptr_offset(ptr: *i32, offset: i32) -> *i32 {
            ptr + offset
        }
    "#;

    let result = parse(source);

    assert!(result.is_ok());
    let items = result.unwrap();
    assert_eq!(items.len(), 1);

    match &items[0].kind {
        blang_ast::ItemKind::UnsafeBlock(unsafe_block) => {
            assert_eq!(unsafe_block.name.name, "ptr_offset");
            assert_eq!(unsafe_block.params.len(), 2);
        }
        _ => panic!("Expected UnsafeBlock"),
    }
}

#[test]
fn test_parse_multiple_unsafe_blocks() {
    let source = r#"
        unsafe block add(a: i32, b: i32) -> i32 {
            a + b
        }

        unsafe block mul(a: i32, b: i32) -> i32 {
            a * b
        }
    "#;

    let result = parse(source);

    assert!(result.is_ok());
    let items = result.unwrap();
    assert_eq!(items.len(), 2);

    match (&items[0].kind, &items[1].kind) {
        (blang_ast::ItemKind::UnsafeBlock(b1), blang_ast::ItemKind::UnsafeBlock(b2)) => {
            assert_eq!(b1.name.name, "add");
            assert_eq!(b2.name.name, "mul");
        }
        _ => panic!("Expected two UnsafeBlocks"),
    }
}

#[test]
fn test_parse_unsafe_fn_still_works() {
    // Ensure that "unsafe fn" syntax still works
    let source = r#"
        unsafe fn dangerous() -> i32 {
            42
        }
    "#;

    let result = parse(source);

    assert!(result.is_ok());
    let items = result.unwrap();
    assert_eq!(items.len(), 1);

    match &items[0].kind {
        blang_ast::ItemKind::Function(func) => {
            assert_eq!(func.sig.name.name, "dangerous");
            assert!(func.sig.unsafe_);
        }
        _ => panic!("Expected Function, got {:?}", items[0].kind),
    }
}
