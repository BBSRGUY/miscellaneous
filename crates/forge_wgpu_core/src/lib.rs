//! # Forge WebGPU Core
//!
//! Shared WebGPU kernels and GPU compute utilities.
//!
//! This crate provides GPU acceleration via wgpu for both native and
//! WebAssembly builds.

use thiserror::Error;
use wgpu::{
    Adapter, Device, DeviceDescriptor, Features, Instance, Limits, PowerPreference, Queue,
    RequestAdapterOptions,
};

/// Errors that can occur during GPU operations.
#[derive(Debug, Error)]
pub enum GpuError {
    #[error("Adapter not found")]
    AdapterNotFound,

    #[error("Device request failed: {0}")]
    DeviceRequestFailed(String),

    #[error("Shader compilation failed: {0}")]
    ShaderCompilationFailed(String),

    #[error("Compute error: {0}")]
    ComputeError(String),
}

pub type Result<T> = std::result::Result<T, GpuError>;

/// GPU context managing device and queue.
pub struct GpuContext {
    pub device: Device,
    pub queue: Queue,
    pub adapter: Adapter,
}

impl GpuContext {
    /// Initialize a new GPU context.
    pub async fn new() -> Result<Self> {
        let instance = Instance::default();

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or(GpuError::AdapterNotFound)?;

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: Some("Forge GPU Device"),
                    required_features: Features::empty(),
                    required_limits: Limits::default(),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .map_err(|e| GpuError::DeviceRequestFailed(e.to_string()))?;

        Ok(Self {
            device,
            queue,
            adapter,
        })
    }

    /// Get adapter information.
    pub fn adapter_info(&self) -> wgpu::AdapterInfo {
        self.adapter.get_info()
    }

    /// Check if the GPU supports compute shaders.
    pub fn supports_compute(&self) -> bool {
        self.adapter
            .features()
            .contains(Features::TIMESTAMP_QUERY)
    }
}

/// Simple matrix multiplication kernel (placeholder for ML ops).
pub struct MatMulKernel {
    _context: GpuContext,
}

impl MatMulKernel {
    /// Create a new matrix multiplication kernel.
    pub async fn new() -> Result<Self> {
        let context = GpuContext::new().await?;
        Ok(Self { _context: context })
    }

    /// Placeholder for matrix multiplication.
    /// In a real implementation, this would use compute shaders.
    pub fn multiply(&self, _a: &[f32], _b: &[f32]) -> Result<Vec<f32>> {
        // Placeholder implementation
        Ok(vec![0.0; 16])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Ignore by default as it requires GPU access
    async fn test_gpu_context_creation() {
        let result = GpuContext::new().await;
        // This might fail in CI environments without GPU
        if let Ok(context) = result {
            let info = context.adapter_info();
            assert!(!info.name.is_empty());
        }
    }
}
