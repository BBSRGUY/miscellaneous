//! # Forge WASM
//!
//! WebAssembly build exposing WebGPU kernels to the browser.
//!
//! This crate provides WASM bindings for running GPU-accelerated operations
//! in the browser using WebGPU.

use wasm_bindgen::prelude::*;

/// Initialize the WASM module.
#[wasm_bindgen(start)]
pub fn init() {
    // Initialization placeholder for the WASM module
    // In production, you might want to set up panic hooks, logging, etc.
}

/// Get the version of the Forge WASM module.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Simple test function for WASM.
#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello from Forge WASM, {}!", name)
}

/// WASM wrapper for GPU context (placeholder).
#[wasm_bindgen]
pub struct WasmGpuContext {
    initialized: bool,
}

#[wasm_bindgen]
impl WasmGpuContext {
    /// Create a new WASM GPU context.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { initialized: false }
    }

    /// Initialize the GPU context.
    pub async fn init(&mut self) -> Result<(), JsValue> {
        self.initialized = true;
        Ok(())
    }

    /// Check if the context is initialized.
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        let ver = version();
        assert!(!ver.is_empty());
    }

    #[test]
    fn test_greet() {
        let greeting = greet("Forge");
        assert!(greeting.contains("Forge"));
    }
}
