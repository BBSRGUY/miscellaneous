//! Integration tests for GPU kernels comparing against CPU reference implementations.

use forge_wgpu_core::{
    ActivationKernel, ActivationType, GpuContext, MatMulKernel, MatMulShape, RMSNormKernel,
};

/// CPU reference implementation of matrix multiplication.
fn cpu_matmul(a: &[f32], b: &[f32], shape: MatMulShape) -> Vec<f32> {
    let mut c = vec![0.0f32; shape.c_size()];

    for batch in 0..shape.batch as usize {
        for i in 0..shape.m as usize {
            for j in 0..shape.n as usize {
                let mut sum = 0.0;
                for k in 0..shape.k as usize {
                    let a_idx = batch * shape.m as usize * shape.k as usize
                        + i * shape.k as usize
                        + k;
                    let b_idx = batch * shape.k as usize * shape.n as usize
                        + k * shape.n as usize
                        + j;
                    sum += a[a_idx] * b[b_idx];
                }
                let c_idx = batch * shape.m as usize * shape.n as usize
                    + i * shape.n as usize
                    + j;
                c[c_idx] = sum;
            }
        }
    }

    c
}

/// CPU reference implementation of ReLU.
fn cpu_relu(input: &[f32]) -> Vec<f32> {
    input.iter().map(|&x| x.max(0.0)).collect()
}

/// CPU reference implementation of GELU.
fn cpu_gelu(input: &[f32]) -> Vec<f32> {
    input
        .iter()
        .map(|&x| {
            let sqrt_2_over_pi = 0.7978845608f32;
            let coeff = 0.044715f32;
            let inner = sqrt_2_over_pi * (x + coeff * x * x * x);
            0.5 * x * (1.0 + inner.tanh())
        })
        .collect()
}

/// CPU reference implementation of RMSNorm.
fn cpu_rmsnorm(input: &[f32], weight: &[f32], epsilon: f32) -> Vec<f32> {
    let size = input.len();

    // Compute mean of squares
    let mean_square: f32 = input.iter().map(|&x| x * x).sum::<f32>() / size as f32;

    // Compute RMS
    let rms = (mean_square + epsilon).sqrt();

    // Apply normalization and weight
    input
        .iter()
        .zip(weight.iter())
        .map(|(&x, &w)| (x / rms) * w)
        .collect()
}

/// Helper to compare two float arrays with relative tolerance.
fn assert_close(a: &[f32], b: &[f32], rel_tol: f32, abs_tol: f32, name: &str) {
    assert_eq!(
        a.len(),
        b.len(),
        "{}: Length mismatch: {} vs {}",
        name,
        a.len(),
        b.len()
    );

    for (i, (&a_val, &b_val)) in a.iter().zip(b.iter()).enumerate() {
        let abs_diff = (a_val - b_val).abs();
        let rel_diff = if b_val.abs() > 1e-8 {
            abs_diff / b_val.abs()
        } else {
            abs_diff
        };

        if abs_diff > abs_tol && rel_diff > rel_tol {
            panic!(
                "{}: Mismatch at index {}: GPU={}, CPU={}, abs_diff={}, rel_diff={}",
                name, i, a_val, b_val, abs_diff, rel_diff
            );
        }
    }

    println!("{}: ✓ GPU matches CPU (within tolerance)", name);
}

#[tokio::test]
#[ignore] // Requires GPU access - run with `cargo test -- --ignored`
async fn test_matmul_small() {
    let ctx = GpuContext::new().await.expect("Failed to create GPU context");
    let kernel = MatMulKernel::new(&ctx).expect("Failed to create MatMul kernel");

    // Test with small matrices: [2, 3, 4] × [2, 4, 5] -> [2, 3, 5]
    let shape = MatMulShape::new(2, 3, 4, 5);

    // Generate test data
    let a: Vec<f32> = (0..shape.a_size()).map(|i| (i % 7) as f32 * 0.5).collect();
    let b: Vec<f32> = (0..shape.b_size()).map(|i| (i % 5) as f32 * 0.3).collect();

    // Compute on GPU
    let gpu_result = kernel
        .multiply(&ctx, &a, &b, shape)
        .await
        .expect("GPU matmul failed");

    // Compute on CPU
    let cpu_result = cpu_matmul(&a, &b, shape);

    // Compare results
    assert_close(&gpu_result, &cpu_result, 1e-4, 1e-5, "MatMul Small");
}

