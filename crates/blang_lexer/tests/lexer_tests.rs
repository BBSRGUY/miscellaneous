//! Integration tests for the Blang lexer.

use blang_lexer::{Base, Lexer, TokenKind};

/// Helper to lex a single token and return its kind.
fn lex_single(source: &str) -> TokenKind {
    let mut lexer = Lexer::new(source);
    lexer.next_token().kind
}

/// Helper to lex all tokens (excluding trivia) and return their kinds.
fn lex_all(source: &str) -> Vec<TokenKind> {
    Lexer::new(source)
        .filter(|t| !t.kind.is_trivia())
        .map(|t| t.kind)
        .collect()
}

#[test]
fn test_all_keywords() {
    assert_eq!(lex_single("as"), TokenKind::As);
    assert_eq!(lex_single("async"), TokenKind::Async);
    assert_eq!(lex_single("await"), TokenKind::Await);
    assert_eq!(lex_single("break"), TokenKind::Break);
    assert_eq!(lex_single("case"), TokenKind::Case);
    assert_eq!(lex_single("catch"), TokenKind::Catch);
    assert_eq!(lex_single("component"), TokenKind::Component);
    assert_eq!(lex_single("computed"), TokenKind::Computed);
    assert_eq!(lex_single("const"), TokenKind::Const);
    assert_eq!(lex_single("continue"), TokenKind::Continue);
    assert_eq!(lex_single("default"), TokenKind::Default);
    assert_eq!(lex_single("defer"), TokenKind::Defer);
    assert_eq!(lex_single("depends_on"), TokenKind::DependsOn);
    assert_eq!(lex_single("do"), TokenKind::Do);
    assert_eq!(lex_single("else"), TokenKind::Else);
    assert_eq!(lex_single("enum"), TokenKind::Enum);
    assert_eq!(lex_single("export"), TokenKind::Export);
    assert_eq!(lex_single("false"), TokenKind::False);
    assert_eq!(lex_single("fn"), TokenKind::Fn);
    assert_eq!(lex_single("for"), TokenKind::For);
    assert_eq!(lex_single("if"), TokenKind::If);
    assert_eq!(lex_single("impl"), TokenKind::Impl);
    assert_eq!(lex_single("import"), TokenKind::Import);
    assert_eq!(lex_single("in"), TokenKind::In);
    assert_eq!(lex_single("interface"), TokenKind::Interface);
    assert_eq!(lex_single("job"), TokenKind::Job);
    assert_eq!(lex_single("let"), TokenKind::Let);
    assert_eq!(lex_single("loop"), TokenKind::Loop);
    assert_eq!(lex_single("match"), TokenKind::Match);
    assert_eq!(lex_single("module"), TokenKind::Module);
    assert_eq!(lex_single("mut"), TokenKind::Mut);
    assert_eq!(lex_single("null"), TokenKind::Null);
    assert_eq!(lex_single("on_mount"), TokenKind::OnMount);
    assert_eq!(lex_single("on_update"), TokenKind::OnUpdate);
    assert_eq!(lex_single("on_unmount"), TokenKind::OnUnmount);
    assert_eq!(lex_single("parallel"), TokenKind::Parallel);
    assert_eq!(lex_single("props"), TokenKind::Props);
    assert_eq!(lex_single("pub"), TokenKind::Pub);
    assert_eq!(lex_single("return"), TokenKind::Return);
    assert_eq!(lex_single("script"), TokenKind::Script);
    assert_eq!(lex_single("self"), TokenKind::SelfKw);
    assert_eq!(lex_single("signal"), TokenKind::Signal);
    assert_eq!(lex_single("state"), TokenKind::State);
    assert_eq!(lex_single("step"), TokenKind::Step);
    assert_eq!(lex_single("struct"), TokenKind::Struct);
    assert_eq!(lex_single("style"), TokenKind::Style);
    assert_eq!(lex_single("super"), TokenKind::Super);
    assert_eq!(lex_single("trait"), TokenKind::Trait);
    assert_eq!(lex_single("true"), TokenKind::True);
    assert_eq!(lex_single("type"), TokenKind::Type);
    assert_eq!(lex_single("typeof"), TokenKind::Typeof);
    assert_eq!(lex_single("unsafe"), TokenKind::Unsafe);
    assert_eq!(lex_single("use"), TokenKind::Use);
    assert_eq!(lex_single("view"), TokenKind::View);
    assert_eq!(lex_single("while"), TokenKind::While);
    assert_eq!(lex_single("yield"), TokenKind::Yield);
}

