//! Property-based tests for the lexer
//!
//! These tests use proptest to generate random inputs and verify invariants.

use blang_lexer::{Lexer, TokenKind};
use proptest::prelude::*;

/// Test that the lexer never panics on any input
#[test]
fn lexer_never_panics_on_any_input() {
    proptest!(|(input in "\\PC*")| {
        let mut lexer = Lexer::new(&input);

        // Consume all tokens - should never panic
        loop {
            let token = lexer.next_token();
            if token.kind == TokenKind::Eof {
                break;
            }
        }
    });
}

/// Test that token spans are contiguous and cover the entire input
#[test]
fn token_spans_are_contiguous() {
    proptest!(|(input in "[a-zA-Z0-9_ +\\-*/=<>!&|(){}\\[\\];:,.]{1,100}")| {
        let mut lexer = Lexer::new(&input);
        let mut expected_start = 0usize;

        loop {
            let token = lexer.next_token();

            if token.kind == TokenKind::Eof {
                // EOF should start at the end of input
                prop_assert_eq!(token.start, input.len());
                break;
            }

            // Token should start where we expect
            prop_assert_eq!(token.start, expected_start,
                "Token at {} has gap or overlap, expected start at {}",
                token.start, expected_start);

            // Update expected start for next token
            expected_start = token.start + token.len;
        }

        // We should have covered the entire input
        prop_assert_eq!(expected_start, input.len(),
            "Tokens don't cover entire input: covered {}, total {}",
            expected_start, input.len());
    });
}

/// Test that keywords are recognized correctly
#[test]
fn keywords_are_recognized() {
    let keywords = vec![
        ("fn", TokenKind::Fn),
        ("let", TokenKind::Let),
        ("if", TokenKind::If),
        ("else", TokenKind::Else),
        ("while", TokenKind::While),
        ("for", TokenKind::For),
        ("return", TokenKind::Return),
        ("break", TokenKind::Break),
        ("continue", TokenKind::Continue),
        ("struct", TokenKind::Struct),
        ("enum", TokenKind::Enum),
        ("impl", TokenKind::Impl),
        ("trait", TokenKind::Trait),
        ("type", TokenKind::Type),
        ("const", TokenKind::Const),
        ("state", TokenKind::State),
        ("pub", TokenKind::Pub),
        ("use", TokenKind::Use),
        ("module", TokenKind::Module),
        ("unsafe", TokenKind::Unsafe),
        ("async", TokenKind::Async),
        ("await", TokenKind::Await),
        ("match", TokenKind::Match),
        ("true", TokenKind::True),
        ("false", TokenKind::False),
        ("null", TokenKind::Null),
        ("component", TokenKind::Component),
        ("script", TokenKind::Script),
        ("block", TokenKind::Block),
    ];

    for (keyword, expected_kind) in keywords {
        let mut lexer = Lexer::new(keyword);
        let token = lexer.next_token();
        assert_eq!(token.kind, expected_kind,
            "Keyword '{}' should be {:?}, got {:?}",
            keyword, expected_kind, token.kind);
    }
}

/// Test that operators are recognized correctly
#[test]
fn operators_are_recognized() {
    let operators = vec![
        ("+", TokenKind::Plus),
        ("-", TokenKind::Minus),
        ("*", TokenKind::Star),
        ("/", TokenKind::Slash),
        ("%", TokenKind::Percent),
        ("==", TokenKind::EqEq),
        ("!=", TokenKind::Ne),
        ("<", TokenKind::Lt),
        (">", TokenKind::Gt),
        ("<=", TokenKind::Le),
        (">=", TokenKind::Ge),
        ("&&", TokenKind::AndAnd),
        ("||", TokenKind::OrOr),
        ("!", TokenKind::Bang),
        ("&", TokenKind::And),
        ("|", TokenKind::Or),
        ("^", TokenKind::Caret),
        ("<<", TokenKind::Shl),
        (">>", TokenKind::Shr),
        ("=", TokenKind::Eq),
        ("+=", TokenKind::PlusEq),
        ("-=", TokenKind::MinusEq),
        ("*=", TokenKind::StarEq),
        ("/=", TokenKind::SlashEq),
        ("->", TokenKind::Arrow),
        ("=>", TokenKind::FatArrow),
        (".", TokenKind::Dot),
        ("..", TokenKind::DotDot),
    ];

    for (op, expected_kind) in operators {
        let mut lexer = Lexer::new(op);
        let token = lexer.next_token();
        assert_eq!(token.kind, expected_kind,
            "Operator '{}' should be {:?}, got {:?}",
            op, expected_kind, token.kind);
    }
}

/// Test that identifiers can contain letters, numbers, and underscores
#[test]
fn identifiers_are_recognized() {
    proptest!(|(ident in "[a-zA-Z_][a-zA-Z0-9_]*")| {
        let mut lexer = Lexer::new(&ident);
        let token = lexer.next_token();

        // Should be either an identifier or a keyword
        prop_assert!(
            matches!(token.kind, TokenKind::Ident) ||
            is_keyword(&token.kind),
            "Expected identifier or keyword, got {:?} for input '{}'",
            token.kind, ident
        );
    });
}

/// Test that numbers are recognized
#[test]
fn numbers_are_recognized() {
    proptest!(|(num in "[0-9]+")| {
        let mut lexer = Lexer::new(&num);
        let token = lexer.next_token();

        prop_assert!(
            matches!(token.kind, TokenKind::Integer { .. }),
            "Expected integer literal, got {:?} for input '{}'",
            token.kind, num
        );
    });
}

