use super::{Camera, GfxState};
use crate::{loader::CIRCLE_MESH_ID, util::ID};
use bytemuck::{Pod, Zeroable};
use egui_wgpu::wgpu::{self, util::DeviceExt};
use glam::Vec2;
use std::{collections::HashMap, ops::Range};

pub struct Mesh {
    pub vertices: Vec<ModelVertex>,
    pub indices: Vec<u32>,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub model: glam::Mat4,
    pub fs_entry_point: String,
}

impl Mesh {
    pub fn update_2d_meshes(meshes: &mut HashMap<ID, Mesh>, queue: &wgpu::Queue, camera: &Camera) {
        if let Some(mesh) = meshes.get_mut(CIRCLE_MESH_ID) {
            let view_mat = camera.view_mat();
            let view_proj = camera.view_proj(&view_mat);
            let camera_right = view_proj.row(0).truncate().normalize();
            let camera_up = view_proj.row(1).truncate().normalize();

            for (vert, v_pos) in mesh.vertices.iter_mut().zip(VERTEX_POSITIONS) {
                vert.position = (camera_right * v_pos[0] + camera_up * v_pos[1]).into();
            }

            queue.write_buffer(&mesh.vertex_buffer, 0, bytemuck::cast_slice(&mesh.vertices));
        }
    }

    pub fn indices_range(&self) -> Range<u32> {
        0..self.indices.len() as u32
    }

    pub fn circle(gfx_state: &GfxState) -> Mesh {
        let indices = vec![0, 1, 2, 3, 2, 1];

        let mut vertices = Vec::new();

        for i in 0..4 {
            let v_pos = VERTEX_POSITIONS[i];
            let mut uv = v_pos * 0.5 + 0.5;
            uv.y = 1. - uv.y;

            vertices.push(ModelVertex {
                position: v_pos.extend(0.).into(),
                uv: uv.into(),
                normal: [0., 0., 1.],
                tangent: Default::default(),
                bitangent: Default::default(),
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
            vertices,
            indices,
            vertex_buffer,
            index_buffer,
            model: glam::Mat4::default(),
            fs_entry_point: "fs_circle".to_string(),
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
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 11]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

#[derive(Pod, Zeroable, Clone, Copy, Debug)]
#[repr(C)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub normal: [f32; 3],
    pub tangent: [f32; 3],
    pub bitangent: [f32; 3],
}

const VERTEX_POSITIONS: [Vec2; 4] = [
    Vec2::new(-1., -1.),
    Vec2::new(1., -1.),
    Vec2::new(-1., 1.),
    Vec2::new(1., 1.),
];
