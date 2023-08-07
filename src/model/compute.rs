use std::num::NonZeroU64;

use crate::traits::{CalculateBufferSize, CustomShader};

use super::{
    emitter::{Emitter, SpawnOptions},
    gfx_state::GfxState,
    Clock,
};
use egui_wgpu_backend::wgpu::{self, util::DeviceExt};

#[allow(dead_code)]
pub struct ComputeState {
    pipeline: wgpu::ComputePipeline,
    particle_buffers: Vec<wgpu::Buffer>,
    emitter_buffer: wgpu::Buffer,

    pub emitter: Emitter,
    pub dispatch_x_count: u32,
    pub bind_groups: Vec<wgpu::BindGroup>,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl ComputeState {
    pub fn update(&mut self, gfx_state: &GfxState, clock: &Clock) {
        self.emitter.update(&clock);

        let buffer_content_raw = self.emitter.create_buffer_content();
        let buffer_content = bytemuck::cast_slice(&buffer_content_raw);

        gfx_state
            .queue
            .write_buffer(&self.emitter_buffer, 0, &buffer_content);
    }

    pub fn compute<'a>(&'a self, clock: &Clock, compute_pass: &mut wgpu::ComputePass<'a>) {
        let bind_group_nr = clock.get_bindgroup_nr();

        compute_pass.set_pipeline(&self.pipeline);
        compute_pass.set_bind_group(0, &self.bind_groups[bind_group_nr], &[]);
        compute_pass.dispatch_workgroups(self.dispatch_x_count, 1, 1);
    }

    pub fn recreate_compute(&self, gfx_state: &GfxState, options: SpawnOptions) -> Self {
        let emitter = self.emitter.from_spawn_options(options);
        gfx_state.create_compute_state(emitter)
    }

    pub fn particle_count_text(&self) -> String {
        format!("Particle count: {}", self.emitter.particle_count())
    }

    pub fn particle_count(&self) -> u64 {
        self.emitter.particle_count()
    }
}

impl GfxState {
    pub fn create_compute_state(&self, emitter: Emitter) -> ComputeState {
        let device = &self.device;

        let emitter_buf_content = emitter.create_buffer_content();

        let particle_buffer_size = NonZeroU64::new(emitter.particle_buffer_size());
        let emitter_buffer_size = emitter_buf_content.cal_buffer_size();

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // Particles
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE
                        | wgpu::ShaderStages::VERTEX
                        | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: particle_buffer_size,
                    },
                    count: None,
                },
                // Particles
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: particle_buffer_size,
                    },
                    count: None,
                },
                // Emitter
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: emitter_buffer_size,
                    },
                    count: None,
                },
            ],
            label: None,
        });

        let mut particle_buffers = Vec::<wgpu::Buffer>::new();
        let mut bind_groups = Vec::<wgpu::BindGroup>::new();

        for i in 0..2 {
            particle_buffers.push(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("Particle Buffer {}", i)),
                mapped_at_creation: false,
                size: emitter.particle_buffer_size(),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
            }));
        }

        let emitter_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Emitters buffer"),
            contents: bytemuck::cast_slice(&emitter_buf_content),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        for i in 0..2 {
            bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: particle_buffers[i].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: particle_buffers[(i + 1) % 2].as_entire_binding(), // bind to opposite buffer
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: emitter_buffer.as_entire_binding(),
                    },
                ],
                label: None,
            }));
        }

        let particle_count = emitter.particle_count() as f64;
        let workgroup_size = 128f64;
        let dispatch_x_count = (particle_count / workgroup_size).ceil() as u32;

        let shader_str = include_str!("../shaders/emitter.wgsl");
        let shader = device.create_shader(shader_str, "Emitter compute");

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Transform pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "main",
        });

        ComputeState {
            emitter,
            pipeline,
            bind_group_layout,
            bind_groups,
            particle_buffers,
            emitter_buffer,
            dispatch_x_count,
        }
    }
}
