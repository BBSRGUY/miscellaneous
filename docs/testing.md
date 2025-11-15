# Blang Testing Strategy

This document describes the comprehensive testing strategy for the Blang programming language compiler and toolchain.

## Overview

The Blang project employs a multi-layered testing approach to ensure industrial-grade quality and reliability:

1. **Unit Tests** - Test individual components in isolation
2. **Property-Based Tests** - Verify invariants hold across random inputs
3. **Integration Tests** - Test interactions between components
4. **End-to-End Tests** - Compile and execute complete programs
5. **Fuzzing** - Discover edge cases through randomized testing

## Test Organization

### Directory Structure

```
crates/
├── blang_lexer/
│   ├── src/
│   │   └── lib.rs                  # Unit tests inline
│   └── tests/
│       └── property_tests.rs       # Property-based tests
├── blang_parser/
│   ├── src/
│   │   └── *.rs                    # Unit tests inline
│   └── tests/
│       ├── property_tests.rs       # Property-based tests
│       └── unsafe_blocks.rs        # Feature-specific tests
├── blang_cli/
│   └── tests/
│       ├── integration_test.rs     # CLI integration tests
│       └── e2e_wasm_tests.rs       # End-to-end WASM execution
└── ...
```

### Test Categories

#### 1. Unit Tests

Located inline in source files using `#[cfg(test)]` modules.

**Purpose**: Test individual functions and methods in isolation.

**Example**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_kind_display() {
        assert_eq!(format!("{:?}", TokenKind::Fn), "Fn");
    }
}
```

**Run with**: `cargo test -p blang_lexer --lib`

#### 2. Property-Based Tests

Located in `tests/property_tests.rs` files, using the `proptest` framework.

**Purpose**: Verify invariants hold across a wide range of randomly generated inputs.

**Key Properties Tested**:

**Lexer** (`crates/blang_lexer/tests/property_tests.rs`):
- Never panics on any input
- Token spans are contiguous and cover entire input
- Tokens can be mapped back to source text
- Keywords and operators are recognized correctly
- Handles very long identifiers without crashing

**Parser** (`crates/blang_parser/tests/property_tests.rs`):
- Never panics on any input
- Produces valid AST nodes with well-formed spans
- Handles whitespace variations correctly
- Parse is deterministic (same input → same output)
- Handles deeply nested structures
- Handles many parameters/fields without overflow

**Example**:
```rust
use proptest::prelude::*;

