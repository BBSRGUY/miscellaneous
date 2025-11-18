//! Batched matrix multiplication kernel.

use crate::{GpuContext, GpuError, Result};
use wgpu::{util::DeviceExt, BindGroupLayout, ComputePipeline};

/// WGSL shader for batched matrix multiplication.
///
/// Computes C = A × B where:
/// - A: [batch, M, K]
/// - B: [batch, K, N]
/// - C: [batch, M, N]
const MATMUL_SHADER: &str = r#"
struct Dimensions {
    batch: u32,
    M: u32,
    K: u32,
    N: u32,
}

@group(0) @binding(0) var<uniform> dims: Dimensions;
@group(0) @binding(1) var<storage, read> A: array<f32>;
@group(0) @binding(2) var<storage, read> B: array<f32>;
@group(0) @binding(3) var<storage, read_write> C: array<f32>;

// Tile size for shared memory optimization
const TILE_SIZE: u32 = 16u;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let batch_idx = global_id.z;
    let row = global_id.y;
    let col = global_id.x;

    // Bounds check
    if (batch_idx >= dims.batch || row >= dims.M || col >= dims.N) {
        return;
    }

    // Compute matrix multiplication
    var sum = 0.0;

    for (var k = 0u; k < dims.K; k++) {
        let a_idx = batch_idx * dims.M * dims.K + row * dims.K + k;
        let b_idx = batch_idx * dims.K * dims.N + k * dims.N + col;
        sum += A[a_idx] * B[b_idx];
    }

    let c_idx = batch_idx * dims.M * dims.N + row * dims.N + col;
    C[c_idx] = sum;
}
"#;

/// Shape for batched matrix multiplication.
#[derive(Debug, Clone, Copy)]
pub struct MatMulShape {
    /// Batch size
    pub batch: u32,
    /// Rows in A and C
    pub m: u32,
    /// Columns in A, rows in B
    pub k: u32,
    /// Columns in B and C
    pub n: u32,
}

impl MatMulShape {
    /// Create a new matrix multiplication shape.
    pub fn new(batch: u32, m: u32, k: u32, n: u32) -> Self {
        Self { batch, m, k, n }
    }

    /// Size of A matrix in elements.
    pub fn a_size(&self) -> usize {
        (self.batch * self.m * self.k) as usize
    }

    /// Size of B matrix in elements.
    pub fn b_size(&self) -> usize {
        (self.batch * self.k * self.n) as usize
    }

    /// Size of C matrix in elements.
    pub fn c_size(&self) -> usize {
        (self.batch * self.m * self.n) as usize
    }

    /// Size of dimension buffer in bytes.
    pub fn dims_size(&self) -> u64 {
        16 // 4 u32s = 16 bytes
    }

    /// Convert to bytes for uniform buffer.
    pub fn to_bytes(&self) -> [u8; 16] {
        let mut bytes = [0u8; 16];
        bytes[0..4].copy_from_slice(&self.batch.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.m.to_le_bytes());
        bytes[8..12].copy_from_slice(&self.k.to_le_bytes());
        bytes[12..16].copy_from_slice(&self.n.to_le_bytes());
        bytes
    }
}

/// Batched matrix multiplication kernel.
pub struct MatMulKernel {
    pipeline: ComputePipeline,
    bind_group_layout: BindGroupLayout,
}

impl MatMulKernel {
    /// Create a new matrix multiplication kernel.
    pub fn new(ctx: &GpuContext) -> Result<Self> {
        // Create shader module
        let shader = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("MatMul Shader"),
            source: wgpu::ShaderSource::Wgsl(MATMUL_SHADER.into()),
        });

        // Create bind group layout
        let bind_group_layout =
            ctx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("MatMul Bind Group Layout"),
                    entries: &[
                        // Dimensions uniform
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        // A matrix storage buffer
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        // B matrix storage buffer
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        // C matrix storage buffer
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });

        // Create pipeline layout
        let pipeline_layout = ctx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("MatMul Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        // Create compute pipeline
        let pipeline = ctx
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("MatMul Pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                cache: None,
            });

        Ok(Self {
            pipeline,
            bind_group_layout,
        })
    }

    /// Execute batched matrix multiplication on the GPU.
    ///
    /// # Arguments
    /// * `ctx` - GPU context
    /// * `a` - Input matrix A with shape [batch, M, K]
    /// * `b` - Input matrix B with shape [batch, K, N]
    /// * `shape` - Dimensions of the operation
    ///
    /// # Returns
    /// Output matrix C with shape [batch, M, N]
    pub async fn multiply(
        &self,
        ctx: &GpuContext,
        a: &[f32],
        b: &[f32],
        shape: MatMulShape,
    ) -> Result<Vec<f32>> {
        // Validate input sizes
        if a.len() != shape.a_size() {
            return Err(GpuError::ComputeError(format!(
                "Input A size mismatch: expected {}, got {}",
                shape.a_size(),
                a.len()
            )));
        }
        if b.len() != shape.b_size() {
            return Err(GpuError::ComputeError(format!(
                "Input B size mismatch: expected {}, got {}",
                shape.b_size(),
                b.len()
            )));
        }

        // Create buffers
        let dims_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("MatMul Dimensions Buffer"),
                contents: &shape.to_bytes(),
                usage: wgpu::BufferUsages::UNIFORM,
            });

        let a_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("MatMul A Buffer"),
                contents: bytemuck::cast_slice(a),
                usage: wgpu::BufferUsages::STORAGE,
            });

        let b_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("MatMul B Buffer"),
                contents: bytemuck::cast_slice(b),
                usage: wgpu::BufferUsages::STORAGE,
            });

        let c_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("MatMul C Buffer"),
            size: (shape.c_size() * std::mem::size_of::<f32>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // Create staging buffer for readback
        let staging_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("MatMul Staging Buffer"),
            size: (shape.c_size() * std::mem::size_of::<f32>()) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group
        let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("MatMul Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: dims_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: a_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: b_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: c_buffer.as_entire_binding(),
                },
            ],
        });

        // Create command encoder and dispatch compute
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("MatMul Encoder"),
            });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("MatMul Compute Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);

            // Calculate workgroup counts (round up division)
            let workgroup_count_x = (shape.n + 15) / 16;
            let workgroup_count_y = (shape.m + 15) / 16;
            let workgroup_count_z = shape.batch;

            compute_pass.dispatch_workgroups(
                workgroup_count_x,
                workgroup_count_y,
                workgroup_count_z,
            );
        }

        // Copy result to staging buffer
        encoder.copy_buffer_to_buffer(
            &c_buffer,
            0,
            &staging_buffer,
            0,
            (shape.c_size() * std::mem::size_of::<f32>()) as u64,
        );

        // Submit commands
        ctx.queue.submit(Some(encoder.finish()));

        // Read back results
        let buffer_slice = staging_buffer.slice(..);
        let (sender, receiver) = futures::channel::oneshot::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            sender.send(result).unwrap();
        });

        // Wait for GPU to finish
        ctx.device.poll(wgpu::Maintain::Wait);

        receiver
            .await
            .map_err(|_| GpuError::BufferMapError("Failed to receive map result".to_string()))?
            .map_err(|e| GpuError::BufferMapError(format!("Buffer mapping failed: {:?}", e)))?;

        // Copy data from mapped buffer
        let data = buffer_slice.get_mapped_range();
        let result: Vec<f32> = bytemuck::cast_slice(&data).to_vec();

        // Cleanup
        drop(data);
        staging_buffer.unmap();

        Ok(result)
    }
}
