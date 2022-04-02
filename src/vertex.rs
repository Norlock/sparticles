#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct Vertex([f32; 3]);

pub const VERTICES: &[Vertex] = &[
    Vertex([-1.0, -1.0, 0.0]),
    Vertex([-1.0, 1.0, 0.0]),
    Vertex([1.0, -1.0, 0.0]),
    Vertex([1.0, 1.0, 0.0]),
];

impl Vertex {
    pub fn descriptor<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            }],
        }
    }
}
