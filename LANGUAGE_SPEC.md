# Blang Language Specification v0.1

**Status**: Draft
**Date**: 2025-01-15
**Authors**: Blang Core Team

---

## Table of Contents

1. [Introduction](#introduction)
2. [Design Principles](#design-principles)
3. [Execution Modes](#execution-modes)
4. [Lexical Structure](#lexical-structure)
5. [Type System](#type-system)
6. [Values and Literals](#values-and-literals)
7. [Expressions](#expressions)
8. [Statements](#statements)
9. [Functions](#functions)
10. [Modules](#modules)
11. [Component Mode](#component-mode)
12. [Script Mode](#script-mode)
13. [Unsafe Blocks](#unsafe-blocks)
14. [Concurrency Model](#concurrency-model)
15. [Reactivity System](#reactivity-system)
16. [Memory Model](#memory-model)
17. [Compilation Model](#compilation-model)
18. [Standard Library](#standard-library)
19. [Interoperability](#interoperability)
20. [Appendix](#appendix)

---

## 1. Introduction

Blang is a browser-first, multi-paradigm programming language designed to unify UI development, application logic, orchestration, and performance-critical code in a single coherent language. It compiles primarily to WebAssembly with minimal JavaScript glue for DOM and Web API interactions.

### 1.1 Goals

- **Unified Language**: Single language for UI, styles, logic, and performance-critical code
- **Type Safety**: Strong, static typing with sophisticated type inference
- **Performance**: WebAssembly-first compilation with optimized code generation
- **Developer Experience**: Clear error messages, fast compilation, integrated tooling
- **Web-Native**: First-class support for DOM, Web APIs, and modern web patterns
- **Extensibility**: Support for future targets (Node.js, WASI, edge runtimes)

### 1.2 Non-Goals

- Dynamic typing or gradual typing
- Runtime reflection (compile-time only)
- Backwards compatibility with JavaScript semantics
- Support for legacy browsers (targets modern evergreen browsers)

---

## 2. Design Principles

1. **Explicitness over Implicitness**: Code should be clear about its intent
2. **Safety by Default, Performance by Choice**: Unsafe operations require explicit `unsafe` blocks
3. **Zero-Cost Abstractions**: High-level constructs compile to efficient code
4. **Composability**: Small, focused primitives that combine well
5. **Locality**: Related code should be co-located (UI + styles + logic)
6. **Predictability**: Behavior should be deterministic and easy to reason about

---

## 3. Execution Modes

Blang supports three distinct execution modes, each optimized for different use cases:

### 3.1 Component Mode

Component mode is used for building reactive UI components. Files use the `.blang` extension.

**Characteristics**:
- Declarative UI syntax (HTML-inspired)
- Co-located styles (CSS-inspired)
- Reactive state management via signals
- Event handling and lifecycle hooks
- Compiled to WASM + DOM bindings

**File Structure**:
```blang
component ButtonCounter {
  state {
    count: Signal<i32> = signal(0);
  }

  view {
    <div class="container">
      <button @click={increment}>
        Clicked {count} times
      </button>
    </div>
  }

  style {
    .container {
      padding: 1rem;
      background: #f0f0f0;
    }
  }

  fn increment() {
    count.update(|c| c + 1);
  }
}
```

### 3.2 Script Mode

Script mode is for orchestration, automation, and job-based workflows. Files use the `.bs` extension.

**Characteristics**:
- REXX-inspired syntax for sequential and parallel job execution
- Built-in scheduling and dependency management
- Error handling and retry logic
- Progress tracking and logging
- Can invoke modules and components

**File Structure**:
```blang
script DataPipeline {
  job extract {
    parallel {
      step fetch_users {
        let users = http.get("/api/users");
        return users;
      }

      step fetch_orders {
        let orders = http.get("/api/orders");
        return orders;
      }
    }
  }

  job transform depends_on(extract) {
    step merge_data {
      let users = extract.fetch_users.result;
      let orders = extract.fetch_orders.result;
      return join_data(users, orders);
    }
  }

  job load depends_on(transform) {
    step save_to_db {
      db.save(transform.merge_data.result);
    }
  }
}
```

### 3.3 Module Mode

Module mode is for pure logic, data structures, algorithms, and performance-critical code. Files use the `.bl` extension.

**Characteristics**:
- Pure functions and data types
- Support for `unsafe` blocks for low-level operations
- No DOM or Web API access (fully isolated)
- Compiled to optimized WASM
- Can be imported by components and scripts

**File Structure**:
```blang
module math {
  export fn fibonacci(n: u64) -> u64 {
    if n <= 1 {
      return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
  }

  export fn fast_sort<T: Ord>(arr: &mut [T]) {
    unsafe {
      // Direct WASM memory operations for performance
      let ptr = arr.as_mut_ptr();
      let len = arr.len();
      quicksort_unchecked(ptr, 0, len - 1);
    }
  }

  unsafe fn quicksort_unchecked<T: Ord>(ptr: *mut T, low: usize, high: usize) {
    // Low-level sorting implementation
    if low < high {
      let pivot = partition(ptr, low, high);
      quicksort_unchecked(ptr, low, pivot - 1);
      quicksort_unchecked(ptr, pivot + 1, high);
    }
  }
}
```

---

## 4. Lexical Structure

### 4.1 Character Set

Blang source files are UTF-8 encoded. Identifiers may contain Unicode alphanumeric characters and underscores.

### 4.2 Comments

```blang
// Single-line comment

/*
   Multi-line comment
   can span multiple lines
*/

/**
 * Documentation comment
 * Used for generating API documentation
 */
```

### 4.3 Keywords

**Reserved Keywords**:
```
as          break       case        catch       component   const
continue    default     do          else        enum        export
false       fn          for         if          impl        import
in          interface   job         let         loop        match
module      mut         null        parallel    pub         return
script      self        signal      state       step        struct
style       super       trait       true        type        typeof
unsafe      use         view        while       yield
```

**Contextual Keywords** (only keywords in specific contexts):
```
actor       channel     depends_on  receiver    send
get         set         async       await       defer
```

### 4.4 Identifiers

```
identifier: [a-zA-Z_][a-zA-Z0-9_]*
```

- Must start with a letter or underscore
- Can contain letters, digits, and underscores
- Cannot be a reserved keyword
- Case-sensitive

**Conventions**:
- `snake_case` for variables, functions, and modules
- `PascalCase` for types, traits, and components
- `SCREAMING_SNAKE_CASE` for constants

### 4.5 Literals

See [Values and Literals](#values-and-literals) section.

---

## 5. Type System

Blang has a strong, static type system with sophisticated type inference.

### 5.1 Primitive Types

| Type | Description | Size | Range |
|------|-------------|------|-------|
| `i8` | Signed 8-bit integer | 1 byte | -128 to 127 |
| `i16` | Signed 16-bit integer | 2 bytes | -32,768 to 32,767 |
| `i32` | Signed 32-bit integer | 4 bytes | -2^31 to 2^31-1 |
| `i64` | Signed 64-bit integer | 8 bytes | -2^63 to 2^63-1 |
| `u8` | Unsigned 8-bit integer | 1 byte | 0 to 255 |
| `u16` | Unsigned 16-bit integer | 2 bytes | 0 to 65,535 |
| `u32` | Unsigned 32-bit integer | 4 bytes | 0 to 2^32-1 |
| `u64` | Unsigned 64-bit integer | 8 bytes | 0 to 2^64-1 |
| `f32` | 32-bit floating point | 4 bytes | IEEE 754 single |
| `f64` | 64-bit floating point | 8 bytes | IEEE 754 double |
| `bool` | Boolean | 1 byte | `true` or `false` |
| `char` | Unicode scalar value | 4 bytes | Any Unicode scalar |
| `str` | String slice | ptr + len | UTF-8 encoded |
| `()` | Unit type | 0 bytes | Single value `()` |
| `never` | Never type | N/A | No values |

### 5.2 Compound Types

#### 5.2.1 Arrays

Fixed-size, homogeneous collections:

```blang
let arr: [i32; 5] = [1, 2, 3, 4, 5];
let matrix: [[f64; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
```

#### 5.2.2 Slices

Dynamic-size views into arrays:

```blang
let slice: &[i32] = &arr[1..4];
let mut_slice: &mut [i32] = &mut arr[..];
```

#### 5.2.3 Tuples

Fixed-size, heterogeneous collections:

```blang
let pair: (i32, str) = (42, "answer");
let triple: (f64, f64, f64) = (1.0, 2.0, 3.0);
let unit: () = ();
```

#### 5.2.4 Structs

Named product types:

```blang
struct Point {
  x: f64,
  y: f64,
}

struct Person {
  name: str,
  age: u8,
  email: Option<str>,
}
```

#### 5.2.5 Enums

Sum types (tagged unions):

```blang
enum Option<T> {
  Some(T),
  None,
}

enum Result<T, E> {
  Ok(T),
  Err(E),
}

enum Message {
  Quit,
  Move { x: i32, y: i32 },
  Write(str),
  ChangeColor(u8, u8, u8),
}
```

### 5.3 Type Constructors

#### 5.3.1 References

Blang uses borrowed references similar to Rust:

```blang
let x: i32 = 42;
let r: &i32 = &x;        // immutable reference
let m: &mut i32 = &mut x; // mutable reference
```

**Rules**:
- At any time, you can have either one mutable reference OR any number of immutable references
- References must always be valid (no dangling references)
- References cannot outlive the data they point to

#### 5.3.2 Pointers (Unsafe)

Raw pointers for unsafe code:

```blang
unsafe {
  let ptr: *const i32 = &x as *const i32;
  let mut_ptr: *mut i32 = &mut x as *mut i32;
}
```

#### 5.3.3 Option

Built-in optional value type:

```blang
let maybe_value: Option<i32> = Some(42);
let no_value: Option<i32> = None;
```

#### 5.3.4 Result

Built-in error handling type:

```blang
fn divide(a: f64, b: f64) -> Result<f64, str> {
  if b == 0.0 {
    return Err("Division by zero");
  }
  return Ok(a / b);
}
```

### 5.4 Generic Types

Blang supports parametric polymorphism:

```blang
struct Container<T> {
  value: T,
}

fn identity<T>(x: T) -> T {
  return x;
}

fn map<T, U>(opt: Option<T>, f: fn(T) -> U) -> Option<U> {
  match opt {
    Some(val) => Some(f(val)),
    None => None,
  }
}
```

### 5.5 Traits

Interfaces for shared behavior:

```blang
trait Display {
  fn display(&self) -> str;
}

trait Eq {
  fn eq(&self, other: &Self) -> bool;
}

trait Ord: Eq {
  fn cmp(&self, other: &Self) -> Ordering;
}

impl Display for Point {
  fn display(&self) -> str {
    return `Point(${self.x}, ${self.y})`;
  }
}
```

### 5.6 Associated Types

```blang
trait Iterator {
  type Item;

  fn next(&mut self) -> Option<Self::Item>;
}

impl Iterator for Range {
  type Item = i32;

  fn next(&mut self) -> Option<i32> {
    // implementation
  }
}
```

### 5.7 Type Inference

Blang performs bidirectional type inference:

```blang
let x = 42;              // inferred as i32
let y = 3.14;            // inferred as f64
let vec = [1, 2, 3];     // inferred as [i32; 3]
let result = divide(10.0, 2.0); // inferred from divide's return type
```

Explicit annotations can override inference:

```blang
let x: i64 = 42;         // explicitly i64
let y: f32 = 3.14;       // explicitly f32
```

### 5.8 Type Aliases

```blang
type Kilometers = f64;
type Result<T> = Result<T, str>;
type Point2D = (f64, f64);
```

### 5.9 Phantom Types

For compile-time type safety without runtime overhead:

```blang
struct Marker<T> {
  _phantom: PhantomData<T>,
}
```

---

## 6. Values and Literals

### 6.1 Integer Literals

```blang
let decimal = 1234;
let hex = 0xFF;
let octal = 0o77;
let binary = 0b1010;
let with_underscores = 1_000_000;
let typed = 42u64;
```

### 6.2 Floating-Point Literals

```blang
let pi = 3.14159;
let scientific = 1.23e-4;
let typed = 2.5f32;
```

### 6.3 Boolean Literals

```blang
let yes = true;
let no = false;
```

### 6.4 Character Literals

```blang
let letter = 'a';
let emoji = '😀';
let newline = '\n';
let unicode = '\u{1F600}';
```

### 6.5 String Literals

```blang
let simple = "Hello, world!";
let multiline = "This is a
multiline string";
let escaped = "Line 1\nLine 2\tTabbed";
let raw = r"C:\Users\path\to\file";
let unicode = "Hello, 世界! 🌍";
```

### 6.6 Template Strings

```blang
let name = "Alice";
let age = 30;
let greeting = `Hello, ${name}! You are ${age} years old.`;
```

### 6.7 Array Literals

```blang
let numbers = [1, 2, 3, 4, 5];
let empty: [i32; 0] = [];
let repeated = [0; 100];  // array of 100 zeros
```

### 6.8 Tuple Literals

```blang
let pair = (1, "one");
let triple = (1.0, 2.0, 3.0);
let nested = ((1, 2), (3, 4));
```

### 6.9 Struct Literals

```blang
let point = Point { x: 1.0, y: 2.0 };
let person = Person {
  name: "Alice",
  age: 30,
  email: Some("alice@example.com")
};

// Shorthand for field names
let x = 1.0;
let y = 2.0;
let point = Point { x, y };
```

### 6.10 Enum Literals

```blang
let some_value = Some(42);
let no_value = None;
let ok_result = Ok("success");
let err_result = Err("failed");
let message = Move { x: 10, y: 20 };
```

---

## 7. Expressions

All expressions have a type and produce a value.

### 7.1 Literal Expressions

```blang
42
3.14
true
"hello"
[1, 2, 3]
```

### 7.2 Path Expressions

```blang
x
module::function
Type::associated_function
self.field
```

### 7.3 Operator Expressions

#### 7.3.1 Arithmetic Operators

```blang
a + b   // addition
a - b   // subtraction
a * b   // multiplication
a / b   // division
a % b   // remainder
-a      // negation
```

#### 7.3.2 Comparison Operators

```blang
a == b  // equality
a != b  // inequality
a < b   // less than
a <= b  // less than or equal
a > b   // greater than
a >= b  // greater than or equal
```

#### 7.3.3 Logical Operators

```blang
a && b  // logical AND (short-circuiting)
a || b  // logical OR (short-circuiting)
!a      // logical NOT
```

#### 7.3.4 Bitwise Operators

```blang
a & b   // bitwise AND
a | b   // bitwise OR
a ^ b   // bitwise XOR
!a      // bitwise NOT
a << b  // left shift
a >> b  // right shift
```

#### 7.3.5 Assignment Operators

```blang
a = b
a += b
a -= b
a *= b
a /= b
a %=b
a &= b
a |= b
a ^= b
a <<= b
a >>= b
```

### 7.4 Call Expressions

```blang
function(arg1, arg2)
method.call()
Type::associated_function(arg)
```

### 7.5 Method Call Expressions

```blang
object.method(args)
array.len()
string.contains("pattern")
```

### 7.6 Field Access Expressions

```blang
point.x
person.name
tuple.0
```

### 7.7 Index Expressions

```blang
array[0]
matrix[i][j]
slice[1..5]
```

### 7.8 Range Expressions

```blang
1..10       // exclusive end
1..=10      // inclusive end
..10        // from start
1..         // to end
..          // full range
```

### 7.9 If Expressions

```blang
let result = if condition {
  value1
} else {
  value2
};

let sign = if x > 0 {
  "positive"
} else if x < 0 {
  "negative"
} else {
  "zero"
};
```

### 7.10 Match Expressions

```blang
let message = match value {
  0 => "zero",
  1 => "one",
  2..=9 => "small",
  _ => "large",
};

let result = match option {
  Some(x) => x * 2,
  None => 0,
};

let point_description = match point {
  Point { x: 0, y: 0 } => "origin",
  Point { x, y: 0 } => `on x-axis at ${x}`,
  Point { x: 0, y } => `on y-axis at ${y}`,
  Point { x, y } => `at (${x}, ${y})`,
};
```

### 7.11 Block Expressions

```blang
{
  let x = 1;
  let y = 2;
  x + y  // final expression is the block's value
}
```

### 7.12 Loop Expressions

```blang
// Infinite loop
loop {
  if condition {
    break;
  }
}

// While loop
while condition {
  // body
}

// For loop
for item in iterator {
  // body
}

// Loop with break value
let result = loop {
  counter += 1;
  if counter == 10 {
    break counter * 2;
  }
};
```

### 7.13 Closure Expressions

```blang
let add = |a, b| a + b;
let increment = |x| { x + 1 };
let capture_example = |x| { x + captured_variable };

// With explicit types
let typed_closure: fn(i32, i32) -> i32 = |a, b| a + b;
```

### 7.14 Return Expressions

```blang
return;
return value;
return if condition { a } else { b };
```

### 7.15 Await Expressions

For async operations (future feature):

```blang
let data = fetch_data().await;
```

### 7.16 Type Cast Expressions

```blang
value as i32
float_value as u64
```

### 7.17 Reference Expressions

```blang
&x          // immutable reference
&mut x      // mutable reference
*ptr        // dereference
```

### 7.18 Struct Update Syntax

```blang
let point1 = Point { x: 1.0, y: 2.0 };
let point2 = Point { x: 3.0, ..point1 };  // y copied from point1
```

---

## 8. Statements

### 8.1 Expression Statements

Any expression followed by a semicolon:

```blang
function_call();
x = 5;
array[0] = 10;
```

### 8.2 Let Statements

```blang
let x = 5;
let mut y = 10;
let z: i32 = 15;
let (a, b) = (1, 2);  // destructuring
let Point { x, y } = point;
```

### 8.3 Item Declarations

See [Functions](#functions) and [Modules](#modules) sections.

### 8.4 Empty Statement

```blang
;  // empty statement
```

---

## 9. Functions

### 9.1 Function Declaration

```blang
fn function_name(param1: Type1, param2: Type2) -> ReturnType {
  // function body
  return value;
}

// Unit return type can be omitted
fn no_return(x: i32) {
  println(x);
}

// Expression body (no return needed)
fn add(a: i32, b: i32) -> i32 {
  a + b
}
```

### 9.2 Function Parameters

```blang
// By value
fn take_ownership(x: String) { }

// By reference
fn borrow(x: &String) { }

// By mutable reference
fn borrow_mut(x: &mut String) { }

// Multiple parameters
fn multiple(a: i32, b: f64, c: bool) { }

// Default parameters (future feature)
fn with_default(x: i32, y: i32 = 0) { }
```

### 9.3 Generic Functions

```blang
fn identity<T>(x: T) -> T {
  return x;
}

fn swap<T>(a: &mut T, b: &mut T) {
  let temp = *a;
  *a = *b;
  *b = temp;
}

fn max<T: Ord>(a: T, b: T) -> T {
  if a > b { a } else { b }
}
```

### 9.4 Methods

```blang
impl Point {
  fn new(x: f64, y: f64) -> Point {
    Point { x, y }
  }

  fn distance(&self) -> f64 {
    (self.x * self.x + self.y * self.y).sqrt()
  }

  fn scale(&mut self, factor: f64) {
    self.x *= factor;
    self.y *= factor;
  }

  fn into_tuple(self) -> (f64, f64) {
    (self.x, self.y)
  }
}
```

### 9.5 Associated Functions

```blang
impl Point {
  fn origin() -> Point {
    Point { x: 0.0, y: 0.0 }
  }
}

let origin = Point::origin();
```

### 9.6 Higher-Order Functions

```blang
fn apply<T, U>(f: fn(T) -> U, x: T) -> U {
  f(x)
}

fn compose<A, B, C>(f: fn(B) -> C, g: fn(A) -> B) -> fn(A) -> C {
  |x| f(g(x))
}
```

### 9.7 Visibility

```blang
pub fn public_function() { }      // Public
fn private_function() { }         // Private (default)
pub(module) fn module_function() { } // Module-visible
```

---

## 10. Modules

### 10.1 Module Declaration

```blang
// In file: math.bl
module math {
  pub fn add(a: i32, b: i32) -> i32 {
    a + b
  }

  fn internal_helper() {
    // private to module
  }

  pub struct Point {
    pub x: f64,
    pub y: f64,
  }
}
```

### 10.2 Module Imports

```blang
// Import entire module
import math;
let result = math::add(1, 2);

// Import specific items
import math::{add, Point};
let result = add(1, 2);

// Import with alias
import math::add as sum;
let result = sum(1, 2);

// Import all public items
import math::*;

// Nested imports
import collections::{vector::Vec, map::Map};
```

### 10.3 Module Hierarchy

```
src/
  lib.bl (root module)
  utils/
    mod.bl (utils module)
    string.bl
    math.bl
  components/
    mod.bl
    button.blang
    input.blang
```

```blang
// In lib.bl
pub module utils;
pub module components;

// In other files
import utils::string::format;
import components::Button;
```

### 10.4 Re-exports

```blang
module prelude {
  pub use core::option::Option;
  pub use core::result::Result;
  pub use core::iter::Iterator;
}

// Users can then do:
import prelude::*;
```

---

## 11. Component Mode

Components are the building blocks for reactive UI applications.

### 11.1 Component Declaration

```blang
component ComponentName {
  // Component structure
}
```

### 11.2 Component State

```blang
component Counter {
  state {
    count: Signal<i32> = signal(0);
    label: Signal<str> = signal("Counter");
  }
}
```

### 11.3 Component Props

```blang
component Button {
  props {
    text: str,
    disabled: bool = false,
    onclick: fn() = || {},
  }
}
```

### 11.4 Component View

```blang
component TodoItem {
  props {
    todo: Todo,
    onToggle: fn(i32),
  }

  view {
    <div class="todo-item">
      <input
        type="checkbox"
        checked={todo.completed}
        @change={handleToggle}
      />
      <span class={if todo.completed { "completed" } else { "" }}>
        {todo.text}
      </span>
    </div>
  }

  fn handleToggle() {
    self.props.onToggle(todo.id);
  }
}
```

### 11.5 Component Styles

```blang
component Card {
  view {
    <div class="card">
      <h2>{props.title}</h2>
      <div class="content">
        {props.children}
      </div>
    </div>
  }

  style {
    .card {
      border: 1px solid #ddd;
      border-radius: 8px;
      padding: 1rem;
      box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    }

    .content {
      margin-top: 1rem;
    }

    /* Scoped to this component */
    h2 {
      margin: 0;
      font-size: 1.5rem;
    }
  }
}
```

### 11.6 Component Lifecycle

```blang
component LifecycleExample {
  state {
    data: Signal<Option<Data>> = signal(None);
  }

  on_mount {
    // Called when component is first mounted
    fetch_data();
  }

  on_update {
    // Called after state updates
    log("Component updated");
  }

  on_unmount {
    // Called before component is removed
    cleanup_resources();
  }

  fn fetch_data() {
    // async data fetching
  }
}
```

### 11.7 Computed Values

```blang
component TodoList {
  state {
    todos: Signal<Vec<Todo>> = signal(vec![]);
  }

  computed {
    completed_count: i32 = {
      todos.value().iter().filter(|t| t.completed).count() as i32
    };

    active_count: i32 = {
      todos.value().len() as i32 - completed_count
    };
  }

  view {
    <div>
      <p>Active: {active_count}</p>
      <p>Completed: {completed_count}</p>
    </div>
  }
}
```

### 11.8 Event Handling

```blang
component EventExample {
  state {
    value: Signal<str> = signal("");
  }

  view {
    <div>
      <input
        type="text"
        value={value}
        @input={handleInput}
        @keypress={handleKeyPress}
      />
      <button @click={handleClick}>Submit</button>
    </div>
  }

  fn handleInput(event: InputEvent) {
    value.set(event.target.value);
  }

  fn handleKeyPress(event: KeyboardEvent) {
    if event.key == "Enter" {
      handleSubmit();
    }
  }

  fn handleClick() {
    handleSubmit();
  }

  fn handleSubmit() {
    log(`Submitted: ${value.value()}`);
  }
}
```

### 11.9 Child Components

```blang
component ParentComponent {
  state {
    items: Signal<Vec<str>> = signal(vec!["A", "B", "C"]);
  }

  view {
    <div class="parent">
      <h1>Parent Component</h1>
      {for item in items.value() {
        <ChildComponent text={item} />
      }}
    </div>
  }
}

component ChildComponent {
  props {
    text: str,
  }

  view {
    <div class="child">{text}</div>
  }
}
```

### 11.10 Slots and Children

```blang
component Layout {
  props {
    children: ComponentChildren,
  }

  view {
    <div class="layout">
      <header>Header</header>
      <main>
        {children}
      </main>
      <footer>Footer</footer>
    </div>
  }
}

// Usage:
<Layout>
  <p>This content goes in the main area</p>
</Layout>
```

---

## 12. Script Mode

Script mode provides REXX-inspired orchestration and job management.

### 12.1 Script Declaration

```blang
script ScriptName {
  // Script structure
}
```

### 12.2 Jobs

Jobs are logical groups of related steps:

```blang
script DataProcessor {
  job extract {
    step fetch_data {
      let data = http.get("/api/data");
      return data;
    }

    step validate {
      let data = extract.fetch_data.result;
      if !is_valid(data) {
        fail("Invalid data");
      }
      return data;
    }
  }
}
```

### 12.3 Steps

Steps are individual units of work:

```blang
step step_name {
  // step body
  return result;
}

step step_with_config {
  retry: 3,
  timeout: 30s,
  on_failure: continue,

  let result = risky_operation();
  return result;
}
```

### 12.4 Dependencies

```blang
script Pipeline {
  job extract {
    step fetch {
      return fetch_data();
    }
  }

  job transform depends_on(extract) {
    step process {
      let data = extract.fetch.result;
      return process_data(data);
    }
  }

  job load depends_on(transform) {
    step save {
      let processed = transform.process.result;
      save_to_db(processed);
    }
  }
}
```

### 12.5 Parallel Execution

```blang
script ParallelExample {
  job parallel_fetch {
    parallel {
      step fetch_users {
        return http.get("/api/users");
      }

      step fetch_products {
        return http.get("/api/products");
      }

      step fetch_orders {
        return http.get("/api/orders");
      }
    }
  }

  job process depends_on(parallel_fetch) {
    step merge {
      let users = parallel_fetch.fetch_users.result;
      let products = parallel_fetch.fetch_products.result;
      let orders = parallel_fetch.fetch_orders.result;
      return merge_data(users, products, orders);
    }
  }
}
```

### 12.6 Error Handling

```blang
script RobustPipeline {
  job risky_job {
    step might_fail {
      retry: 3,
      retry_delay: 5s,
      on_failure: continue,

      let result = unreliable_operation();
      return result;
    }

    step handle_failure {
      if risky_job.might_fail.status == "failed" {
        log("Step failed, using fallback");
        return fallback_value();
      } else {
        return risky_job.might_fail.result;
      }
    }
  }
}
```

### 12.7 Scheduling

```blang
script ScheduledTasks {
  schedule {
    cron: "0 0 * * *",  // Daily at midnight
  }

  job daily_cleanup {
    step clean_temp_files {
      clean_directory("/tmp/app");
    }

    step archive_old_logs {
      archive_logs(days: 7);
    }
  }
}
```

### 12.8 Script Configuration

```blang
script ConfigurableScript {
  config {
    max_parallel: 4,
    timeout: 300s,
    retry_default: 2,
  }

  job example {
    // jobs and steps
  }
}
```

### 12.9 Script State

```blang
script StatefulScript {
  state {
    processed_count: i32 = 0,
    errors: Vec<str> = vec![],
  }

  job process_items {
    step process {
      for item in get_items() {
        match process_item(item) {
          Ok(_) => self.state.processed_count += 1,
          Err(e) => self.state.errors.push(e),
        }
      }
    }
  }

  job report {
    depends_on(process_items),

    step generate_report {
      log(`Processed: ${self.state.processed_count}`);
      log(`Errors: ${self.state.errors.len()}`);
    }
  }
}
```

---

## 13. Unsafe Blocks

Unsafe blocks allow low-level operations for performance-critical code.

### 13.1 Unsafe Block Syntax

```blang
unsafe {
  // unsafe operations
}
```

### 13.2 Unsafe Operations

The following operations require `unsafe` blocks:

1. Dereferencing raw pointers
2. Calling unsafe functions
3. Accessing or modifying mutable static variables
4. Implementing unsafe traits
5. Accessing fields of unions
6. Direct memory manipulation
7. Inline WASM instructions

### 13.3 Raw Pointers

```blang
fn pointer_example() {
  let x = 42;
  let ptr: *const i32 = &x as *const i32;

  unsafe {
    let value = *ptr;  // dereference raw pointer
    log(`Value: ${value}`);
  }
}
```

### 13.4 Unsafe Functions

```blang
unsafe fn dangerous_operation(ptr: *mut i32) {
  *ptr = 42;
}

fn caller() {
  let mut x = 0;
  unsafe {
    dangerous_operation(&mut x as *mut i32);
  }
}
```

### 13.5 Direct Memory Access

```blang
unsafe fn memcpy(dest: *mut u8, src: *const u8, count: usize) {
  for i in 0..count {
    *dest.offset(i as isize) = *src.offset(i as isize);
  }
}
```

### 13.6 Inline WASM

```blang
unsafe fn add_wasm(a: i32, b: i32) -> i32 {
  wasm! {
    (local.get $a)
    (local.get $b)
    (i32.add)
  }
}
```

### 13.7 SIMD Operations

```blang
unsafe fn vector_add(a: &[f32], b: &[f32], result: &mut [f32]) {
  for i in (0..a.len()).step_by(4) {
    wasm! {
      (v128.load (local.get $a) (i32.const $i))
      (v128.load (local.get $b) (i32.const $i))
      (f32x4.add)
      (v128.store (local.get $result) (i32.const $i))
    }
  }
}
```

### 13.8 Safety Invariants

Code in unsafe blocks must maintain these invariants:

1. **No dangling pointers**: All pointers must point to valid memory
2. **No data races**: Concurrent access must be properly synchronized
3. **Type safety**: Raw pointers must be cast correctly
4. **Memory alignment**: Access must respect alignment requirements
5. **Bounds checking**: Manual bounds checking required for raw pointer access

### 13.9 Unsafe Traits

```blang
unsafe trait UnsafeTrait {
  fn dangerous_method(&self);
}

unsafe impl UnsafeTrait for MyType {
  fn dangerous_method(&self) {
    // implementation
  }
}
```

---

## 14. Concurrency Model

Blang provides structured concurrency through actors and channels.

### 14.1 Actors

```blang
actor Counter {
  state {
    count: i32 = 0,
  }

  receiver increment() {
    self.state.count += 1;
  }

  receiver get_count() -> i32 {
    return self.state.count;
  }

  receiver add(value: i32) {
    self.state.count += value;
  }
}

// Usage:
let counter = spawn_actor(Counter::new());
counter.send(increment());
let count = counter.send(get_count()).await;
```

### 14.2 Channels

```blang
// Bounded channel
let (tx, rx) = channel::<i32>(capacity: 10);

// Unbounded channel
let (tx, rx) = unbounded_channel::<str>();

// Send messages
tx.send(42);
tx.send(100);

// Receive messages
let value = rx.recv();  // blocking
let value = rx.try_recv();  // non-blocking
```

### 14.3 Select

```blang
select {
  value = rx1.recv() => {
    log(`Received from rx1: ${value}`);
  },
  value = rx2.recv() => {
    log(`Received from rx2: ${value}`);
  },
  timeout(1000ms) => {
    log("Timeout!");
  },
}
```

### 14.4 Async/Await (Future Feature)

```blang
async fn fetch_data(url: str) -> Result<Data, Error> {
  let response = http.get(url).await?;
  let data = response.json().await?;
  return Ok(data);
}

async fn process() {
  let data = fetch_data("https://api.example.com").await;
  match data {
    Ok(d) => process_data(d),
    Err(e) => log_error(e),
  }
}
```

### 14.5 Spawn

```blang
fn background_task() {
  loop {
    do_work();
    sleep(1000ms);
  }
}

let handle = spawn(background_task);
// Later...
handle.join();
```

---

## 15. Reactivity System

Blang's reactivity system is built on signals, inspired by SolidJS and Leptos.

### 15.1 Signals

```blang
// Create a signal
let count = signal(0);

// Read signal value
let value = count.value();

// Set signal value
count.set(42);

// Update signal value
count.update(|c| c + 1);
```

### 15.2 Effects

```blang
let count = signal(0);

// Effect runs when dependencies change
effect(|| {
  log(`Count is now: ${count.value()}`);
});

count.set(1);  // Effect runs
count.set(2);  // Effect runs again
```

### 15.3 Memos (Computed Values)

```blang
let first_name = signal("John");
let last_name = signal("Doe");

let full_name = memo(|| {
  `${first_name.value()} ${last_name.value()}`
});

log(full_name.value());  // "John Doe"
first_name.set("Jane");
log(full_name.value());  // "Jane Doe"
```

### 15.4 Resources (Async Data)

```blang
let user_id = signal(1);

let user_data = resource(
  || user_id.value(),
  |id| async {
    fetch_user(id).await
  }
);

// In component:
match user_data.value() {
  Loading => <div>Loading...</div>,
  Ready(user) => <div>{user.name}</div>,
  Error(err) => <div>Error: {err}</div>,
}
```

### 15.5 Batching

```blang
batch(|| {
  signal1.set(1);
  signal2.set(2);
  signal3.set(3);
  // All effects run once after this block
});
```

### 15.6 Untrack

```blang
let a = signal(1);
let b = signal(2);

effect(|| {
  log(`a = ${a.value()}`);
  untrack(|| {
    log(`b = ${b.value()}`);  // doesn't create dependency
  });
});

b.set(3);  // Effect doesn't run
a.set(2);  // Effect runs
```

---

## 16. Memory Model

### 16.1 Stack Allocation

Local variables are stack-allocated by default:

```blang
fn example() {
  let x = 42;  // stack-allocated
  let arr = [1, 2, 3];  // stack-allocated array
}
```

### 16.2 Heap Allocation

Dynamic-size data structures are heap-allocated:

```blang
let vec = Vec::new();  // heap-allocated
let string = String::from("hello");  // heap-allocated
let boxed = Box::new(42);  // explicit heap allocation
```

### 16.3 Ownership

Every value has a single owner:

```blang
let s1 = String::from("hello");
let s2 = s1;  // ownership moved to s2
// s1 is no longer valid
```

### 16.4 Borrowing

Values can be borrowed without transferring ownership:

```blang
fn calculate_length(s: &String) -> usize {
  s.len()  // borrow s
}

let s = String::from("hello");
let len = calculate_length(&s);
// s is still valid
```

### 16.5 Lifetimes

```blang
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
  if x.len() > y.len() { x } else { y }
}

struct RefHolder<'a> {
  reference: &'a str,
}
```

### 16.6 Drop

Resources are automatically cleaned up when they go out of scope:

```blang
{
  let s = String::from("hello");
  // use s
}  // s is dropped here
```

Custom drop implementation:

```blang
impl Drop for MyType {
  fn drop(&mut self) {
    // cleanup code
  }
}
```

---

## 17. Compilation Model

### 17.1 Compilation Pipeline

```
Source Code (.blang, .bl, .bs)
    ↓
Lexer (tokens)
    ↓
Parser (AST)
    ↓
Name Resolution
    ↓
Type Checker (Typed AST)
    ↓
HIR (High-level IR)
    ↓
MIR (Mid-level IR)
    ↓
Optimizations
    ↓
LIR (Low-level IR)
    ↓
Code Generation
    ↓
WASM + JS Glue
```

### 17.2 Compilation Targets

1. **Module Mode** (.bl):
   - Compiles to pure WASM
   - No DOM or Web API access
   - Optimized for performance

2. **Component Mode** (.blang):
   - Compiles to WASM + JS glue
   - DOM manipulation via JS bindings
   - Reactive runtime in WASM

3. **Script Mode** (.bs):
   - Compiles to orchestration engine
   - Async task scheduling
   - Can call modules and components

### 17.3 Optimization Levels

```
--opt-level 0  (debug, no optimizations)
--opt-level 1  (basic optimizations)
--opt-level 2  (full optimizations)
--opt-level 3  (aggressive optimizations)
```

### 17.4 Output Formats

1. **WASM Module**: Core logic compiled to WebAssembly
2. **JS Glue**: Minimal JavaScript for DOM and Web APIs
3. **Type Definitions**: TypeScript .d.ts files for interop
4. **Source Maps**: For debugging

### 17.5 Incremental Compilation

- Module-level caching
- Dependency tracking
- Fast rebuild for changed files

### 17.6 Tree Shaking

- Dead code elimination
- Unused import removal
- Minimal output size

---

## 18. Standard Library

### 18.1 Core Module

```blang
module core {
  // Primitives
  pub type i8, i16, i32, i64;
  pub type u8, u16, u32, u64;
  pub type f32, f64;
  pub type bool, char, str;

  // Option and Result
  pub enum Option<T> { Some(T), None }
  pub enum Result<T, E> { Ok(T), Err(E) }

  // Iterator trait
  pub trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
  }
}
```

### 18.2 Collections Module

```blang
module collections {
  pub struct Vec<T> { ... }
  pub struct Map<K, V> { ... }
  pub struct Set<T> { ... }
  pub struct LinkedList<T> { ... }
}
```

### 18.3 String Module

```blang
module string {
  pub struct String { ... }
  pub fn format(template: str, args: &[Any]) -> String;
  pub fn parse<T>(s: str) -> Result<T, ParseError>;
}
```

### 18.4 IO Module

```blang
module io {
  pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error>;
  }

  pub trait Write {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error>;
  }

  pub fn println(s: str);
  pub fn eprintln(s: str);
}
```

### 18.5 Math Module

```blang
module math {
  pub const PI: f64 = 3.14159265358979323846;
  pub const E: f64 = 2.71828182845904523536;

  pub fn abs<T: Num>(x: T) -> T;
  pub fn sqrt(x: f64) -> f64;
  pub fn sin(x: f64) -> f64;
  pub fn cos(x: f64) -> f64;
  pub fn pow(base: f64, exp: f64) -> f64;
}
```

### 18.6 Time Module

```blang
module time {
  pub struct Duration { ... }
  pub struct Instant { ... }

  pub fn now() -> Instant;
  pub fn sleep(duration: Duration);
}
```

### 18.7 HTTP Module

```blang
module http {
  pub struct Request { ... }
  pub struct Response { ... }

  pub fn get(url: str) -> Result<Response, Error>;
  pub fn post(url: str, body: &[u8]) -> Result<Response, Error>;
}
```

### 18.8 DOM Module (Component Mode Only)

```blang
module dom {
  pub struct Element { ... }
  pub struct Event { ... }

  pub fn query_selector(selector: str) -> Option<Element>;
  pub fn create_element(tag: str) -> Element;
}
```

---

## 19. Interoperability

### 19.1 JavaScript Interop

```blang
// Import JS function
extern "js" {
  fn console_log(msg: str);
  fn parse_json(json: str) -> JsValue;
}

// Export to JS
#[export]
pub fn exported_function(x: i32) -> i32 {
  return x * 2;
}
```

### 19.2 Web API Access

```blang
// In component mode
fn use_web_api() {
  let location = window.location();
  let url = location.href();

  localStorage.setItem("key", "value");
  let value = localStorage.getItem("key");
}
```

### 19.3 NPM Package Integration

```blang
// blang.toml
[dependencies.npm]
lodash = "4.17.21"

// In code
import lodash from "npm:lodash";

fn use_lodash() {
  let result = lodash.chunk([1, 2, 3, 4], 2);
}
```

---

## 20. Appendix

### 20.1 Reserved for Future Use

- `async`, `await` - Async/await syntax
- `yield` - Generator syntax
- `macro` - Macro system
- `reflect` - Reflection capabilities

### 20.2 Attributes

```blang
#[inline]
fn fast_function() { }

#[derive(Debug, Clone)]
struct MyStruct { }

#[test]
fn test_example() { }

#[deprecated]
fn old_function() { }

#[export]
pub fn exported() { }
```

### 20.3 Compiler Directives

```blang
#![no_std]  // No standard library
#![allow(unused)]  // Allow unused code
#![warn(missing_docs)]  // Warn on missing docs
```

### 20.4 File Extensions

- `.blang` - Component mode
- `.bl` - Module mode
- `.bs` - Script mode
- `.toml` - Configuration files

### 20.5 Versioning

Blang follows semantic versioning (SemVer):
- MAJOR: Breaking changes
- MINOR: New features, backwards-compatible
- PATCH: Bug fixes, backwards-compatible

---

## Conclusion

This specification defines Blang v0.1, a browser-first language designed for building modern web applications. The language combines UI development, application logic, orchestration, and performance-critical code in a unified, type-safe environment.

The specification is intentionally detailed to serve as the single source of truth for compiler implementation. As the language evolves, this document will be updated to reflect new features and refinements.

For implementation details, see the Grammar specification (GRAMMAR.md) and Development Roadmap (ROADMAP.md).
