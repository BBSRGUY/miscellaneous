//! # Forge WebGPU Core
//!
//! GPU acceleration for ML operations using WebGPU.
//!
//! This crate provides GPU-accelerated kernels for machine learning operations
//! using wgpu, designed to work on both native and WebAssembly targets.

mod context;
mod kernels;

pub use context::{GpuContext, GpuError, GpuOptions, Result};
pub use kernels::{
    activation::{ActivationKernel, ActivationType},
    matmul::{MatMulKernel, MatMulShape},
    norm::RMSNormKernel,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires GPU access
    async fn test_gpu_context_creation() {
        let result = GpuContext::new().await;
        if let Ok(context) = result {
            let info = context.adapter_info();
            println!("GPU: {} ({:?})", info.name, info.backend);
            assert!(!info.name.is_empty());
        }
    }
}
