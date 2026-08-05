use crate::engine::rendering::model;
use std::sync::Arc;
use wgpu::util::DeviceExt;

#[derive(Clone, Debug)]
pub(crate) struct Instance {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
}

impl Instance {
    /// Converts to a GPU-ready representation, combining translation and rotation
    /// into a model matrix, and extracting a normal matrix from the rotation.
    fn to_raw(&self) -> InstanceRaw {
        let model =
            cgmath::Matrix4::from_translation(self.position) * cgmath::Matrix4::from(self.rotation);
        InstanceRaw {
            model: model.into(),
            normal: cgmath::Matrix3::from(self.rotation).into(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[allow(dead_code)]
pub(crate) struct InstanceRaw {
    pub model: [[f32; 4]; 4],
    pub normal: [[f32; 3]; 3],
}

impl model::Vertex for InstanceRaw {
    /// Describes the vertex buffer layout for instanced rendering.
    /// Slots 5–8 carry the model matrix (4×`Float32x4`), slots 9–11 carry the normal matrix (3×`Float32x3`).
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<InstanceRaw>() as wgpu::BufferAddress,

            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 19]>() as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 22]>() as wgpu::BufferAddress,
                    shader_location: 11,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub(crate) struct ChunkBlockGroup {
    pub model: Arc<model::Model>,
    pub instances: Vec<Instance>,
    pub instance_buffer: wgpu::Buffer,
}

impl ChunkBlockGroup {
    /// Creates a new group, immediately converting all instances to raw form and uploading them to a GPU vertex buffer.
    pub(crate) fn new(
        model: Arc<model::Model>,
        instances: Vec<Instance>,
        device: &wgpu::Device,
        label: &str,
    ) -> Self {
        let data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(&data),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        Self {
            model,
            instances,
            instance_buffer,
        }
    }

    /// Reupload instance data to the GPU, reallocating the buffer at 2× the required size if it no longer fits.
    pub(crate) fn rewrite_buffer(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let data = self
            .instances
            .iter()
            .map(Instance::to_raw)
            .collect::<Vec<_>>();

        let required_size = instance_buffer_size(data.len());

        if required_size > self.instance_buffer.size() {
            self.instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Instance Buffer"),
                size: required_size * 2,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        if !data.is_empty() {
            queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&data));
        }
    }
}

/// Returns the required buffer size in bytes for `instance_count` instances,
/// with a minimum of one `InstanceRaw` to avoid zero-size allocations.
fn instance_buffer_size(instance_count: usize) -> wgpu::BufferAddress {
    let size = instance_count * size_of::<InstanceRaw>();
    size.max(size_of::<InstanceRaw>()) as wgpu::BufferAddress
}