#[tokio::test]
#[ignore] // Requires GPU access - run with `cargo test -- --ignored`
async fn test_matmul_single_batch() {
    let ctx = GpuContext::new().await.expect("Failed to create GPU context");
    let kernel = MatMulKernel::new(&ctx).expect("Failed to create MatMul kernel");

    // Test with single batch: [1, 4, 4] × [1, 4, 4] -> [1, 4, 4]
    let shape = MatMulShape::new(1, 4, 4, 4);

    // Identity-like matrices
    let a: Vec<f32> = vec![
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    ];
    let b: Vec<f32> = vec![
        2.0, 0.0, 0.0, 0.0,
        0.0, 3.0, 0.0, 0.0,
        0.0, 0.0, 4.0, 0.0,
        0.0, 0.0, 0.0, 5.0,
    ];

    // Compute on GPU
    let gpu_result = kernel
        .multiply(&ctx, &a, &b, shape)
        .await
        .expect("GPU matmul failed");

    // Compute on CPU
    let cpu_result = cpu_matmul(&a, &b, shape);

    // Compare results
    assert_close(&gpu_result, &cpu_result, 1e-5, 1e-6, "MatMul Single Batch");
}

#[tokio::test]
#[ignore] // Requires GPU access - run with `cargo test -- --ignored`
async fn test_relu() {
    let ctx = GpuContext::new().await.expect("Failed to create GPU context");
    let kernel = ActivationKernel::new(&ctx).expect("Failed to create Activation kernel");

    // Test with mixed positive/negative values
    let input = vec![-2.0, -1.0, -0.5, 0.0, 0.5, 1.0, 2.0, 3.5];

    // Compute on GPU
    let gpu_result = kernel
        .apply(&ctx, &input, ActivationType::ReLU)
        .await
        .expect("GPU ReLU failed");

    // Compute on CPU
    let cpu_result = cpu_relu(&input);

    // Compare results
    assert_close(&gpu_result, &cpu_result, 1e-6, 1e-7, "ReLU");
}

#[tokio::test]
#[ignore] // Requires GPU access - run with `cargo test -- --ignored`
async fn test_gelu() {
    let ctx = GpuContext::new().await.expect("Failed to create GPU context");
    let kernel = ActivationKernel::new(&ctx).expect("Failed to create Activation kernel");

    // Test with range of values
    let input: Vec<f32> = vec![-3.0, -2.0, -1.0, -0.5, 0.0, 0.5, 1.0, 2.0, 3.0];

    // Compute on GPU
    let gpu_result = kernel
        .apply(&ctx, &input, ActivationType::GELU)
        .await
        .expect("GPU GELU failed");

    // Compute on CPU
    let cpu_result = cpu_gelu(&input);

    // Compare results (GELU approximation may have slightly higher tolerance)
    assert_close(&gpu_result, &cpu_result, 1e-3, 1e-4, "GELU");
}