/// Test that whitespace is recognized but preserved
#[test]
fn whitespace_is_tokenized() {
    let whitespace_inputs = vec![
        " ",
        "\t",
        "\n",
        "\r\n",
        "   ",
        "\t\t",
        "\n\n",
        " \t\n ",
    ];

    for input in whitespace_inputs {
        let mut lexer = Lexer::new(input);
        let mut has_whitespace = false;

        loop {
            let token = lexer.next_token();
            if token.kind == TokenKind::Eof {
                break;
            }
            if token.kind == TokenKind::Whitespace {
                has_whitespace = true;
            }
        }

        assert!(has_whitespace, "Expected whitespace token for input {:?}", input);
    }
}

/// Test that string literals are recognized
#[test]
fn string_literals_are_recognized() {
    let strings = vec![
        r#""""#,
        r#""hello""#,
        r#""hello world""#,
        r#""with\nnewline""#,
        r#""with\ttab""#,
        r#""with\"quote""#,
    ];

    for s in strings {
        let mut lexer = Lexer::new(s);
        let token = lexer.next_token();
        assert!(
            matches!(token.kind, TokenKind::String { .. }),
            "Expected string literal for '{}', got {:?}",
            s, token.kind
        );
    }
}

/// Test that comments are recognized
#[test]
fn comments_are_recognized() {
    let comments = vec![
        "// line comment",
        "// line comment\n",
        "/* block comment */",
        "/* multi\nline\ncomment */",
        "/* nested /* comment */ */",
    ];

    for comment in comments {
        let mut lexer = Lexer::new(comment);
        let mut found_comment = false;

        loop {
            let token = lexer.next_token();
            if token.kind == TokenKind::Eof {
                break;
            }
            if matches!(token.kind, TokenKind::LineComment | TokenKind::BlockComment { .. }) {
                found_comment = true;
            }
        }

        assert!(found_comment, "Expected comment token for '{}'", comment);
    }
}

/// Test lexer with realistic code samples
#[test]
fn lexer_handles_realistic_code() {
    let samples = vec![
        "fn main() {}",
        "let x = 42;",
        "if x > 0 { return x; }",
        "struct Point { x: i32, y: i32 }",
        "fn add(a: i32, b: i32) -> i32 { a + b }",
        "component Counter {}",
        "script DataProcessor {}",
        "unsafe block raw_ptr() -> *i32 { 0 as *i32 }",
    ];

    for sample in samples {
        let mut lexer = Lexer::new(sample);
        let mut token_count = 0;

        loop {
            let token = lexer.next_token();
            if token.kind == TokenKind::Eof {
                break;
            }
            token_count += 1;

            // Verify token is in bounds
            assert!(token.start + token.len <= sample.len(),
                "Token out of bounds in sample: '{}'", sample);
        }

        assert!(token_count > 0, "Expected tokens in sample: '{}'", sample);
    }
}

/// Test that lexer correctly handles token positions
#[test]
fn token_positions_are_correct() {
    let input = "let x = 42;";
    let mut lexer = Lexer::new(input);

    let tok1 = lexer.next_token(); // "let"
    assert_eq!(&input[tok1.start..tok1.start + tok1.len], "let");

    lexer.next_token(); // whitespace

    let tok2 = lexer.next_token(); // "x"
    assert_eq!(&input[tok2.start..tok2.start + tok2.len], "x");

    lexer.next_token(); // whitespace

    let tok3 = lexer.next_token(); // "="
    assert_eq!(&input[tok3.start..tok3.start + tok3.len], "=");

    lexer.next_token(); // whitespace

    let tok4 = lexer.next_token(); // "42"
    assert_eq!(&input[tok4.start..tok4.start + tok4.len], "42");

    let tok5 = lexer.next_token(); // ";"
    assert_eq!(&input[tok5.start..tok5.start + tok5.len], ";");
}

/// Helper function to check if a token kind is a keyword
fn is_keyword(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Fn
            | TokenKind::Let
            | TokenKind::If
            | TokenKind::Else
            | TokenKind::While
            | TokenKind::For
            | TokenKind::Return
            | TokenKind::Break
            | TokenKind::Continue
            | TokenKind::Struct
            | TokenKind::Enum
            | TokenKind::Impl
            | TokenKind::Trait
            | TokenKind::Type
            | TokenKind::Const
            | TokenKind::State
            | TokenKind::Pub
            | TokenKind::Use
            | TokenKind::Module
            | TokenKind::Unsafe
            | TokenKind::Async
            | TokenKind::Await
            | TokenKind::Match
            | TokenKind::True
            | TokenKind::False
            | TokenKind::Null
            | TokenKind::Component
            | TokenKind::Script
            | TokenKind::Block
    )
}

/// Stress test: lex a large randomly generated program
#[test]
fn stress_test_large_input() {
    proptest!(ProptestConfig::with_cases(10), |(
        // Generate a program-like string
        lines in prop::collection::vec(
            prop::string::string_regex("[a-z]+ [a-z]+ = [0-9]+;").unwrap(),
            1..100
        )
    )| {
        let input = lines.join("\n");
        let mut lexer = Lexer::new(&input);

        let mut token_count = 0;
        loop {
            let token = lexer.next_token();
            if token.kind == TokenKind::Eof {
                break;
            }
            token_count += 1;

            // Prevent infinite loops in tests
            prop_assert!(token_count < 100000, "Too many tokens generated");
        }
    });
}
