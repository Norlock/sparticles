use bytemuck::{Pod, Zeroable};
use egui_wgpu::wgpu::{self, util::DeviceExt};
use glam::Vec2;
use std::ops::Range;

use super::{emitter::ModelType, Camera, EmitterState, GfxState};

pub struct Mesh {
    pub label: String,
    pub vertices: Vec<ModelVertex>,
    pub indices: Vec<u32>,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
}

pub struct Material {
    pub name: String,
    pub diffuse_texture: wgpu::Texture,
}

impl Mesh {
    pub fn update(emitter: &mut EmitterState, queue: &wgpu::Queue, camera: &Camera) {
        match emitter.uniform.model {
            ModelType::Circle => {
                let mesh = &mut emitter.mesh;
                let view_mat = camera.view_mat();
                let view_proj = camera.view_proj(&view_mat);
                let camera_right = view_proj.row(0).truncate().normalize();
                let camera_up = view_proj.row(1).truncate().normalize();

                mesh.vertices.iter_mut().enumerate().for_each(|(i, v)| {
                    v.position = (camera_right * VERTEX_POSITIONS[i][0]
                        + camera_up * VERTEX_POSITIONS[i][1])
                        .into();
                });

                queue.write_buffer(&mesh.vertex_buffer, 0, bytemuck::cast_slice(&mesh.vertices));
            }
            _ => {}
        }
    }

    pub fn indices_range(&self) -> Range<u32> {
        0..self.indices.len() as u32
    }

    pub fn circle(gfx_state: &GfxState) -> Mesh {
        let indices = vec![0, 1, 2, 1, 2, 3];

        let uvs = [
            Vec2::new(0., 1.).into(),
            Vec2::new(1., 1.).into(),
            Vec2::new(0., 0.).into(),
            Vec2::new(1., 0.).into(),
        ];

        let mut vertices = Vec::new();

        for i in 0..4 {
            vertices.push(ModelVertex {
                position: VERTEX_POSITIONS[i].extend(0.).into(),
                uv: uvs[i],
                normal: Default::default(),
            })
        }

        let device = &gfx_state.device;
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Circle Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Circle Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Mesh {
            label: "Circle".to_string(),
            vertices,
            indices,
            vertex_buffer,
            index_buffer,
        }
    }
}

impl ModelVertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

#[derive(Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub normal: [f32; 3],
}

const VERTEX_POSITIONS: [Vec2; 4] = [
    Vec2::new(-1., -1.),
    Vec2::new(1., -1.),
    Vec2::new(-1., 1.),
    Vec2::new(1., 1.),
];
