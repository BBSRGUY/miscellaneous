//! Activation function kernels (GELU, ReLU).

use crate::{GpuContext, GpuError, Result};
use wgpu::util::DeviceExt;

/// WGSL shader for activation functions.
const ACTIVATION_SHADER: &str = r#"
struct Config {
    size: u32,
    activation_type: u32, // 0 = ReLU, 1 = GELU
}

@group(0) @binding(0) var<uniform> config: Config;
@group(0) @binding(1) var<storage, read> input: array<f32>;
@group(0) @binding(2) var<storage, read_write> output: array<f32>;

// GELU approximation: 0.5 * x * (1 + tanh(sqrt(2/π) * (x + 0.044715 * x^3)))
fn gelu(x: f32) -> f32 {
    let sqrt_2_over_pi = 0.7978845608;
    let coeff = 0.044715;
    let inner = sqrt_2_over_pi * (x + coeff * x * x * x);
    return 0.5 * x * (1.0 + tanh(inner));
}

// ReLU: max(0, x)
fn relu(x: f32) -> f32 {
    return max(0.0, x);
}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;

    if (idx >= config.size) {
        return;
    }

    let x = input[idx];

    if (config.activation_type == 0u) {
        output[idx] = relu(x);
    } else {
        output[idx] = gelu(x);
    }
}
"#;

/// Activation function type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivationType {
    /// Rectified Linear Unit: max(0, x)
    ReLU,
    /// Gaussian Error Linear Unit
    GELU,
}

impl ActivationType {
    fn to_u32(&self) -> u32 {
        match self {
            ActivationType::ReLU => 0,
            ActivationType::GELU => 1,
        }
    }
}

/// Activation function kernel.
pub struct ActivationKernel {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl ActivationKernel {
    /// Create a new activation kernel.
    pub fn new(ctx: &GpuContext) -> Result<Self> {
        // Create shader module
        let shader = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Activation Shader"),
            source: wgpu::ShaderSource::Wgsl(ACTIVATION_SHADER.into()),
        });

        // Create bind group layout
        let bind_group_layout =
            ctx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Activation Bind Group Layout"),
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
                        // Output buffer
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
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
                label: Some("Activation Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        // Create compute pipeline
        let pipeline = ctx
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Activation Pipeline"),
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

    /// Apply activation function to input tensor.
    ///
    /// # Arguments
    /// * `ctx` - GPU context
    /// * `input` - Input tensor
    /// * `activation_type` - Type of activation to apply
    ///
    /// # Returns
    /// Output tensor with activation applied
    pub async fn apply(
        &self,
        ctx: &GpuContext,
        input: &[f32],
        activation_type: ActivationType,
    ) -> Result<Vec<f32>> {
        let size = input.len();

        // Create config buffer
        let mut config_bytes = [0u8; 8];
        config_bytes[0..4].copy_from_slice(&(size as u32).to_le_bytes());
        config_bytes[4..8].copy_from_slice(&activation_type.to_u32().to_le_bytes());

        let config_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Activation Config Buffer"),
                contents: &config_bytes,
                usage: wgpu::BufferUsages::UNIFORM,
            });

        // Create input buffer
        let input_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Activation Input Buffer"),
                contents: bytemuck::cast_slice(input),
                usage: wgpu::BufferUsages::STORAGE,
            });

        // Create output buffer
        let output_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Activation Output Buffer"),
            size: (size * std::mem::size_of::<f32>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // Create staging buffer for readback
        let staging_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Activation Staging Buffer"),
            size: (size * std::mem::size_of::<f32>()) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group
        let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Activation Bind Group"),
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
                    resource: output_buffer.as_entire_binding(),
                },
            ],
        });

        // Create command encoder and dispatch compute
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Activation Encoder"),
            });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Activation Compute Pass"),
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
