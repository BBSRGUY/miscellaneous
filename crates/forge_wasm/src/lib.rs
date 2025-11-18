//! # Forge WASM
//!
//! WebAssembly bindings for Forge GPU kernels.
//!
//! This crate provides JavaScript-interoperable GPU compute capabilities
//! for running ML operations in the browser via WebGPU.

use forge_wgpu_core::{
    ActivationKernel, ActivationType, GpuContext, MatMulKernel, MatMulShape,
    RMSNormKernel,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

mod utils;

/// Result type for WASM operations.
pub type Result<T> = std::result::Result<T, JsValue>;

/// Initialize panic hook for better error messages in the browser console.
#[wasm_bindgen(start)]
pub fn init() {
    utils::set_panic_hook();
    tracing_wasm::set_as_global_default();
}

/// GPU kernel manager for browser WebGPU operations.
#[wasm_bindgen]
pub struct ForgeWasm {
    context: GpuContext,
}

/// Matrix multiplication configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct MatMulConfig {
    pub batch: u32,
    pub m: u32,
    pub k: u32,
    pub n: u32,
}

#[wasm_bindgen]
impl MatMulConfig {
    #[wasm_bindgen(constructor)]
    pub fn new(batch: u32, m: u32, k: u32, n: u32) -> Self {
        Self { batch, m, k, n }
    }
}

/// Performance metrics for kernel execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct PerformanceMetrics {
    pub kernel_name: String,
    pub duration_ms: f64,
    pub input_size: usize,
    pub output_size: usize,
}

#[wasm_bindgen]
impl PerformanceMetrics {
    #[wasm_bindgen(getter)]
    pub fn kernel_name(&self) -> String {
        self.kernel_name.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn duration_ms(&self) -> f64 {
        self.duration_ms
    }

    #[wasm_bindgen(getter)]
    pub fn input_size(&self) -> usize {
        self.input_size
    }

    #[wasm_bindgen(getter)]
    pub fn output_size(&self) -> usize {
        self.output_size
    }
}

#[wasm_bindgen]
impl ForgeWasm {
    /// Initialize Forge WASM with WebGPU context.
    #[wasm_bindgen]
    pub async fn init() -> Result<ForgeWasm> {
        utils::log("Initializing Forge WASM with WebGPU...");

        let context = GpuContext::new()
            .await
            .map_err(|e| JsValue::from_str(&format!("Failed to create GPU context: {}", e)))?;

        let info = context.adapter_info();
        utils::log(&format!(
            "WebGPU adapter initialized: {} ({:?})",
            info.name, info.backend
        ));

        Ok(ForgeWasm { context })
    }

    /// Get GPU adapter information.
    #[wasm_bindgen]
    pub fn get_adapter_info(&self) -> JsValue {
        let info = self.context.adapter_info();
        let obj = js_sys::Object::new();

        js_sys::Reflect::set(&obj, &"name".into(), &info.name.into()).unwrap();
        js_sys::Reflect::set(&obj, &"vendor".into(), &format!("{}", info.vendor).into()).unwrap();
        js_sys::Reflect::set(&obj, &"device".into(), &format!("{}", info.device).into()).unwrap();
        js_sys::Reflect::set(
            &obj,
            &"deviceType".into(),
            &format!("{:?}", info.device_type).into(),
        )
        .unwrap();
        js_sys::Reflect::set(&obj, &"backend".into(), &format!("{:?}", info.backend).into())
            .unwrap();

        obj.into()
    }

    /// Run matrix multiplication on GPU.
    #[wasm_bindgen]
    pub async fn matmul(
        &self,
        a: &[f32],
        b: &[f32],
        config: &MatMulConfig,
    ) -> Result<js_sys::Float32Array> {
        let shape = MatMulShape::new(config.batch, config.m, config.k, config.n);

        if a.len() != shape.a_size() {
            return Err(JsValue::from_str(&format!(
                "Input A size mismatch: expected {}, got {}",
                shape.a_size(),
                a.len()
            )));
        }
        if b.len() != shape.b_size() {
            return Err(JsValue::from_str(&format!(
                "Input B size mismatch: expected {}, got {}",
                shape.b_size(),
                b.len()
            )));
        }

        let kernel = MatMulKernel::new(&self.context)
            .map_err(|e| JsValue::from_str(&format!("Failed to create MatMul kernel: {}", e)))?;

        let result = kernel
            .multiply(&self.context, a, b, shape)
            .await
            .map_err(|e| JsValue::from_str(&format!("MatMul execution failed: {}", e)))?;

        let result_array = js_sys::Float32Array::new_with_length(result.len() as u32);
        result_array.copy_from(&result);

        Ok(result_array)
    }

    /// Run matrix multiplication with performance metrics.
    #[wasm_bindgen]
    pub async fn matmul_with_metrics(
        &self,
        a: &[f32],
        b: &[f32],
        config: &MatMulConfig,
    ) -> Result<JsValue> {
        let start = js_sys::Date::now();

        let result = self.matmul(a, b, config).await?;

        let duration_ms = js_sys::Date::now() - start;

        let metrics = PerformanceMetrics {
            kernel_name: "MatMul".to_string(),
            duration_ms,
            input_size: a.len() + b.len(),
            output_size: result.length() as usize,
        };

        let obj = js_sys::Object::new();
        js_sys::Reflect::set(&obj, &"result".into(), &result.into()).unwrap();
        js_sys::Reflect::set(
            &obj,
            &"metrics".into(),
            &serde_wasm_bindgen::to_value(&metrics).unwrap(),
        )
        .unwrap();

        Ok(obj.into())
    }

