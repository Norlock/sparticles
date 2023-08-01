use crate::texture::DiffuseTexture;
use egui_wgpu_backend::wgpu;

use super::app_state::AppState;

pub struct Particle {
    diffuse_texture: DiffuseTexture,
    render_pipeline: wgpu::RenderPipeline,
    compute_pipeline: wgpu::ComputePipeline,
    num_particles: u32,

    pub buffers: Vec<wgpu::Buffer>,
    pub bind_groups: Vec<wgpu::BindGroup>,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl Particle {}

impl AppState {
    pub fn create() {}
}
