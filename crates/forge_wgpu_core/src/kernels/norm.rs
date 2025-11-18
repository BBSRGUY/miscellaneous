//! RMSNorm (Root Mean Square Normalization) kernel.

use crate::{GpuContext, GpuError, Result};
use wgpu::util::DeviceExt;

/// WGSL shader for RMSNorm.
///
/// RMSNorm formula: output = input / sqrt(mean(input^2) + epsilon) * weight
const RMSNORM_SHADER: &str = r#"
struct Config {
    size: u32,
    epsilon: f32,
}

@group(0) @binding(0) var<uniform> config: Config;
@group(0) @binding(1) var<storage, read> input: array<f32>;
@group(0) @binding(2) var<storage, read> weight: array<f32>;
@group(0) @binding(3) var<storage, read_write> output: array<f32>;

// Shared memory for reduction
var<workgroup> shared_sum: array<f32, 256>;

@compute @workgroup_size(256)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
    @builtin(workgroup_id) workgroup_id: vec3<u32>
) {
    let tid = local_id.x;
    let idx = global_id.x;

    // Compute sum of squares
    var local_sum = 0.0;
    if (idx < config.size) {
        let val = input[idx];
        local_sum = val * val;
    }

    shared_sum[tid] = local_sum;
    workgroupBarrier();

    // Parallel reduction to compute sum
    for (var stride = 128u; stride > 0u; stride = stride / 2u) {
        if (tid < stride && (tid + stride) < 256u) {
            shared_sum[tid] += shared_sum[tid + stride];
        }
        workgroupBarrier();
    }

    // Compute RMS normalization factor
    var rms: f32;
    if (tid == 0u) {
        let mean_square = shared_sum[0] / f32(config.size);
        rms = sqrt(mean_square + config.epsilon);
    }
    workgroupBarrier();

    // Broadcast RMS to all threads
    if (tid == 0u) {
        shared_sum[0] = rms;
    }
    workgroupBarrier();
    rms = shared_sum[0];

    // Apply normalization and weight
    if (idx < config.size) {
        output[idx] = (input[idx] / rms) * weight[idx];
    }
}
"#;

/// RMSNorm kernel for layer normalization.
pub struct RMSNormKernel {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl RMSNormKernel {
    /// Create a new RMSNorm kernel.
    pub fn new(ctx: &GpuContext) -> Result<Self> {
        // Create shader module
        let shader = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("RMSNorm Shader"),
            source: wgpu::ShaderSource::Wgsl(RMSNORM_SHADER.into()),
        });

        // Create bind group layout
        let bind_group_layout =
            ctx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("RMSNorm Bind Group Layout"),
                    entries: &[
                        // Config uniform
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
                        // Input buffer
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
                        // Weight buffer
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
                        // Output buffer
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
                label: Some("RMSNorm Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        // Create compute pipeline
        let pipeline = ctx
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("RMSNorm Pipeline"),
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

    /// Apply RMSNorm to input tensor.
    ///
    /// # Arguments
    /// * `ctx` - GPU context
    /// * `input` - Input tensor
    /// * `weight` - Learned weight parameters (same size as input)
    /// * `epsilon` - Small constant for numerical stability (typically 1e-6)
    ///
    /// # Returns
    /// Normalized output tensor
    pub async fn normalize(
        &self,
        ctx: &GpuContext,
        input: &[f32],
        weight: &[f32],
        epsilon: f32,
    ) -> Result<Vec<f32>> {
        let size = input.len();

        // Validate inputs
        if weight.len() != size {
            return Err(GpuError::ComputeError(format!(
                "Weight size mismatch: expected {}, got {}",
                size,
                weight.len()
            )));
        }

        // Create config buffer
        let mut config_bytes = [0u8; 8];
        config_bytes[0..4].copy_from_slice(&(size as u32).to_le_bytes());
        config_bytes[4..8].copy_from_slice(&epsilon.to_le_bytes());

        let config_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("RMSNorm Config Buffer"),
                contents: &config_bytes,
                usage: wgpu::BufferUsages::UNIFORM,
            });

        // Create input buffer
        let input_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("RMSNorm Input Buffer"),
                contents: bytemuck::cast_slice(input),
                usage: wgpu::BufferUsages::STORAGE,
            });

        // Create weight buffer
        let weight_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("RMSNorm Weight Buffer"),
                contents: bytemuck::cast_slice(weight),
                usage: wgpu::BufferUsages::STORAGE,
            });

        // Create output buffer
        let output_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("RMSNorm Output Buffer"),
            size: (size * std::mem::size_of::<f32>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // Create staging buffer for readback
        let staging_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("RMSNorm Staging Buffer"),
            size: (size * std::mem::size_of::<f32>()) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group
        let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("RMSNorm Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: config_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: input_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: weight_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: output_buffer.as_entire_binding(),
                },
            ],
        });

        // Create command encoder and dispatch compute
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("RMSNorm Encoder"),
            });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("RMSNorm Compute Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);

            // Calculate workgroup count (256 threads per workgroup)
            let workgroup_count = ((size as u32) + 255) / 256;
            compute_pass.dispatch_workgroups(workgroup_count, 1, 1);
        }

        // Copy result to staging buffer
        encoder.copy_buffer_to_buffer(
            &output_buffer,
            0,
            &staging_buffer,
            0,
            (size * std::mem::size_of::<f32>()) as u64,
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