    /// Apply activation function (ReLU or GELU) to input tensor.
    #[wasm_bindgen]
    pub async fn activation(
        &self,
        input: &[f32],
        activation_type: &str,
    ) -> Result<js_sys::Float32Array> {
        let activation = match activation_type.to_lowercase().as_str() {
            "relu" => ActivationType::ReLU,
            "gelu" => ActivationType::GELU,
            _ => return Err(JsValue::from_str("Invalid activation type (use 'relu' or 'gelu')")),
        };

        let kernel = ActivationKernel::new(&self.context).map_err(|e| {
            JsValue::from_str(&format!("Failed to create Activation kernel: {}", e))
        })?;

        let result = kernel
            .apply(&self.context, input, activation)
            .await
            .map_err(|e| JsValue::from_str(&format!("Activation execution failed: {}", e)))?;

        let result_array = js_sys::Float32Array::new_with_length(result.len() as u32);
        result_array.copy_from(&result);

        Ok(result_array)
    }

    /// Apply RMSNorm to input tensor.
    #[wasm_bindgen]
    pub async fn rmsnorm(
        &self,
        input: &[f32],
        weight: &[f32],
        epsilon: f32,
    ) -> Result<js_sys::Float32Array> {
        if input.len() != weight.len() {
            return Err(JsValue::from_str(&format!(
                "Input and weight size mismatch: {} vs {}",
                input.len(),
                weight.len()
            )));
        }

        let kernel = RMSNormKernel::new(&self.context)
            .map_err(|e| JsValue::from_str(&format!("Failed to create RMSNorm kernel: {}", e)))?;

        let result = kernel
            .normalize(&self.context, input, weight, epsilon)
            .await
            .map_err(|e| JsValue::from_str(&format!("RMSNorm execution failed: {}", e)))?;

        let result_array = js_sys::Float32Array::new_with_length(result.len() as u32);
        result_array.copy_from(&result);

        Ok(result_array)
    }

    /// Run comprehensive benchmark of all kernels.
    #[wasm_bindgen]
    pub async fn benchmark(&self) -> Result<JsValue> {
        let mut results = Vec::new();

        // Benchmark MatMul (small)
        {
            let config = MatMulConfig::new(1, 8, 8, 8);
            let a: Vec<f32> = (0..64).map(|i| i as f32 / 64.0).collect();
            let b: Vec<f32> = (0..64).map(|i| (i % 8) as f32 / 8.0).collect();

            let start = js_sys::Date::now();
            let _ = self.matmul(&a, &b, &config).await?;
            let duration = js_sys::Date::now() - start;

            results.push(PerformanceMetrics {
                kernel_name: "MatMul (8x8x8)".to_string(),
                duration_ms: duration,
                input_size: 128,
                output_size: 64,
            });
        }

        // Benchmark ReLU
        {
            let input: Vec<f32> = (0..1024).map(|i| (i as f32 / 256.0) - 2.0).collect();

            let start = js_sys::Date::now();
            let _ = self.activation(&input, "relu").await?;
            let duration = js_sys::Date::now() - start;

            results.push(PerformanceMetrics {
                kernel_name: "ReLU (1024)".to_string(),
                duration_ms: duration,
                input_size: 1024,
                output_size: 1024,
            });
        }

        // Benchmark GELU
        {
            let input: Vec<f32> = (0..1024).map(|i| (i as f32 / 256.0) - 2.0).collect();

            let start = js_sys::Date::now();
            let _ = self.activation(&input, "gelu").await?;
            let duration = js_sys::Date::now() - start;

            results.push(PerformanceMetrics {
                kernel_name: "GELU (1024)".to_string(),
                duration_ms: duration,
                input_size: 1024,
                output_size: 1024,
            });
        }

        // Benchmark RMSNorm
        {
            let input: Vec<f32> = (0..512).map(|i| (i as f32 / 100.0).sin()).collect();
            let weight: Vec<f32> = vec![1.0; 512];

            let start = js_sys::Date::now();
            let _ = self.rmsnorm(&input, &weight, 1e-5).await?;
            let duration = js_sys::Date::now() - start;

            results.push(PerformanceMetrics {
                kernel_name: "RMSNorm (512)".to_string(),
                duration_ms: duration,
                input_size: 1024,
                output_size: 512,
            });
        }

        serde_wasm_bindgen::to_value(&results).map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

/// Check if WebGPU is supported in the current browser.
#[wasm_bindgen]
pub fn is_webgpu_supported() -> bool {
    // Check if navigator.gpu exists
    if let Ok(window) = js_sys::global().dyn_into::<web_sys::Window>() {
        js_sys::Reflect::has(&window.navigator(), &"gpu".into()).unwrap_or(false)
    } else {
        false
    }
}

/// Get WebGPU support information.
#[wasm_bindgen]
pub async fn get_webgpu_info() -> Result<JsValue> {
    let obj = js_sys::Object::new();

    // Check if navigator.gpu exists via js_sys
    let has_gpu = if let Ok(window) = js_sys::global().dyn_into::<web_sys::Window>() {
        js_sys::Reflect::has(&window.navigator(), &"gpu".into()).unwrap_or(false)
    } else {
        false
    };

    js_sys::Reflect::set(&obj, &"supported".into(), &has_gpu.into()).unwrap();

    Ok(obj.into())
}
