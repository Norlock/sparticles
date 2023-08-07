use crate::traits::Animation;
use std::num::NonZeroU64;

use crate::{
    texture::DiffuseTexture,
    traits::{CalculateBufferSize, CustomShader},
};

use super::{emitter::Emitter, gfx_state::GfxState, Camera, Clock, GuiState};
use egui_wgpu_backend::wgpu::{self, util::DeviceExt};

#[allow(dead_code)]
pub struct ParticleState {
    pipeline: wgpu::ComputePipeline,
    particle_buffers: Vec<wgpu::Buffer>,
    emitter_buffer: wgpu::Buffer,
    diffuse_texture: DiffuseTexture,
    render_pipeline: wgpu::RenderPipeline,
    animations: Vec<Box<dyn Animation>>,

    pub emitter: Emitter,
    pub dispatch_x_count: u32,
    pub bind_groups: Vec<wgpu::BindGroup>,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl ParticleState {
    pub fn update(&mut self, gfx_state: &GfxState, clock: &Clock) {
        self.emitter.update(clock);

        let buffer_content_raw = self.emitter.create_buffer_content();
        let buffer_content = bytemuck::cast_slice(&buffer_content_raw);

        gfx_state
            .queue
            .write_buffer(&self.emitter_buffer, 0, &buffer_content);

        for anim in self.animations.iter_mut() {
            anim.update(&clock, gfx_state);
        }
    }

    pub fn handle_gui(&mut self, gui_state: &GuiState, gfx_state: &GfxState, camera: &Camera) {
        self.emitter.handle_gui(gui_state);

        if gui_state.update_spawn {
            *self = self.recreate_particle_state(&gfx_state, &camera);
        }
    }

    pub fn compute<'a>(&'a self, clock: &Clock, compute_pass: &mut wgpu::ComputePass<'a>) {
        let bind_group_nr = clock.get_bindgroup_nr();

        compute_pass.set_pipeline(&self.pipeline);
        compute_pass.set_bind_group(0, &self.bind_groups[bind_group_nr], &[]);
        compute_pass.dispatch_workgroups(self.dispatch_x_count, 1, 1);

        for anim in self.animations.iter() {
            anim.compute(&self, &clock, compute_pass);
        }
    }

    pub fn render<'a>(
        &'a self,
        clock: &Clock,
        camera: &'a Camera,
        render_pass: &mut wgpu::RenderPass<'a>,
    ) {
        let nr = clock.get_alt_bindgroup_nr();

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.diffuse_texture.bind_group, &[]);
        render_pass.set_bind_group(1, &camera.bind_group, &[]);
        render_pass.set_bind_group(2, &self.bind_groups[nr], &[]);
        render_pass.draw(0..4, 0..self.particle_count() as u32);
    }

    pub fn recreate_particle_state(&self, gfx_state: &GfxState, camera: &Camera) -> ParticleState {
        let emitter = self.emitter.clone();

        let mut particle = gfx_state.create_particle_state(emitter, camera);

        let animations: Vec<Box<dyn Animation>> = self
            .animations
            .iter()
            .map(|a| a.create_new(gfx_state, &particle))
            .collect();

        particle.set_animations(animations);

        return particle;
    }

    pub fn set_animations(&mut self, animations: Vec<Box<dyn Animation>>) {
        self.animations = animations;
    }

    pub fn particle_count_text(&self) -> String {
        format!("Particle count: {}", self.emitter.particle_count())
    }

    pub fn particle_count(&self) -> u64 {
        self.emitter.particle_count()
    }
}

impl GfxState {
    pub fn create_particle_state(&self, emitter: Emitter, camera: &Camera) -> ParticleState {
        let device = &self.device;
        let emitter_buf_content = emitter.create_buffer_content();
        let diffuse_texture = self.create_diffuse_texture();

        let particle_buffer_size = NonZeroU64::new(emitter.particle_buffer_size());
        let emitter_buffer_size = emitter_buf_content.cal_buffer_size();

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // Particles
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::VERTEX,
                    //| wgpu::ShaderStages::FRAGMENT,
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

        let render_pipeline =
            self.create_render_pipeline(&diffuse_texture, &camera, &bind_group_layout);

        ParticleState {
            emitter,
            pipeline,
            render_pipeline,
            bind_group_layout,
            bind_groups,
            particle_buffers,
            emitter_buffer,
            dispatch_x_count,
            diffuse_texture,
            animations: vec![],
        }
    }

    fn create_render_pipeline(
        &self,
        diffuse_texture: &DiffuseTexture,
        camera: &Camera,
        layout: &wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {
        let device = &self.device;
        let surface_config = &self.surface_config;

        let shader_str = include_str!("../shaders/particle.wgsl");
        let shader = device.create_shader(shader_str, "Particle render");

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Empty Render Pipeline Layout"),
                bind_group_layouts: &[
                    &diffuse_texture.bind_group_layout,
                    &camera.bind_group_layout,
                    &layout,
                ],
                push_constant_ranges: &[],
            });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: Some(wgpu::IndexFormat::Uint16),
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        })
    }
}
