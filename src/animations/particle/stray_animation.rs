use std::fs;

use crate::{
    model::{Clock, GfxState, ParticleState},
    traits::{Animation, CalculateBufferSize, CreateAnimation, CustomShader},
};
use egui_wgpu::wgpu;
use wgpu::util::DeviceExt;

#[derive(Clone, Copy)]
pub struct StrayUniform {
    pub stray_radians: f32,
    pub from_sec: f32,
    pub until_sec: f32,
}

impl StrayUniform {
    fn create_buffer_content(&self) -> [f32; 4] {
        [self.stray_radians, self.from_sec, self.until_sec, 0.]
    }
}

impl CreateAnimation for StrayUniform {
    fn create_animation(
        self: Box<Self>,
        gfx_state: &GfxState,
        particle: &ParticleState,
    ) -> Box<dyn Animation> {
        Box::new(StrayAnimation::new(*self, particle, &gfx_state.device))
    }
}

struct StrayAnimation {
    pipeline: wgpu::ComputePipeline,
    uniform: StrayUniform,
    bind_group: wgpu::BindGroup,
}

impl Animation for StrayAnimation {
    fn update(&mut self, _clock: &Clock, _gfx_state: &GfxState) {}

    fn compute<'a>(
        &'a self,
        particle: &'a ParticleState,
        clock: &Clock,
        compute_pass: &mut wgpu::ComputePass<'a>,
    ) {
        compute_pass.set_pipeline(&self.pipeline);
        compute_pass.set_bind_group(0, &particle.bind_groups[clock.get_bindgroup_nr()], &[]);
        compute_pass.set_bind_group(1, &self.bind_group, &[]);
        compute_pass.dispatch_workgroups(particle.dispatch_x_count, 1, 1);
    }

    fn recreate(&self, gfx_state: &GfxState, particle: &ParticleState) -> Box<dyn Animation> {
        Box::new(Self::new(self.uniform, particle, &gfx_state.device))
    }
}

impl StrayAnimation {
    fn new(uniform: StrayUniform, particle: &ParticleState, device: &wgpu::Device) -> Self {
        let shader = device.create_shader("stray_anim.wgsl", "Stray animation");

        let animation_uniform = uniform.create_buffer_content();

        let animation_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Stray buffer"),
            contents: bytemuck::cast_slice(&animation_uniform),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let animation_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // Uniform data
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: animation_uniform.cal_buffer_size(),
                    },
                    count: None,
                },
            ],
            label: None,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &animation_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: animation_buffer.as_entire_binding(),
            }],
            label: Some("Stray animation"),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Stray layout"),
            bind_group_layouts: &[&particle.bind_group_layout, &animation_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Stray animation pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "main",
        });

        Self {
            pipeline,
            bind_group,
            uniform,
        }
    }
}