#[tokio::test]
#[ignore] // Requires GPU access - run with `cargo test -- --ignored`
async fn test_rmsnorm() {
    let ctx = GpuContext::new().await.expect("Failed to create GPU context");
    let kernel = RMSNormKernel::new(&ctx).expect("Failed to create RMSNorm kernel");

    // Test with small vector
    let input = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
    let weight = vec![1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
    let epsilon = 1e-6;

    // Compute on GPU
    let gpu_result = kernel
        .normalize(&ctx, &input, &weight, epsilon)
        .await
        .expect("GPU RMSNorm failed");

    // Compute on CPU
    let cpu_result = cpu_rmsnorm(&input, &weight, epsilon);

    // Compare results
    assert_close(&gpu_result, &cpu_result, 1e-4, 1e-5, "RMSNorm");
}

#[tokio::test]
#[ignore] // Requires GPU access - run with `cargo test -- --ignored`
async fn test_rmsnorm_with_learned_weights() {
    let ctx = GpuContext::new().await.expect("Failed to create GPU context");
    let kernel = RMSNormKernel::new(&ctx).expect("Failed to create RMSNorm kernel");

    // Test with learned weights
    let input = vec![0.5, 1.0, 1.5, 2.0, 2.5, 3.0];
    let weight = vec![0.8, 1.2, 0.9, 1.1, 1.0, 0.95];
    let epsilon = 1e-5;

    // Compute on GPU
    let gpu_result = kernel
        .normalize(&ctx, &input, &weight, epsilon)
        .await
        .expect("GPU RMSNorm failed");

    // Compute on CPU
    let cpu_result = cpu_rmsnorm(&input, &weight, epsilon);

    // Compare results
    assert_close(&gpu_result, &cpu_result, 1e-4, 1e-5, "RMSNorm Learned Weights");
}

#[tokio::test]
#[ignore] // Requires GPU access - run with `cargo test -- --ignored`
async fn test_matmul_larger() {
    let ctx = GpuContext::new().await.expect("Failed to create GPU context");
    let kernel = MatMulKernel::new(&ctx).expect("Failed to create MatMul kernel");

    // Test with larger matrices: [1, 32, 32] × [1, 32, 32] -> [1, 32, 32]
    let shape = MatMulShape::new(1, 32, 32, 32);

    // Generate random-like test data
    let a: Vec<f32> = (0..shape.a_size())
        .map(|i| ((i * 13 + 7) % 100) as f32 / 100.0)
        .collect();
    let b: Vec<f32> = (0..shape.b_size())
        .map(|i| ((i * 17 + 11) % 100) as f32 / 100.0)
        .collect();

    // Compute on GPU
    let gpu_result = kernel
        .multiply(&ctx, &a, &b, shape)
        .await
        .expect("GPU matmul failed");

    // Compute on CPU
    let cpu_result = cpu_matmul(&a, &b, shape);

    // Compare results (slightly higher tolerance for larger matrices due to accumulation)
    assert_close(&gpu_result, &cpu_result, 1e-3, 1e-4, "MatMul Larger");
}

#[tokio::test]
#[ignore] // Requires GPU access - run with `cargo test -- --ignored`
async fn test_activation_larger() {
    let ctx = GpuContext::new().await.expect("Failed to create GPU context");
    let kernel = ActivationKernel::new(&ctx).expect("Failed to create Activation kernel");

    // Test with larger tensor
    let input: Vec<f32> = (0..1024)
        .map(|i| (i as f32 / 256.0) - 2.0)
        .collect();

    // Test ReLU
    let gpu_relu = kernel
        .apply(&ctx, &input, ActivationType::ReLU)
        .await
        .expect("GPU ReLU failed");
    let cpu_relu = cpu_relu(&input);
    assert_close(&gpu_relu, &cpu_relu, 1e-6, 1e-7, "ReLU Larger");

    // Test GELU
    let gpu_gelu = kernel
        .apply(&ctx, &input, ActivationType::GELU)
        .await
        .expect("GPU GELU failed");
    let cpu_gelu = cpu_gelu(&input);
    assert_close(&gpu_gelu, &cpu_gelu, 1e-3, 1e-4, "GELU Larger");
}

#[tokio::test]
#[ignore] // Requires GPU access - run with `cargo test -- --ignored`
async fn test_rmsnorm_larger() {
    let ctx = GpuContext::new().await.expect("Failed to create GPU context");
    let kernel = RMSNormKernel::new(&ctx).expect("Failed to create RMSNorm kernel");

    // Test with larger vector (typical embedding dimension)
    let size = 512;
    let input: Vec<f32> = (0..size).map(|i| (i as f32 / 100.0).sin()).collect();
    let weight: Vec<f32> = (0..size).map(|i| 1.0 + (i as f32 / 1000.0)).collect();
    let epsilon = 1e-5;

    // Compute on GPU
    let gpu_result = kernel
        .normalize(&ctx, &input, &weight, epsilon)
        .await
        .expect("GPU RMSNorm failed");

    // Compute on CPU
    let cpu_result = cpu_rmsnorm(&input, &weight, epsilon);

    // Compare results
    assert_close(&gpu_result, &cpu_result, 1e-3, 1e-4, "RMSNorm Larger");
}
