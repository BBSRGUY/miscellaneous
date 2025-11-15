//! Blang WebAssembly Runtime
//!
//! This crate provides the host-side runtime that bridges Blang WebAssembly modules
//! with JavaScript/browser environments. It defines the contract between WASM and JS
//! for memory management, console operations, timers, and DOM manipulation.
//!
//! # Architecture
//!
//! The runtime is split into two parts:
//! 1. **Rust side (this crate)**: Defines the FFI interface and provides utilities
//!    for WASM modules to interact with the host environment.
//! 2. **JS side (runtime.js)**: Implements the actual host functionality using browser APIs.
//!
//! # Modules
//!
//! - `memory`: Memory allocation and management
//! - `console`: Logging functions (log, error, warn)
//! - `timer`: Timing primitives (setTimeout, clearTimeout)
//! - `dom`: DOM manipulation abstraction layer
//!
//! # Usage
//!
//! On the Rust/WASM side, import this crate and use the provided functions:
//!
//! ```rust,no_run
//! use blang_runtime::console;
//! use blang_runtime::dom;
//!
//! // Log to browser console
//! console::log("Hello from WASM!");
//!
//! // Create and manipulate DOM elements
//! let div_id = dom::create_element("div");
//! dom::set_text_content(div_id, "Hello, World!");
//! ```
//!
//! On the JS side, load the WASM module with the runtime:
//!
//! ```javascript
//! import { loadWasmModule, mountComponent } from './runtime.js';
//!
//! const wasmBytes = await fetch('component.wasm').then(r => r.arrayBuffer());
//! const instance = await loadWasmModule(wasmBytes);
//! mountComponent('#app', 'counter', instance);
//! ```

#![cfg_attr(target_arch = "wasm32", no_std)]

pub mod console;
pub mod dom;
pub mod memory;
pub mod timer;

// Re-export commonly used items
pub use console::{error, log, warn};
pub use dom::{
    add_event_listener, append_child, create_element, query_selector, remove_element,
    set_attribute, set_text_content, ElementId,
};
pub use memory::{alloc, dealloc};
pub use timer::{clear_timeout, set_timeout, TimerId};
