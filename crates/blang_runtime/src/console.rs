//! Console logging functions for WASM-JS interop
//!
//! This module provides functions to log messages from WebAssembly to the browser console.
//! The actual logging is implemented in JavaScript and imported into the WASM module.
//!
//! # Contract
//!
//! - **WASM side**: Calls these functions with string pointers and lengths
//! - **JS side**: Reads strings from WASM memory and calls console.log/error/warn
//!
//! # Usage
//!
//! ```rust,no_run
//! use blang_runtime::console;
//!
//! console::log("Application started");
//! console::warn("This is a warning");
//! console::error("Something went wrong!");
//! ```
//!
//! # String Encoding
//!
//! Strings are passed as UTF-8 encoded bytes with a pointer and length.
//! JavaScript uses TextDecoder to convert them to JS strings.

#[cfg(target_arch = "wasm32")]
extern "C" {
    /// Import from JS: console.log
    ///
    /// Logs a message to the browser console.
    ///
    /// # Arguments
    ///
    /// * `ptr` - Pointer to UTF-8 encoded string in WASM memory
    /// * `len` - Length of the string in bytes
    fn console_log(ptr: *const u8, len: u32);

    /// Import from JS: console.error
    ///
    /// Logs an error message to the browser console.
    ///
    /// # Arguments
    ///
    /// * `ptr` - Pointer to UTF-8 encoded string in WASM memory
    /// * `len` - Length of the string in bytes
    fn console_error(ptr: *const u8, len: u32);

    /// Import from JS: console.warn
    ///
    /// Logs a warning message to the browser console.
    ///
    /// # Arguments
    ///
    /// * `ptr` - Pointer to UTF-8 encoded string in WASM memory
    /// * `len` - Length of the string in bytes
    fn console_warn(ptr: *const u8, len: u32);
}

/// Log a message to the console
///
/// # Example
///
/// ```rust,no_run
/// use blang_runtime::console;
///
/// console::log("Hello, world!");
/// ```
#[cfg(target_arch = "wasm32")]
pub fn log(msg: &str) {
    let bytes = msg.as_bytes();
    unsafe {
        console_log(bytes.as_ptr(), bytes.len() as u32);
    }
}

/// Log an error message to the console
///
/// # Example
///
/// ```rust,no_run
/// use blang_runtime::console;
///
/// console::error("Failed to load resource");
/// ```
#[cfg(target_arch = "wasm32")]
pub fn error(msg: &str) {
    let bytes = msg.as_bytes();
    unsafe {
        console_error(bytes.as_ptr(), bytes.len() as u32);
    }
}

/// Log a warning message to the console
///
/// # Example
///
/// ```rust,no_run
/// use blang_runtime::console;
///
/// console::warn("Deprecated API usage");
/// ```
#[cfg(target_arch = "wasm32")]
pub fn warn(msg: &str) {
    let bytes = msg.as_bytes();
    unsafe {
        console_warn(bytes.as_ptr(), bytes.len() as u32);
    }
}

// Non-WASM stubs for testing
#[cfg(not(target_arch = "wasm32"))]
pub fn log(msg: &str) {
    println!("[LOG] {}", msg);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn error(msg: &str) {
    eprintln!("[ERROR] {}", msg);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn warn(msg: &str) {
    eprintln!("[WARN] {}", msg);
}

/// Format and log a message (like println!)
///
/// This is a convenience macro for formatted logging.
///
/// # Example
///
/// ```rust,no_run
/// use blang_runtime::log_fmt;
///
/// let count = 42;
/// log_fmt!("Count: {}", count);
/// ```
#[macro_export]
macro_rules! log_fmt {
    ($($arg:tt)*) => {{
        #[cfg(target_arch = "wasm32")]
        {
            extern crate alloc;
            use alloc::format;
            let msg = format!($($arg)*);
            $crate::console::log(&msg);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let msg = format!($($arg)*);
            $crate::console::log(&msg);
        }
    }};
}
