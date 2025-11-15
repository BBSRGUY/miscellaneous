# Unsafe Blocks - Low-Level Programming in Blang

A comprehensive guide to writing performance-critical and low-level code using unsafe blocks in Blang.

## Table of Contents

- [Introduction](#introduction)
- [When to Use Unsafe](#when-to-use-unsafe)
- [Unsafe Block Syntax](#unsafe-block-syntax)
- [Raw Pointers](#raw-pointers)
- [Memory Operations](#memory-operations)
- [WASM Intrinsics](#wasm-intrinsics)
- [FFI - Foreign Function Interface](#ffi---foreign-function-interface)
- [Inline Assembly](#inline-assembly)
- [SIMD Operations](#simd-operations)
- [Performance Optimization](#performance-optimization)
- [Safety Guidelines](#safety-guidelines)
- [Common Patterns](#common-patterns)
- [Debugging Unsafe Code](#debugging-unsafe-code)

## Introduction

Unsafe blocks in Blang allow you to bypass safety guarantees for performance-critical code or low-level operations. While Blang provides memory safety by default through its ownership system, unsafe blocks give you direct control when needed.

### Key Concepts

- **Unsafe Functions**: Functions that contain unsafe operations
- **Raw Pointers**: Direct memory addresses without safety guarantees
- **Memory Operations**: Direct memory reads/writes
- **WASM Intrinsics**: Direct WebAssembly instructions
- **FFI**: Calling external (JavaScript) functions
- **Responsibility**: You guarantee safety, not the compiler

## When to Use Unsafe

Use unsafe blocks for:

1. **Performance-critical code**: SIMD, direct memory access
2. **Low-level operations**: Custom allocators, data structures
3. **FFI**: Interacting with JavaScript or external APIs
4. **WASM intrinsics**: Accessing platform-specific features
5. **Optimization**: Eliminating bounds checks in hot loops

**DO NOT** use unsafe for:

- General application logic
- Code that can be written safely
- Convenience (avoiding the borrow checker)
- Unclear performance benefits

## Unsafe Block Syntax

### Unsafe Functions

```blang
// Declare an unsafe function
unsafe fn read_pointer<T>(ptr: *const T) -> T {
    *ptr
}

// Call unsafe function
fn use_unsafe() {
    let x = 42;
    let ptr = &x as *const i32;

    unsafe {
        let value = read_pointer(ptr);
        println(value.to_string());
    }
}
```

### Unsafe Blocks

```blang
// Unsafe block for isolated operations
fn process_buffer(data: &[u8]) {
    unsafe {
        let ptr = data.as_ptr();
        let first = *ptr;
        println("First byte: " + first.to_string());
    }
}
```

### Standalone Unsafe Blocks

```blang
// Top-level unsafe block for platform-specific code
unsafe block fast_memcpy {
    pub fn memcpy(dest: *mut u8, src: *const u8, count: usize) {
        wasm! {
            (memory.copy
                (local.get $dest)
                (local.get $src)
                (local.get $count))
        }
    }
}
```

## Raw Pointers

### Pointer Types

```blang
// Const pointer (read-only)
let ptr: *const i32 = &x;

// Mutable pointer (read-write)
let mut_ptr: *mut i32 = &mut x;

// Null pointer
let null_ptr: *const i32 = 0 as *const i32;
```

### Creating Pointers

```blang
fn pointer_basics() {
    let x = 42;

    // Get pointer from reference
    let ptr = &x as *const i32;

    // Get mutable pointer
    let mut y = 10;
    let mut_ptr = &mut y as *mut i32;

    // Pointer from raw address
    let addr_ptr = 0x1000 as *const u8;
}
```

### Dereferencing Pointers

```blang
unsafe fn dereference_example() {
    let x = 42;
    let ptr = &x as *const i32;

    // Read through pointer
    let value = *ptr;
    println(value.to_string());  // 42

    // Write through mutable pointer
    let mut y = 10;
    let mut_ptr = &mut y as *mut i32;
    *mut_ptr = 20;
    println(y.to_string());  // 20
}
```

### Pointer Arithmetic

```blang
unsafe fn pointer_arithmetic() {
    let arr = [1, 2, 3, 4, 5];
    let ptr = arr.as_ptr();

    // Offset pointer
    let second = ptr.offset(1);
    let value = *second;  // 2

    // Add to pointer
    let third = ptr.add(2);
    let value = *third;  // 3

    // Subtract from pointer
    let first = third.sub(2);
    let value = *first;  // 1
}
```

## Memory Operations

### Direct Memory Access

```blang
unsafe fn read_write_memory() {
    // Allocate memory
    let size = 1024;
    let ptr = malloc(size) as *mut u8;

    // Write to memory
    for i in 0..size {
        *ptr.add(i) = i as u8;
    }

    // Read from memory
    let value = *ptr.add(512);

    // Free memory
    free(ptr as *mut ());
}
```

### Memory Copy

```blang
unsafe fn copy_memory<T>(dest: *mut T, src: *const T, count: usize) {
    for i in 0..count {
        *dest.add(i) = *src.add(i);
    }
}

// Usage
unsafe fn use_copy() {
    let src = [1, 2, 3, 4, 5];
    let mut dest = [0, 0, 0, 0, 0];

    copy_memory(
        dest.as_mut_ptr(),
        src.as_ptr(),
        5
    );
}
```

### Memory Set

```blang
unsafe fn memset(ptr: *mut u8, value: u8, count: usize) {
    for i in 0..count {
        *ptr.add(i) = value;
    }
}

// Usage
unsafe fn zero_buffer() {
    let mut buffer = [0u8; 1024];
    memset(buffer.as_mut_ptr(), 0, 1024);
}
```

## WASM Intrinsics

### Direct WASM Instructions

```blang
unsafe block wasm_intrinsics {
    pub fn fast_sqrt(x: f32) -> f32 {
        wasm! {
            (f32.sqrt (local.get $x))
        }
    }

    pub fn fast_min(a: i32, b: i32) -> i32 {
        wasm! {
            (select
                (local.get $a)
                (local.get $b)
                (i32.lt_s (local.get $a) (local.get $b)))
        }
    }

    pub fn population_count(x: i32) -> i32 {
        wasm! {
            (i32.popcnt (local.get $x))
        }
    }
}
```

### Memory Instructions

```blang
unsafe block wasm_memory {
    pub fn load_i32(addr: i32) -> i32 {
        wasm! {
            (i32.load (local.get $addr))
        }
    }

    pub fn store_i32(addr: i32, value: i32) {
        wasm! {
            (i32.store (local.get $addr) (local.get $value))
        }
    }

    pub fn memory_size() -> i32 {
        wasm! {
            (memory.size)
        }
    }

    pub fn memory_grow(pages: i32) -> i32 {
        wasm! {
            (memory.grow (local.get $pages))
        }
    }
}
```

## FFI - Foreign Function Interface

### Calling JavaScript

```blang
// Declare external JavaScript function
extern "js" {
    fn console_log(message: str);
    fn fetch(url: str) -> Promise<str>;
    fn setTimeout(callback: fn(), delay: i32) -> i32;
}

// Use in unsafe block
unsafe fn call_js() {
    console_log("Hello from Blang!");

    let handle = setTimeout(|| {
        console_log("Timer fired!");
    }, 1000);
}
```

### Exposing Functions to JavaScript

```blang
// Export function to JavaScript
#[export]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Export with custom name
#[export(name = "multiply")]
pub fn mul(a: i32, b: i32) -> i32 {
    a * b
}

// Export unsafe function
#[export]
pub unsafe fn process_buffer(ptr: *const u8, len: usize) -> i32 {
    let mut sum = 0;
    for i in 0..len {
        sum += *ptr.add(i) as i32;
    }
    sum
}
```

### Passing Complex Types

```blang
// Define ABI-compatible struct
#[repr(C)]
struct Point {
    x: f32,
    y: f32,
}

#[export]
pub fn create_point(x: f32, y: f32) -> Point {
    Point { x, y }
}

#[export]
pub fn distance(p1: Point, p2: Point) -> f32 {
    let dx = p1.x - p2.x;
    let dy = p1.y - p2.y;
    unsafe {
        fast_sqrt(dx * dx + dy * dy)
    }
}

unsafe fn fast_sqrt(x: f32) -> f32 {
    wasm! {
        (f32.sqrt (local.get $x))
    }
}
```

## Inline Assembly

### WASM Assembly Blocks

```blang
unsafe fn inline_asm_example(x: i32, y: i32) -> i32 {
    wasm! {
        // Load parameters
        (local.get $x)
        (local.get $y)

        // Multiply
        (i32.mul)

        // Add constant
        (i32.const 42)
        (i32.add)

        // Return result
    }
}
```

### Complex Assembly

```blang
unsafe fn bitwise_operations(a: i32, b: i32) -> i32 {
    wasm! {
        // Compute (a & b) | (a ^ b)
        (local.get $a)
        (local.get $b)
        (i32.and)

        (local.get $a)
        (local.get $b)
        (i32.xor)

        (i32.or)
    }
}
```

## SIMD Operations

### Vector Types

```blang
unsafe block simd {
    // WASM SIMD types
    type v128 = __wasm_v128;

    pub fn add_vectors(a: v128, b: v128) -> v128 {
        wasm! {
            (i32x4.add (local.get $a) (local.get $b))
        }
    }

    pub fn mul_vectors(a: v128, b: v128) -> v128 {
        wasm! {
            (i32x4.mul (local.get $a) (local.get $b))
        }
    }
}
```

### SIMD Array Operations

```blang
unsafe fn vector_add(a: &[f32], b: &[f32], result: &mut [f32]) {
    assert!(a.len() == b.len() && a.len() == result.len());
    assert!(a.len() % 4 == 0);  // Must be multiple of 4

    let count = a.len() / 4;

    for i in 0..count {
        let offset = i * 4;

        // Load 4 floats from each array
        let va = wasm! {
            (v128.load (i32.add
                (local.get $a)
                (i32.mul (local.get $offset) (i32.const 4))))
        };

        let vb = wasm! {
            (v128.load (i32.add
                (local.get $b)
                (i32.mul (local.get $offset) (i32.const 4))))
        };

        // Add vectors
        let vresult = wasm! {
            (f32x4.add (local.get $va) (local.get $vb))
        };

        // Store result
        wasm! {
            (v128.store
                (i32.add
                    (local.get $result)
                    (i32.mul (local.get $offset) (i32.const 4)))
                (local.get $vresult))
        };
    }
}
```

## Performance Optimization

### Eliminating Bounds Checks

```blang
// Safe version with bounds checks
fn safe_sum(arr: &[i32]) -> i32 {
    let mut sum = 0;
    for i in 0..arr.len() {
        sum += arr[i];  // Bounds check on every access
    }
    sum
}

// Unsafe version without bounds checks
unsafe fn fast_sum(arr: &[i32]) -> i32 {
    let mut sum = 0;
    let ptr = arr.as_ptr();
    let len = arr.len();

    for i in 0..len {
        sum += *ptr.add(i);  // No bounds check
    }
    sum
}
```

### Loop Unrolling

```blang
unsafe fn unrolled_sum(arr: &[i32]) -> i32 {
    let mut sum = 0;
    let ptr = arr.as_ptr();
    let len = arr.len();
    let chunks = len / 4;
    let remainder = len % 4;

    // Process 4 elements at a time
    for i in 0..chunks {
        let offset = i * 4;
        sum += *ptr.add(offset);
        sum += *ptr.add(offset + 1);
        sum += *ptr.add(offset + 2);
        sum += *ptr.add(offset + 3);
    }

    // Process remaining elements
    let offset = chunks * 4;
    for i in 0..remainder {
        sum += *ptr.add(offset + i);
    }

    sum
}
```

### Cache-Friendly Access

```blang
unsafe fn transpose_matrix(src: &[f32], dest: &mut [f32], rows: usize, cols: usize) {
    let src_ptr = src.as_ptr();
    let dest_ptr = dest.as_mut_ptr();

    // Cache-friendly blocked transpose
    let block_size = 16;

    for i in (0..rows).step_by(block_size) {
        for j in (0..cols).step_by(block_size) {
            let i_end = min(i + block_size, rows);
            let j_end = min(j + block_size, cols);

            for ii in i..i_end {
                for jj in j..j_end {
                    let src_idx = ii * cols + jj;
                    let dest_idx = jj * rows + ii;
                    *dest_ptr.add(dest_idx) = *src_ptr.add(src_idx);
                }
            }
        }
    }
}
```

## Safety Guidelines

### 1. Document Unsafe Code

```blang
/// Copies `count` bytes from `src` to `dest`.
///
/// # Safety
///
/// This function is unsafe because:
/// - `src` must be valid for reads of `count` bytes
/// - `dest` must be valid for writes of `count` bytes
/// - `src` and `dest` must not overlap
/// - Both pointers must be properly aligned
unsafe fn copy_nonoverlapping(dest: *mut u8, src: *const u8, count: usize) {
    for i in 0..count {
        *dest.add(i) = *src.add(i);
    }
}
```

### 2. Minimize Unsafe Surface

```blang
// Good: Small unsafe core with safe wrapper
pub fn process_array(arr: &[i32]) -> i32 {
    // Validate inputs in safe code
    if arr.is_empty() {
        return 0;
    }

    // Minimal unsafe block
    unsafe {
        fast_sum_unchecked(arr)
    }
}

unsafe fn fast_sum_unchecked(arr: &[i32]) -> i32 {
    let mut sum = 0;
    let ptr = arr.as_ptr();

    for i in 0..arr.len() {
        sum += *ptr.add(i);
    }

    sum
}
```

### 3. Validate Inputs

```blang
pub fn safe_read<T>(ptr: *const T, offset: usize) -> Option<T> {
    // Check for null
    if ptr.is_null() {
        return None;
    }

    // Check alignment
    if ptr as usize % align_of::<T>() != 0 {
        return None;
    }

    unsafe {
        Some(*ptr.add(offset))
    }
}
```

### 4. Use Debug Assertions

```blang
unsafe fn indexed_access<T>(arr: &[T], index: usize) -> &T {
    debug_assert!(index < arr.len(), "Index out of bounds");

    let ptr = arr.as_ptr();
    &*ptr.add(index)
}
```

### 5. Prefer Safe Abstractions

```blang
// Good: Safe wrapper around unsafe code
pub struct Buffer {
    ptr: *mut u8,
    len: usize,
    capacity: usize,
}

impl Buffer {
    pub fn new(capacity: usize) -> Buffer {
        unsafe {
            let ptr = malloc(capacity) as *mut u8;
            Buffer { ptr, len: 0, capacity }
        }
    }

    pub fn push(&mut self, value: u8) {
        assert!(self.len < self.capacity, "Buffer full");

        unsafe {
            *self.ptr.add(self.len) = value;
        }

        self.len += 1;
    }

    pub fn get(&self, index: usize) -> Option<u8> {
        if index >= self.len {
            return None;
        }

        unsafe {
            Some(*self.ptr.add(index))
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            free(self.ptr as *mut ());
        }
    }
}
```

## Common Patterns

### Custom Allocator

```blang
unsafe block allocator {
    static mut HEAP_START: *mut u8 = 0 as *mut u8;
    static mut HEAP_OFFSET: usize = 0;
    static mut HEAP_SIZE: usize = 0;

    pub fn init(size: usize) {
        let pages = (size + 65535) / 65536;
        memory_grow(pages as i32);

        HEAP_START = 0 as *mut u8;
        HEAP_SIZE = size;
        HEAP_OFFSET = 0;
    }

    pub fn alloc(size: usize, align: usize) -> *mut u8 {
        let offset = (HEAP_OFFSET + align - 1) & !(align - 1);

        if offset + size > HEAP_SIZE {
            return 0 as *mut u8;
        }

        let ptr = HEAP_START.add(offset);
        HEAP_OFFSET = offset + size;
        ptr
    }

    fn memory_grow(pages: i32) -> i32 {
        wasm! {
            (memory.grow (local.get $pages))
        }
    }
}
```

### Ring Buffer

```blang
pub struct RingBuffer<T> {
    buffer: *mut T,
    capacity: usize,
    read_pos: usize,
    write_pos: usize,
}

impl<T> RingBuffer<T> {
    pub fn new(capacity: usize) -> RingBuffer<T> {
        unsafe {
            let buffer = malloc(capacity * size_of::<T>()) as *mut T;
            RingBuffer {
                buffer,
                capacity,
                read_pos: 0,
                write_pos: 0,
            }
        }
    }

    pub fn push(&mut self, value: T) -> bool {
        let next_write = (self.write_pos + 1) % self.capacity;

        if next_write == self.read_pos {
            return false;  // Buffer full
        }

        unsafe {
            *self.buffer.add(self.write_pos) = value;
        }

        self.write_pos = next_write;
        true
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.read_pos == self.write_pos {
            return None;  // Buffer empty
        }

        unsafe {
            let value = *self.buffer.add(self.read_pos);
            self.read_pos = (self.read_pos + 1) % self.capacity;
            Some(value)
        }
    }
}
```

## Debugging Unsafe Code

### Logging and Tracing

```blang
unsafe fn debug_pointer_access(ptr: *const i32, index: usize) -> i32 {
    log.debug(format!(
        "Accessing pointer {:p} at offset {}",
        ptr, index
    ));

    let value = *ptr.add(index);

    log.debug(format!(
        "Read value {} from {:p}",
        value, ptr.add(index)
    ));

    value
}
```

### Runtime Checks (Debug Mode)

```blang
unsafe fn checked_access<T>(ptr: *const T, index: usize, len: usize) -> T {
    #[cfg(debug)]
    {
        assert!(!ptr.is_null(), "Null pointer access");
        assert!(index < len, "Index {} out of bounds (len: {})", index, len);
        assert!(
            ptr as usize % align_of::<T>() == 0,
            "Misaligned pointer access"
        );
    }

    *ptr.add(index)
}
```

### Testing Unsafe Code

```blang
#[test]
fn test_buffer_operations() {
    let mut buffer = Buffer::new(1024);

    // Test push
    for i in 0..100 {
        buffer.push(i as u8);
    }

    // Test get
    for i in 0..100 {
        assert_eq!(buffer.get(i), Some(i as u8));
    }

    // Test bounds
    assert_eq!(buffer.get(100), None);
}

#[test]
#[should_panic]
fn test_buffer_overflow() {
    let mut buffer = Buffer::new(10);

    for i in 0..20 {
        buffer.push(i as u8);  // Should panic at 10
    }
}
```

---

For more information, see:

- [Language Overview](language_overview.md) - Core language features
- [Component Mode](component_mode.md) - Building reactive UI components
- [Script Mode](script_mode.md) - Data processing and orchestration