#[test]
fn lexer_never_panics_on_any_input() {
    proptest!(|(input in "\\PC*")| {
        let mut lexer = Lexer::new(&input);
        loop {
            let token = lexer.next_token();
            if token.kind == TokenKind::Eof {
                break;
            }
        }
    });
}
```

**Run with**: `cargo test -p blang_lexer property_tests`

#### 3. Integration Tests

Located in `tests/*.rs` files within each crate.

**Purpose**: Test interactions between multiple components.

**Examples**:
- CLI argument parsing and command execution
- Parser with specific language features (unsafe blocks, components, scripts)
- Type checker with parser output

**Run with**: `cargo test -p blang_cli --test integration_test`

#### 4. End-to-End WASM Tests

Located in `crates/blang_cli/tests/e2e_wasm_tests.rs`.

**Purpose**: Test the entire compilation pipeline from source to execution.

**Process**:
1. Write Blang source code
2. Compile to WebAssembly using the full pipeline
3. Load WASM into wasmtime runtime
4. Execute exported functions
5. Verify results match expectations

**Test Cases**:
- Simple constant returns
- Arithmetic operations (add, subtract, multiply)
- Conditional logic (if/else)
- Loops (while, for)
- Recursive functions (fibonacci, factorial)
- Boolean operations (and, or, not)
- Comparison operations (==, !=, <, >, <=, >=)
- Complex expressions with multiple operators

**Example**:
```rust
#[test]
fn test_simple_addition() {
    let source = r#"
        fn add(a: i32, b: i32) -> i32 {
            a + b
        }
    "#;

    let (mut store, instance) = compile_and_load(source)
        .expect("Compilation failed");

    let add = get_func::<(i32, i32), i32>(&mut store, &instance, "add")
        .expect("Failed to get function");

    let result = add.call(&mut store, (2, 3))
        .expect("Execution failed");

    assert_eq!(result, 5);
}
```

**Run with**: `cargo test -p blang_cli e2e_wasm_tests`

**Note**: Many E2E tests are currently marked `#[ignore]` due to known issues in the IR lowering phase. They serve as documentation of intended behavior and will be enabled as the compiler matures.

#### 5. Fuzzing (Future)

**Purpose**: Discover crashes and undefined behavior through aggressive randomized testing.

**Planned Approaches**:
- `cargo-fuzz` with libFuzzer for continuous fuzzing
- AFL++ for coverage-guided fuzzing
- Structure-aware fuzzing for generating syntactically valid programs

**Targets**:
- Lexer: Random byte sequences
- Parser: Random token sequences
- Type checker: Random well-formed ASTs
- Code generator: Random typed IR

## CI Integration

### GitHub Actions Workflow

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      # Unit tests
      - name: Run unit tests
        run: cargo test --workspace --lib

      # Integration tests
      - name: Run integration tests
        run: cargo test --workspace --tests

      # Property-based tests (with limited iterations for CI)
      - name: Run property tests
        run: cargo test --workspace property_tests
        env:
          PROPTEST_CASES: 100  # Reduced for CI speed

      # End-to-end tests (excluding ignored tests)
      - name: Run E2E tests
        run: cargo test --workspace --test e2e_wasm_tests

      # Check test coverage
      - name: Generate coverage
        run: cargo tarpaulin --out Lcov --workspace

      - name: Upload coverage
        uses: coverallsapp/github-action@v2
        with:
          github-token: ${{ secrets.GITHUB_TOKEN }}
```

### Running Tests Locally

```bash
# Run all tests (unit + integration)
cargo test --workspace

# Run only unit tests
cargo test --workspace --lib

# Run only integration tests
cargo test --workspace --tests

# Run property tests for specific crate
cargo test -p blang_lexer property_tests

# Run E2E tests (including ignored ones)
cargo test -p blang_cli e2e_wasm_tests -- --include-ignored

# Run with verbose output
cargo test --workspace -- --nocapture

# Run specific test by name
cargo test test_simple_addition

# Run tests in release mode (faster)
cargo test --workspace --release
```

### Test Configuration

**Proptest Configuration** (in test files):
```rust
use proptest::prelude::*;

// Default: 256 test cases per property
proptest! {
    #[test]
    fn my_property(input in any::<String>()) {
        // test code
    }
}

// Custom: 1000 test cases for thorough testing
proptest!(ProptestConfig::with_cases(1000), {
    #[test]
    fn thorough_property(input in any::<String>()) {
        // test code
    }
});
```

## Test Coverage Goals

| Component | Target Coverage | Current Status |
|-----------|----------------|----------------|
| Lexer | 95%+ | ✅ High coverage |
| Parser | 90%+ | ✅ High coverage |
| Type Checker | 85%+ | 🚧 In progress |
| IR Lowering | 85%+ | ⚠️ Known issues |
| Code Generation | 90%+ | ⚠️ Limited |
| Runtime | 95%+ | ❌ Not started |
| CLI | 80%+ | ✅ Good coverage |

## Known Issues and TODOs

### Current Limitations

1. **IR Lowering Infinite Loop**: Several E2E tests are disabled due to an infinite loop in the IR lowering phase. This is tracked and will be fixed in Phase 12.

2. **Type Checker Tests**: Property-based tests for the type checker are not yet implemented (coming in Phase 12).

3. **No Benchmark Tests**: Performance regression tests using `criterion` are planned but not yet implemented.

### Future Work

- [ ] Add property-based tests for type checker
- [ ] Implement fuzzing targets with `cargo-fuzz`
- [ ] Add snapshot testing for error messages using `insta`
- [ ] Create performance benchmarks with `criterion`
- [ ] Add mutation testing with `cargo-mutants`
- [ ] Implement test fixtures for common test scenarios
- [ ] Add golden file tests for code generation output

## Testing Best Practices

### Writing Good Tests

1. **Test One Thing**: Each test should verify a single behavior
2. **Clear Names**: Test names should describe what they test
3. **Arrange-Act-Assert**: Structure tests in three clear phases
4. **Independent**: Tests should not depend on each other
5. **Fast**: Unit tests should run in milliseconds
6. **Reliable**: Tests should not be flaky

### Example Test Structure

```rust
#[test]
fn test_lexer_recognizes_keywords() {
    // Arrange
    let input = "fn let if else";
    let mut lexer = Lexer::new(input);

    // Act
    let tokens: Vec<_> = std::iter::from_fn(|| {
        let tok = lexer.next_token();
        if tok.kind == TokenKind::Eof {
            None
        } else {
            Some(tok)
        }
    }).collect();

    // Assert
    assert_eq!(tokens.len(), 7); // 4 keywords + 3 whitespace
    assert_eq!(tokens[0].kind, TokenKind::Fn);
    assert_eq!(tokens[2].kind, TokenKind::Let);
    assert_eq!(tokens[4].kind, TokenKind::If);
    assert_eq!(tokens[6].kind, TokenKind::Else);
}
```

### Property Test Design

When writing property tests:

1. **Define Clear Invariants**: What must always be true?
2. **Generate Valid Inputs**: Use appropriate strategies
3. **Check Postconditions**: Verify results satisfy properties
4. **Shrinking**: Let proptest find minimal failing cases
5. **Limit Scope**: Don't generate unreasonably large inputs

### Integration Test Design

When writing integration tests:

1. **Test Real Scenarios**: Use realistic code examples
2. **Cover Edge Cases**: Empty inputs, large inputs, special characters
3. **Test Error Paths**: Verify errors are handled correctly
4. **Minimize Dependencies**: Use mocks/stubs where appropriate

### E2E Test Design

When writing E2E tests:

1. **Test User Workflows**: Compile → Execute → Verify
2. **Use Realistic Programs**: Test actual use cases
3. **Verify Semantics**: Check behavior, not just compilation
4. **Performance Bounds**: Ensure reasonable execution time
5. **Resource Limits**: Test with constrained memory/stack

## Debugging Test Failures

### Common Issues

**Test Timeout**: Increase timeout or check for infinite loops
```bash
cargo test -- --test-threads=1 --nocapture
```

**Flaky Tests**: Run multiple times to identify
```bash
cargo test -- --test-threads=1 --nocapture --ignored
```

**Property Test Failure**: Check the shrunk input
```bash
RUST_LOG=proptest cargo test property_tests -- --nocapture
```

**E2E Test Failure**: Enable debug output
```bash
RUST_LOG=debug cargo test e2e_wasm_tests -- --nocapture
```

### Useful Environment Variables

```bash
# Limit proptest cases for faster iteration
PROPTEST_CASES=10 cargo test

# Show test output even on success
cargo test -- --nocapture

# Run tests with backtrace on failure
RUST_BACKTRACE=1 cargo test

# Run tests in single thread for debugging
cargo test -- --test-threads=1
```

## Continuous Integration

### Test Stages in CI

1. **Fast Tests** (< 5 minutes)
   - Unit tests
   - Quick integration tests
   - Runs on every commit

2. **Full Tests** (< 15 minutes)
   - All unit and integration tests
   - Property tests with standard iterations
   - Runs on pull requests

3. **Extended Tests** (< 60 minutes)
   - Property tests with high iteration counts
   - E2E tests including slow cases
   - Fuzzing for 30 minutes
   - Runs nightly

4. **Coverage Reports**
   - Generated on main branch
   - Published to coverage service
   - Fails if coverage drops below threshold

### Quality Gates

Pull requests must pass:
- ✅ All unit tests
- ✅ All integration tests
- ✅ All property tests (standard iterations)
- ✅ Code coverage ≥ 80%
- ✅ No new compiler warnings
- ✅ Clippy lints pass

## Conclusion

The Blang testing strategy provides multiple layers of defense against bugs:

1. **Unit tests** catch basic logic errors
2. **Property tests** verify invariants and edge cases
3. **Integration tests** ensure components work together
4. **E2E tests** validate the entire compilation pipeline
5. **Fuzzing** discovers unexpected edge cases

This comprehensive approach ensures industrial-grade quality and confidence in the Blang compiler and toolchain.
