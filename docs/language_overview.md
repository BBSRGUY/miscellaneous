# Blang Language Overview

A comprehensive guide to the Blang programming language syntax, semantics, and features.

## Table of Contents

- [Introduction](#introduction)
- [Basic Syntax](#basic-syntax)
- [Variables and Types](#variables-and-types)
- [Functions](#functions)
- [Control Flow](#control-flow)
- [Data Structures](#data-structures)
- [Pattern Matching](#pattern-matching)
- [Error Handling](#error-handling)
- [Modules and Imports](#modules-and-imports)
- [Generics and Traits](#generics-and-traits)
- [Memory Management](#memory-management)
- [Concurrency](#concurrency)
- [Standard Library](#standard-library)

## Introduction

Blang is a statically-typed, compiled programming language designed for building high-performance web applications. It compiles to WebAssembly and provides three distinct programming modes:

- **Module Mode**: General-purpose programming
- **Component Mode**: Reactive UI components
- **Script Mode**: Job orchestration and data pipelines

This guide focuses on the core language features available in all modes.

## Basic Syntax

### Comments

```blang
// Single-line comment

/*
  Multi-line comment
  Can span multiple lines
*/

/* Nested /* comments */ are supported */
```

### Semicolons

Statements require semicolons:

```blang
let x = 42;
let y = 10;
return x + y;
```

Expressions in blocks don't need semicolons:

```blang
fn add(a: i32, b: i32) -> i32 {
    a + b  // No semicolon - this is a return expression
}
```

## Variables and Types

### Variable Declaration

```blang
// Immutable by default
let x = 42;
let name = "Alice";
let is_active = true;

// Mutable variables with 'mut'
let mut counter = 0;
counter = counter + 1;
counter += 1;  // Compound assignment
```

### Type Annotations

```blang
// Explicit type annotations
let age: i32 = 25;
let price: f64 = 19.99;
let message: str = "Hello";

// Type inference works in most cases
let x = 42;           // Inferred as i32
let y = 3.14;         // Inferred as f64
let name = "Bob";     // Inferred as str
```

### Primitive Types

```blang
// Integers
let i8_val: i8 = -128;
let i16_val: i16 = -32768;
let i32_val: i32 = -2147483648;
let i64_val: i64 = -9223372036854775808;

let u8_val: u8 = 255;
let u16_val: u16 = 65535;
let u32_val: u32 = 4294967295;
let u64_val: u64 = 18446744073709551615;

// Floating-point
let f32_val: f32 = 3.14;
let f64_val: f64 = 3.141592653589793;

// Boolean
let is_true: bool = true;
let is_false: bool = false;

// String
let text: str = "Hello, World!";

// Character
let ch: char = 'a';

// Unit type (similar to void)
let unit: () = ();
```

### Compound Types

```blang
// Arrays (fixed size)
let numbers: [i32; 5] = [1, 2, 3, 4, 5];
let zeros: [i32; 100] = [0; 100];  // Initialize all to 0

// Tuples
let pair: (i32, str) = (42, "answer");
let triple = (1, 2.0, "three");  // Type inferred

// Accessing tuple elements
let first = pair.0;
let second = pair.1;
```

## Functions

### Function Declaration

```blang
// Basic function
fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Function with no return value
fn greet(name: str) {
    println("Hello, " + name);
}

// Multiple return values via tuple
fn divmod(a: i32, b: i32) -> (i32, i32) {
    (a / b, a % b)
}

// Early return
fn check_positive(x: i32) -> bool {
    if x < 0 {
        return false;
    }
    true
}
```

### Function Overloading

Functions can be overloaded based on parameter types:

```blang
fn print(x: i32) {
    println(x.to_string());
}

fn print(x: str) {
    println(x);
}

fn print(x: bool) {
    println(if x { "true" } else { "false" });
}
```

### Generic Functions

```blang
// Generic function with type parameter
fn identity<T>(value: T) -> T {
    value
}

// Multiple type parameters
fn pair<T, U>(first: T, second: U) -> (T, U) {
    (first, second)
}

// Type constraints
fn max<T: Ord>(a: T, b: T) -> T {
    if a > b { a } else { b }
}
```

### Higher-Order Functions

```blang
// Function as parameter
fn apply<T>(f: fn(T) -> T, value: T) -> T {
    f(value)
}

// Returning functions
fn make_adder(x: i32) -> fn(i32) -> i32 {
    fn adder(y: i32) -> i32 {
        x + y
    }
    adder
}
```

## Control Flow

### If Expressions

```blang
// If as statement
if x > 0 {
    println("positive");
}

// If-else
if x > 0 {
    println("positive");
} else {
    println("not positive");
}

// If-else if-else
if x > 0 {
    println("positive");
} else if x < 0 {
    println("negative");
} else {
    println("zero");
}

// If as expression
let result = if x > 0 {
    "positive"
} else {
    "negative"
};
```

### Loops

```blang
// While loop
let mut i = 0;
while i < 10 {
    println(i);
    i += 1;
}

// Infinite loop
loop {
    println("forever");
    break;  // Exit the loop
}

// For loop over range
for i in 0..10 {
    println(i);
}

// For loop over array
let numbers = [1, 2, 3, 4, 5];
for num in numbers {
    println(num);
}

// Break and continue
for i in 0..100 {
    if i % 2 == 0 {
        continue;  // Skip even numbers
    }
    if i > 50 {
        break;  // Stop at 50
    }
    println(i);
}
```

### Match Expressions

```blang
// Match on values
let number = 42;
let description = match number {
    0 => "zero",
    1 => "one",
    2 => "two",
    _ => "many",  // Default case
};

// Match on types
fn process(value: Value) {
    match value {
        Value::Int(n) => println("Integer: " + n.to_string()),
        Value::Str(s) => println("String: " + s),
        Value::Bool(b) => println("Boolean: " + b.to_string()),
    }
}

// Match with guards
match x {
    n if n < 0 => "negative",
    0 => "zero",
    n if n % 2 == 0 => "even",
    _ => "odd",
}
```

## Data Structures

### Structs

```blang
// Define a struct
struct Point {
    x: i32,
    y: i32,
}

// Create instance
let p = Point { x: 10, y: 20 };

// Access fields
let x_coord = p.x;
let y_coord = p.y;

// Struct with methods
impl Point {
    fn new(x: i32, y: i32) -> Point {
        Point { x, y }
    }

    fn distance(&self) -> f64 {
        sqrt((self.x * self.x + self.y * self.y) as f64)
    }

    fn translate(&mut self, dx: i32, dy: i32) {
        self.x += dx;
        self.y += dy;
    }
}

// Usage
let mut point = Point::new(0, 0);
point.translate(10, 20);
let dist = point.distance();
```

### Enums

```blang
// Simple enum
enum Direction {
    North,
    South,
    East,
    West,
}

// Enum with data
enum Option<T> {
    Some(T),
    None,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}

// Enum with methods
impl Direction {
    fn opposite(&self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
        }
    }
}
```

### Tuples and Tuple Structs

```blang
// Regular tuple
let point = (10, 20);

// Tuple struct (named tuple)
struct Point(i32, i32);
struct Color(u8, u8, u8);

let p = Point(10, 20);
let red = Color(255, 0, 0);

// Access via index
let x = p.0;
let y = p.1;
```

## Pattern Matching

### Destructuring

```blang
// Tuple destructuring
let (x, y) = (10, 20);
let (a, b, c) = (1, 2, 3);

// Struct destructuring
struct Point { x: i32, y: i32 }
let Point { x, y } = point;
let Point { x: x_coord, y: y_coord } = point;  // Rename

// Array destructuring
let [first, second, rest..] = [1, 2, 3, 4, 5];

// Enum destructuring
match option {
    Option::Some(value) => println(value),
    Option::None => println("no value"),
}
```

### Pattern Matching in Function Parameters

```blang
fn print_point(Point { x, y }: Point) {
    println("Point at (" + x.to_string() + ", " + y.to_string() + ")");
}

fn get_first((first, _): (i32, i32)) -> i32 {
    first
}
```

## Error Handling

### Result Type

```blang
enum Result<T, E> {
    Ok(T),
    Err(E),
}

// Function that can fail
fn divide(a: i32, b: i32) -> Result<i32, str> {
    if b == 0 {
        Result::Err("division by zero")
    } else {
        Result::Ok(a / b)
    }
}

// Handle errors
let result = divide(10, 2);
match result {
    Result::Ok(value) => println("Result: " + value.to_string()),
    Result::Err(msg) => println("Error: " + msg),
}
```

### Option Type

```blang
enum Option<T> {
    Some(T),
    None,
}

// Function that might not return a value
fn find_first(arr: [i32], target: i32) -> Option<i32> {
    for (i, value) in arr.enumerate() {
        if value == target {
            return Option::Some(i);
        }
    }
    Option::None
}

// Handle option
match find_first(numbers, 42) {
    Option::Some(index) => println("Found at index " + index.to_string()),
    Option::None => println("Not found"),
}
```

### The ? Operator

```blang
fn process_file(path: str) -> Result<str, Error> {
    let content = read_file(path)?;  // Propagate error if read fails
    let parsed = parse_content(content)?;  // Propagate error if parse fails
    Ok(transform(parsed))
}
```

## Modules and Imports

### Module Declaration

```blang
// In math.blang
module math {
    pub fn add(a: i32, b: i32) -> i32 {
        a + b
    }

    fn internal_helper() {
        // Private function
    }

    pub const PI: f64 = 3.14159;
}
```

### Import Statements

```blang
// Import specific items
use math::add;
use math::{add, subtract};

// Import all public items
use math::*;

// Import with alias
use math::add as math_add;

// Import from nested modules
use utils::string::trim;
use utils::string::{trim, split};
```

### Module Hierarchy

```blang
// project/
// ├── lib.blang
// ├── math/
// │   ├── mod.blang
// │   ├── algebra.blang
// │   └── geometry.blang

// In lib.blang
module math;

// In math/mod.blang
pub module algebra;
pub module geometry;

// In math/algebra.blang
pub fn solve_linear(a: f64, b: f64) -> f64 {
    -b / a
}
```

## Generics and Traits

### Generic Types

```blang
// Generic struct
struct Box<T> {
    value: T,
}

impl<T> Box<T> {
    fn new(value: T) -> Box<T> {
        Box { value }
    }

    fn get(&self) -> &T {
        &self.value
    }
}

// Generic enum
enum Maybe<T> {
    Just(T),
    Nothing,
}

// Multiple type parameters
struct Pair<T, U> {
    first: T,
    second: U,
}
```

### Traits

```blang
// Define a trait
trait Printable {
    fn print(&self);
}

// Implement trait for type
impl Printable for i32 {
    fn print(&self) {
        println(self.to_string());
    }
}

impl Printable for str {
    fn print(&self) {
        println(self);
    }
}

// Trait bounds
fn print_twice<T: Printable>(value: T) {
    value.print();
    value.print();
}

// Multiple trait bounds
fn process<T: Printable + Clone>(value: T) {
    let copy = value.clone();
    copy.print();
}
```

### Common Traits

```blang
// Clone trait
trait Clone {
    fn clone(&self) -> Self;
}

// Copy trait (for types that can be copied bitwise)
trait Copy: Clone {}

// PartialEq and Eq for equality
trait PartialEq {
    fn eq(&self, other: &Self) -> bool;
}

trait Eq: PartialEq {}

// Ord for ordering
trait Ord {
    fn cmp(&self, other: &Self) -> Ordering;
}

enum Ordering {
    Less,
    Equal,
    Greater,
}
```

## Memory Management

### Ownership

```blang
// Ownership basics
let s1 = String::from("hello");
let s2 = s1;  // s1 is moved to s2, s1 is no longer valid

// Borrowing (references)
fn calculate_length(s: &str) -> i32 {
    s.len()
}

let s = String::from("hello");
let len = calculate_length(&s);  // Borrow s
// s is still valid here
```

### Mutable References

```blang
fn append_world(s: &mut String) {
    s.push_str(" world");
}

let mut s = String::from("hello");
append_world(&mut s);
println(s);  // "hello world"
```

### Lifetimes

```blang
// Explicit lifetime annotations
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}

// Struct with lifetime
struct Excerpt<'a> {
    text: &'a str,
}

impl<'a> Excerpt<'a> {
    fn get_text(&self) -> &str {
        self.text
    }
}
```

## Concurrency

### Actors

```blang
// Define an actor
actor Counter {
    state count: i32 = 0;

    fn increment() {
        count += 1;
    }

    fn decrement() {
        count -= 1;
    }

    fn get_count() -> i32 {
        count
    }
}

// Use actor
let counter = Counter::spawn();
counter.send(Counter::increment);
let value = counter.send(Counter::get_count).await;
```

### Channels

```blang
// Create a channel
let (sender, receiver) = channel<i32>();

// Send values
sender.send(42);
sender.send(100);

// Receive values
let value1 = receiver.receive();
let value2 = receiver.receive();
```

### Async/Await

```blang
async fn fetch_data(url: str) -> Result<String, Error> {
    let response = http::get(url).await?;
    Ok(response.text().await?)
}

async fn process() {
    let data = fetch_data("https://api.example.com/data").await;
    match data {
        Ok(text) => println(text),
        Err(e) => println("Error: " + e.to_string()),
    }
}
```

## Standard Library

### Common Modules

```blang
// String operations
use std::string::{String, format, trim, split};

let s = String::from("hello");
let formatted = format("Number: {}", 42);
let trimmed = trim("  spaces  ");

// Collections
use std::collections::{Vec, HashMap, HashSet};

let mut vec = Vec::new();
vec.push(1);
vec.push(2);

let mut map = HashMap::new();
map.insert("key", "value");

// Math
use std::math::{abs, sqrt, pow, sin, cos};

let x = abs(-42);
let y = sqrt(16.0);

// IO
use std::io::{println, read_line, read_file, write_file};

println("Hello, World!");
let input = read_line();
let content = read_file("data.txt")?;

// Time
use std::time::{now, Duration, sleep};

let start = now();
sleep(Duration::from_secs(1));
let elapsed = now() - start;
```

### Iterators

```blang
// Create iterators
let numbers = [1, 2, 3, 4, 5];
let iter = numbers.iter();

// Iterator methods
let doubled = numbers.iter().map(|x| x * 2);
let evens = numbers.iter().filter(|x| x % 2 == 0);
let sum = numbers.iter().sum();
let product = numbers.iter().product();

// Chaining
let result = numbers.iter()
    .filter(|x| x % 2 == 0)
    .map(|x| x * x)
    .sum();

// Collecting
let vec: Vec<i32> = (0..10).collect();
```

## Best Practices

### Naming Conventions

- **Types**: PascalCase (`Point`, `HttpRequest`)
- **Functions**: snake_case (`calculate_total`, `send_message`)
- **Constants**: SCREAMING_SNAKE_CASE (`MAX_SIZE`, `DEFAULT_TIMEOUT`)
- **Variables**: snake_case (`user_name`, `total_count`)

### Code Organization

```blang
// Group related functionality
module geometry {
    pub struct Point { x: f64, y: f64 }
    pub struct Line { start: Point, end: Point }

    impl Point {
        pub fn distance(&self, other: &Point) -> f64 {
            sqrt((self.x - other.x).pow(2) + (self.y - other.y).pow(2))
        }
    }
}

// Keep functions small and focused
fn process_order(order: Order) -> Result<(), Error> {
    validate_order(&order)?;
    calculate_total(&order)?;
    submit_payment(&order)?;
    send_confirmation(&order)?;
    Ok(())
}
```

### Error Handling

- Use `Result` for operations that can fail
- Use `Option` for values that might not exist
- Provide descriptive error messages
- Use the `?` operator for error propagation

### Performance

- Use references to avoid unnecessary copies
- Prefer iterators over explicit loops
- Use `const` for compile-time constants
- Profile before optimizing

---

This completes the language overview. For mode-specific features, see:

- [Component Mode](component_mode.md) - Reactive UI components
- [Script Mode](script_mode.md) - Job orchestration
- [Unsafe Blocks](unsafe_blocks.md) - Low-level programming