#[test]
fn test_contextual_keywords() {
    assert_eq!(lex_single("actor"), TokenKind::Actor);
    assert_eq!(lex_single("channel"), TokenKind::Channel);
    assert_eq!(lex_single("receiver"), TokenKind::Receiver);
    assert_eq!(lex_single("send"), TokenKind::Send);
    assert_eq!(lex_single("get"), TokenKind::Get);
    assert_eq!(lex_single("set"), TokenKind::Set);
}

#[test]
fn test_identifiers() {
    assert_eq!(lex_single("foo"), TokenKind::Ident);
    assert_eq!(lex_single("_bar"), TokenKind::Ident);
    assert_eq!(lex_single("baz123"), TokenKind::Ident);
    assert_eq!(lex_single("CamelCase"), TokenKind::Ident);
    assert_eq!(lex_single("snake_case"), TokenKind::Ident);
    assert_eq!(lex_single("SCREAMING_SNAKE_CASE"), TokenKind::Ident);
}

#[test]
fn test_decimal_integers() {
    assert_eq!(
        lex_single("0"),
        TokenKind::Integer {
            base: Base::Decimal,
            empty: false
        }
    );
    assert_eq!(
        lex_single("42"),
        TokenKind::Integer {
            base: Base::Decimal,
            empty: false
        }
    );
    assert_eq!(
        lex_single("1234567890"),
        TokenKind::Integer {
            base: Base::Decimal,
            empty: false
        }
    );
    assert_eq!(
        lex_single("1_000_000"),
        TokenKind::Integer {
            base: Base::Decimal,
            empty: false
        }
    );
}

#[test]
fn test_hex_integers() {
    assert_eq!(
        lex_single("0xFF"),
        TokenKind::Integer {
            base: Base::Hexadecimal,
            empty: false
        }
    );
    assert_eq!(
        lex_single("0xDEADBEEF"),
        TokenKind::Integer {
            base: Base::Hexadecimal,
            empty: false
        }
    );
    assert_eq!(
        lex_single("0x1234_5678"),
        TokenKind::Integer {
            base: Base::Hexadecimal,
            empty: false
        }
    );
}

#[test]
fn test_octal_integers() {
    assert_eq!(
        lex_single("0o777"),
        TokenKind::Integer {
            base: Base::Octal,
            empty: false
        }
    );
    assert_eq!(
        lex_single("0o123_456"),
        TokenKind::Integer {
            base: Base::Octal,
            empty: false
        }
    );
}

#[test]
fn test_binary_integers() {
    assert_eq!(
        lex_single("0b1010"),
        TokenKind::Integer {
            base: Base::Binary,
            empty: false
        }
    );
    assert_eq!(
        lex_single("0b1111_0000"),
        TokenKind::Integer {
            base: Base::Binary,
            empty: false
        }
    );
}

#[test]
fn test_empty_numeric_literals() {
    assert_eq!(
        lex_single("0x"),
        TokenKind::Integer {
            base: Base::Hexadecimal,
            empty: true
        }
    );
    assert_eq!(
        lex_single("0o"),
        TokenKind::Integer {
            base: Base::Octal,
            empty: true
        }
    );
    assert_eq!(
        lex_single("0b"),
        TokenKind::Integer {
            base: Base::Binary,
            empty: true
        }
    );
}

#[test]
fn test_floats() {
    assert_eq!(lex_single("3.14"), TokenKind::Float { empty: false });
    assert_eq!(lex_single("0.5"), TokenKind::Float { empty: false });
    assert_eq!(lex_single("1.0"), TokenKind::Float { empty: false });
    assert_eq!(lex_single("123.456"), TokenKind::Float { empty: false });
    assert_eq!(lex_single("1_000.5"), TokenKind::Float { empty: false });
}

#[test]
fn test_floats_with_exponent() {
    assert_eq!(lex_single("1e10"), TokenKind::Float { empty: false });
    assert_eq!(lex_single("1E10"), TokenKind::Float { empty: false });
    assert_eq!(lex_single("1e+10"), TokenKind::Float { empty: false });
    assert_eq!(lex_single("1e-10"), TokenKind::Float { empty: false });
    assert_eq!(lex_single("1.5e10"), TokenKind::Float { empty: false });
    assert_eq!(lex_single("1.5E-5"), TokenKind::Float { empty: false });
}

