#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct LightUniform {
    pub(crate) position: [f32; 3],
    pub(crate) _padding: u32,
    pub(crate) color: [f32; 3],
    pub(crate) _padding2: u32,
}
