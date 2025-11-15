//! Memory allocation primitives for WASM-JS interop
//!
//! This module provides a simple memory allocation interface for WebAssembly modules.
//! The actual memory management is handled by the WASM linear memory, but these functions
//! provide a convenient API for allocating and deallocating memory that can be shared
//! between WASM and JavaScript.
//!
//! # Memory Contract
//!
//! - **WASM side**: Exports linear memory that JS can read/write
//! - **JS side**: Can read strings and data from WASM memory using pointers and lengths
//! - **Allocation**: WASM manages its own heap using a bump allocator or similar
//!
//! # Usage
//!
//! ```rust,no_run
//! use blang_runtime::memory::{alloc, dealloc};
//!
//! // Allocate 100 bytes
//! let ptr = alloc(100);
//!
//! // Use the memory...
//!
//! // Free when done
//! dealloc(ptr, 100);
//! ```
//!
//! # Implementation Notes
//!
//! For simple use cases, WASM modules can use a bump allocator where:
//! - A static mut variable tracks the current heap position
//! - `alloc()` advances the position and returns the old value
//! - `dealloc()` is a no-op (memory reuse comes from resetting the allocator)
//!
//! For more complex scenarios, integrate a proper allocator like `wee_alloc`.

#[cfg(target_arch = "wasm32")]
use core::alloc::{GlobalAlloc, Layout};

/// Simple bump allocator for WASM
///
/// This allocator never frees memory, just keeps allocating forward.
/// It's suitable for short-lived WASM instances or scenarios where
/// memory is reset between operations.
#[cfg(target_arch = "wasm32")]
pub struct BumpAllocator {
    next: core::sync::atomic::AtomicUsize,
}

#[cfg(target_arch = "wasm32")]
impl BumpAllocator {
    pub const fn new() -> Self {
        Self {
            next: core::sync::atomic::AtomicUsize::new(0),
        }
    }
}

#[cfg(target_arch = "wasm32")]
unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();

        // Get current position and align it
        let mut current = self
            .next
            .load(core::sync::atomic::Ordering::Relaxed);
        let aligned = (current + align - 1) & !(align - 1);

        // Reserve space
        let new = aligned + size;
        self.next
            .store(new, core::sync::atomic::Ordering::Relaxed);

        aligned as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Bump allocator doesn't free memory
    }
}

/// Allocate memory of the specified size
///
/// Returns a pointer to the allocated memory. The caller is responsible
/// for freeing this memory with `dealloc()` when done.
///
/// # Arguments
///
/// * `size` - Number of bytes to allocate
///
/// # Returns
///
/// Pointer to allocated memory, or null if allocation fails
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn alloc(size: u32) -> *mut u8 {
    unsafe {
        let layout = Layout::from_size_align_unchecked(size as usize, 1);
        ALLOCATOR.alloc(layout)
    }
}

/// Free previously allocated memory
///
/// # Arguments
///
/// * `ptr` - Pointer to memory to free
/// * `size` - Size of the memory block
///
/// # Safety
///
/// The pointer must have been returned by `alloc()` and not yet freed.
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn dealloc(ptr: *mut u8, size: u32) {
    unsafe {
        let layout = Layout::from_size_align_unchecked(size as usize, 1);
        ALLOCATOR.dealloc(ptr, layout);
    }
}

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator::new();

/// Helper function to allocate and copy a string into WASM memory
///
/// This is useful for passing strings from WASM to JS. The returned
/// pointer and length can be passed to JS functions.
///
/// # Returns
///
/// (pointer, length) tuple
#[cfg(target_arch = "wasm32")]
pub fn alloc_string(s: &str) -> (*const u8, u32) {
    let bytes = s.as_bytes();
    let len = bytes.len() as u32;
    let ptr = alloc(len);

    if !ptr.is_null() {
        unsafe {
            core::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len());
        }
    }

    (ptr, len)
}

// Non-WASM stubs for testing
#[cfg(not(target_arch = "wasm32"))]
pub fn alloc(_size: u32) -> *mut u8 {
    std::ptr::null_mut()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn dealloc(_ptr: *mut u8, _size: u32) {}

#[cfg(not(target_arch = "wasm32"))]
pub fn alloc_string(_s: &str) -> (*const u8, u32) {
    (std::ptr::null(), 0)
}