#[test]
fn test_strings() {
    assert_eq!(
        lex_single(r#""hello""#),
        TokenKind::String {
            raw: false,
            terminated: true
        }
    );
    assert_eq!(
        lex_single(r#""multi
line""#),
        TokenKind::String {
            raw: false,
            terminated: true
        }
    );
    assert_eq!(
        lex_single(r#""with \"escapes\"""#),
        TokenKind::String {
            raw: false,
            terminated: true
        }
    );
}

#[test]
fn test_raw_strings() {
    assert_eq!(
        lex_single(r#"r"raw string""#),
        TokenKind::String {
            raw: true,
            terminated: true
        }
    );
    assert_eq!(
        lex_single(r#"r"no \n escapes""#),
        TokenKind::String {
            raw: true,
            terminated: true
        }
    );
}

#[test]
fn test_unterminated_strings() {
    assert_eq!(
        lex_single(r#""unterminated"#),
        TokenKind::String {
            raw: false,
            terminated: false
        }
    );
    assert_eq!(
        lex_single(r#"r"unterminated"#),
        TokenKind::String {
            raw: true,
            terminated: false
        }
    );
}

#[test]
fn test_characters() {
    assert_eq!(lex_single("'a'"), TokenKind::Char { terminated: true });
    assert_eq!(lex_single("'Z'"), TokenKind::Char { terminated: true });
    assert_eq!(lex_single("'0'"), TokenKind::Char { terminated: true });
    assert_eq!(lex_single("'\\n'"), TokenKind::Char { terminated: true });
    assert_eq!(lex_single("'\\t'"), TokenKind::Char { terminated: true });
    assert_eq!(lex_single("'\\\\'"), TokenKind::Char { terminated: true });
    assert_eq!(lex_single("'\\''"), TokenKind::Char { terminated: true });
}

#[test]
fn test_unterminated_characters() {
    assert_eq!(lex_single("'a"), TokenKind::Char { terminated: false });
    assert_eq!(lex_single("'\\n"), TokenKind::Char { terminated: false });
}

#[test]
fn test_template_strings() {
    assert_eq!(
        lex_single("`hello`"),
        TokenKind::TemplateString { terminated: true }
    );
    assert_eq!(
        lex_single("`hello ${name}`"),
        TokenKind::TemplateString { terminated: true }
    );
    assert_eq!(
        lex_single("`multi ${a} ${b} interpolations`"),
        TokenKind::TemplateString { terminated: true }
    );
}

#[test]
fn test_unterminated_template_strings() {
    assert_eq!(
        lex_single("`unterminated"),
        TokenKind::TemplateString { terminated: false }
    );
}

#[test]
fn test_arithmetic_operators() {
    assert_eq!(lex_single("+"), TokenKind::Plus);
    assert_eq!(lex_single("-"), TokenKind::Minus);
    assert_eq!(lex_single("*"), TokenKind::Star);
    assert_eq!(lex_single("/"), TokenKind::Slash);
    assert_eq!(lex_single("%"), TokenKind::Percent);
}

#[test]
fn test_comparison_operators() {
    assert_eq!(lex_single("=="), TokenKind::EqEq);
    assert_eq!(lex_single("!="), TokenKind::Ne);
    assert_eq!(lex_single("<"), TokenKind::Lt);
    assert_eq!(lex_single("<="), TokenKind::Le);
    assert_eq!(lex_single(">"), TokenKind::Gt);
    assert_eq!(lex_single(">="), TokenKind::Ge);
}

#[test]
fn test_logical_operators() {
    assert_eq!(lex_single("&&"), TokenKind::AndAnd);
    assert_eq!(lex_single("||"), TokenKind::OrOr);
    assert_eq!(lex_single("!"), TokenKind::Bang);
}

#[test]
fn test_bitwise_operators() {
    assert_eq!(lex_single("&"), TokenKind::And);
    assert_eq!(lex_single("|"), TokenKind::Or);
    assert_eq!(lex_single("^"), TokenKind::Caret);
    assert_eq!(lex_single("~"), TokenKind::Tilde);
    assert_eq!(lex_single("<<"), TokenKind::Shl);
    assert_eq!(lex_single(">>"), TokenKind::Shr);
}

#[test]
fn test_assignment_operators() {
    assert_eq!(lex_single("="), TokenKind::Eq);
    assert_eq!(lex_single("+="), TokenKind::PlusEq);
    assert_eq!(lex_single("-="), TokenKind::MinusEq);
    assert_eq!(lex_single("*="), TokenKind::StarEq);
    assert_eq!(lex_single("/="), TokenKind::SlashEq);
    assert_eq!(lex_single("%="), TokenKind::PercentEq);
    assert_eq!(lex_single("&="), TokenKind::AndEq);
    assert_eq!(lex_single("|="), TokenKind::OrEq);
    assert_eq!(lex_single("^="), TokenKind::CaretEq);
    assert_eq!(lex_single("<<="), TokenKind::ShlEq);
    assert_eq!(lex_single(">>="), TokenKind::ShrEq);
}

#[test]
fn test_delimiters() {
    assert_eq!(lex_single("("), TokenKind::OpenParen);
    assert_eq!(lex_single(")"), TokenKind::CloseParen);
    assert_eq!(lex_single("["), TokenKind::OpenBracket);
    assert_eq!(lex_single("]"), TokenKind::CloseBracket);
    assert_eq!(lex_single("{"), TokenKind::OpenBrace);
    assert_eq!(lex_single("}"), TokenKind::CloseBrace);
}

#[test]
fn test_punctuation() {
    assert_eq!(lex_single(";"), TokenKind::Semi);
    assert_eq!(lex_single(":"), TokenKind::Colon);
    assert_eq!(lex_single(","), TokenKind::Comma);
    assert_eq!(lex_single("."), TokenKind::Dot);
    assert_eq!(lex_single(".."), TokenKind::DotDot);
    assert_eq!(lex_single("..="), TokenKind::DotDotEq);
    assert_eq!(lex_single("->"), TokenKind::Arrow);
    assert_eq!(lex_single("=>"), TokenKind::FatArrow);
    assert_eq!(lex_single("::"), TokenKind::ColonColon);
    assert_eq!(lex_single("@"), TokenKind::At);
    assert_eq!(lex_single("#"), TokenKind::Hash);
    assert_eq!(lex_single("$"), TokenKind::Dollar);
}

#[test]
fn test_line_comments() {
    assert_eq!(lex_single("// comment"), TokenKind::LineComment);
    assert_eq!(
        lex_single("// comment with symbols !@#$%"),
        TokenKind::LineComment
    );
}

#[test]
fn test_block_comments() {
    assert_eq!(
        lex_single("/* comment */"),
        TokenKind::BlockComment { terminated: true }
    );
    assert_eq!(
        lex_single("/* multi\nline\ncomment */"),
        TokenKind::BlockComment { terminated: true }
    );
}

#[test]
fn test_nested_block_comments() {
    assert_eq!(
        lex_single("/* outer /* inner */ outer */"),
        TokenKind::BlockComment { terminated: true }
    );
    assert_eq!(
        lex_single("/* /* /* deeply */ nested */ comment */"),
        TokenKind::BlockComment { terminated: true }
    );
}

#[test]
fn test_unterminated_block_comments() {
    assert_eq!(
        lex_single("/* unterminated"),
        TokenKind::BlockComment { terminated: false }
    );
    assert_eq!(
        lex_single("/* /* nested unterminated"),
        TokenKind::BlockComment { terminated: false }
    );
}

#[test]
fn test_function_declaration() {
    let tokens = lex_all("fn add(a: i32, b: i32) -> i32 { a + b }");
    assert_eq!(
        tokens,
        vec![
            TokenKind::Fn,
            TokenKind::Ident,
            TokenKind::OpenParen,
            TokenKind::Ident,
            TokenKind::Colon,
            TokenKind::Ident,
            TokenKind::Comma,
            TokenKind::Ident,
            TokenKind::Colon,
            TokenKind::Ident,
            TokenKind::CloseParen,
            TokenKind::Arrow,
            TokenKind::Ident,
            TokenKind::OpenBrace,
            TokenKind::Ident,
            TokenKind::Plus,
            TokenKind::Ident,
            TokenKind::CloseBrace,
        ]
    );
}

#[test]
fn test_component_declaration() {
    let source = r#"
        component Counter {
            state {
                count: Signal<i32> = signal(0);
            }
            view {
                <button @click={increment}>Click</button>
            }
        }
    "#;
    let tokens = lex_all(source);
    assert!(tokens.contains(&TokenKind::Component));
    assert!(tokens.contains(&TokenKind::State));
    assert!(tokens.contains(&TokenKind::View));
    assert!(tokens.contains(&TokenKind::Signal));
}

#[test]
fn test_script_declaration() {
    let source = r#"
        script Pipeline {
            job extract {
                step fetch {
                    return data;
                }
            }
        }
    "#;
    let tokens = lex_all(source);
    assert!(tokens.contains(&TokenKind::Script));
    assert!(tokens.contains(&TokenKind::Job));
    assert!(tokens.contains(&TokenKind::Step));
}

#[test]
fn test_module_declaration() {
    let source = r#"
        module math {
            pub fn add(a: i32, b: i32) -> i32 {
                a + b
            }
        }
    "#;
    let tokens = lex_all(source);
    assert!(tokens.contains(&TokenKind::Module));
    assert!(tokens.contains(&TokenKind::Pub));
    assert!(tokens.contains(&TokenKind::Fn));
}

#[test]
fn test_unsafe_block() {
    let source = r#"
        unsafe {
            let ptr = &x as *const i32;
        }
    "#;
    let tokens = lex_all(source);
    assert!(tokens.contains(&TokenKind::Unsafe));
    assert!(tokens.contains(&TokenKind::Let));
    assert!(tokens.contains(&TokenKind::As));
}

#[test]
fn test_generic_function() {
    let source = "fn identity<T>(x: T) -> T { x }";
    let tokens = lex_all(source);
    assert!(tokens.contains(&TokenKind::Fn));
    assert!(tokens.contains(&TokenKind::Lt));
    assert!(tokens.contains(&TokenKind::Gt));
    assert!(tokens.contains(&TokenKind::Arrow));
}

#[test]
fn test_match_expression() {
    let source = r#"
        match value {
            Some(x) => x * 2,
            None => 0,
        }
    "#;
    let tokens = lex_all(source);
    assert!(tokens.contains(&TokenKind::Match));
    assert!(tokens.contains(&TokenKind::FatArrow));
}

#[test]
fn test_complex_expression() {
    let source = "let result = (a + b) * c / d - e % f;";
    let tokens = lex_all(source);
    assert!(tokens.contains(&TokenKind::Let));
    assert!(tokens.contains(&TokenKind::Eq));
    assert!(tokens.contains(&TokenKind::Plus));
    assert!(tokens.contains(&TokenKind::Star));
    assert!(tokens.contains(&TokenKind::Slash));
    assert!(tokens.contains(&TokenKind::Minus));
    assert!(tokens.contains(&TokenKind::Percent));
    assert!(tokens.contains(&TokenKind::Semi));
}

#[test]
fn test_ranges() {
    let source = "0..10, 0..=10, ..10, 0..";
    let tokens = lex_all(source);
    assert!(tokens.contains(&TokenKind::DotDot));
    assert!(tokens.contains(&TokenKind::DotDotEq));
}

#[test]
fn test_path_expressions() {
    let source = "std::collections::Vec";
    let tokens = lex_all(source);
    assert_eq!(
        tokens,
        vec![
            TokenKind::Ident,
            TokenKind::ColonColon,
            TokenKind::Ident,
            TokenKind::ColonColon,
            TokenKind::Ident,
        ]
    );
}

#[test]
fn test_mixed_content() {
    let source = r#"
        // Function to calculate fibonacci
        fn fibonacci(n: u64) -> u64 {
            if n <= 1 {
                return n;
            }
            /* recursive call */
            fibonacci(n - 1) + fibonacci(n - 2)
        }
    "#;
    let tokens = lex_all(source);
    assert!(tokens.len() > 20);
    assert!(tokens.contains(&TokenKind::Fn));
    assert!(tokens.contains(&TokenKind::If));
    assert!(tokens.contains(&TokenKind::Return));
}

#[test]
fn test_error_tokens() {
    assert!(lex_single("0x").is_error());
    assert!(lex_single(r#""unterminated"#).is_error());
    assert!(lex_single("'unterminated").is_error());
    assert!(lex_single("/* unterminated").is_error());
}

#[test]
fn test_token_positions() {
    let source = "let x = 42;";
    let mut lexer = Lexer::new(source);

    let token1 = lexer.next_token();
    assert_eq!(token1.kind, TokenKind::Let);
    assert_eq!(token1.start, 0);
    assert_eq!(token1.len, 3);

    lexer.next_token(); // whitespace

    let token2 = lexer.next_token();
    assert_eq!(token2.kind, TokenKind::Ident);
    assert_eq!(token2.start, 4);
    assert_eq!(token2.len, 1);
}

#[test]
fn test_empty_source() {
    let mut lexer = Lexer::new("");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Eof);
}

#[test]
fn test_unicode_identifiers() {
    assert_eq!(lex_single("hello_世界"), TokenKind::Ident);
    assert_eq!(lex_single("café"), TokenKind::Ident);
}
