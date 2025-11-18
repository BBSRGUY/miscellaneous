//! GPU context management and initialization.

use thiserror::Error;
use wgpu::{
    Adapter, Backends, Device, DeviceDescriptor, Features, Instance, InstanceDescriptor, Limits,
    PowerPreference, Queue, RequestAdapterOptions,
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

    #[error("Buffer map error: {0}")]
    BufferMapError(String),
}

pub type Result<T> = std::result::Result<T, GpuError>;

/// GPU context managing device and queue.
///
/// This is the main entry point for GPU operations. It handles initialization
/// of the WebGPU instance, adapter selection, and device creation.
pub struct GpuContext {
    /// The GPU device
    pub device: Device,
    /// The command queue
    pub queue: Queue,
    /// The adapter (for info queries)
    pub adapter: Adapter,
}

impl GpuContext {
    /// Initialize a new GPU context with default settings.
    ///
    /// This will try to select a high-performance discrete GPU if available.
    pub async fn new() -> Result<Self> {
        Self::with_options(GpuOptions::default()).await
    }

    /// Initialize a GPU context with custom options.
    pub async fn with_options(options: GpuOptions) -> Result<Self> {
        // Create instance with specified backends
        let instance = Instance::new(InstanceDescriptor {
            backends: options.backends,
            ..Default::default()
        });

        // Request adapter with specified preferences
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: options.power_preference,
                compatible_surface: None,
                force_fallback_adapter: options.force_fallback,
            })
            .await
            .ok_or(GpuError::AdapterNotFound)?;

        // Log adapter info
        let info = adapter.get_info();
        tracing::info!(
            "Selected GPU: {} ({:?}, {:?})",
            info.name,
            info.backend,
            info.device_type
        );

        // Determine required features and limits
        let mut features = Features::empty();
        if options.enable_timestamps {
            features |= Features::TIMESTAMP_QUERY;
        }

        let limits = if options.high_limits {
            Limits {
                max_buffer_size: 1 << 30, // 1 GB
                max_storage_buffer_binding_size: 1 << 30,
                max_compute_workgroup_size_x: 1024,
                max_compute_workgroup_size_y: 1024,
                max_compute_workgroup_size_z: 64,
                max_compute_workgroups_per_dimension: 65535,
                ..Limits::default()
            }
        } else {
            Limits::default()
        };

        // Request device and queue
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: Some("Forge GPU Device"),
                    required_features: features,
                    required_limits: limits,
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

    /// Get adapter features.
    pub fn features(&self) -> Features {
        self.adapter.features()
    }

    /// Get adapter limits.
    pub fn limits(&self) -> Limits {
        self.adapter.limits()
    }

    /// Check if the GPU supports compute shaders.
    pub fn supports_compute(&self) -> bool {
        // All WebGPU implementations support compute
        true
    }
}

/// Options for GPU context initialization.
#[derive(Debug, Clone)]
pub struct GpuOptions {
    /// Backend selection (Vulkan, Metal, DX12, WebGPU, etc.)
    pub backends: Backends,
    /// Power preference for adapter selection
    pub power_preference: PowerPreference,
    /// Force software fallback adapter
    pub force_fallback: bool,
    /// Enable timestamp queries for profiling
    pub enable_timestamps: bool,
    /// Request higher limits for large computations
    pub high_limits: bool,
}

impl Default for GpuOptions {
    fn default() -> Self {
        Self {
            backends: Backends::all(),
            power_preference: PowerPreference::HighPerformance,
            force_fallback: false,
            enable_timestamps: false,
            high_limits: true,
        }
    }
}

impl GpuOptions {
    /// Create options preferring discrete GPUs.
    pub fn discrete_gpu() -> Self {
        Self {
            power_preference: PowerPreference::HighPerformance,
            ..Default::default()
        }
    }

    /// Create options preferring integrated/low-power GPUs.
    pub fn integrated_gpu() -> Self {
        Self {
            power_preference: PowerPreference::LowPower,
            ..Default::default()
        }
    }

    /// Create options for CPU software rendering.
    pub fn software() -> Self {
        Self {
            force_fallback: true,
            ..Default::default()
        }
    }
}
