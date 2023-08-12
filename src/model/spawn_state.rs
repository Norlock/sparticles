use crate::{texture::DepthTexture, traits::Animation};
use std::{
    fmt::{Debug, Formatter},
    num::NonZeroU64,
};

use crate::{
    texture::DiffuseTexture,
    traits::{CalculateBufferSize, CustomShader},
};

use super::{emitter::Emitter, gfx_state::GfxState, Camera, Clock};
use egui_wgpu::wgpu;
use egui_wgpu::wgpu::util::DeviceExt;
use glam::Vec3;

#[allow(dead_code)]
pub struct SpawnState {
    pipeline: wgpu::ComputePipeline,
    particle_buffers: Vec<wgpu::Buffer>,
    emitter_buffer: wgpu::Buffer,
    diffuse_texture: DiffuseTexture,
    render_pipeline: wgpu::RenderPipeline,
    animations: Vec<Box<dyn Animation>>,
    emitter: Emitter,

    pub id: String,
    pub dispatch_x_count: u32,
    pub bind_groups: Vec<wgpu::BindGroup>,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub gui: SpawnGuiState,

    pub is_light: bool,
}

pub struct SpawnGuiState {
    pub spawn_count: u32,
    pub spawn_delay_sec: f32,
    pub particle_lifetime_sec: f32,
    pub recreate: bool,

    pub box_position: Vec3,
    pub box_dimensions: Vec3,
    pub box_rotation_deg: Vec3,

    pub diff_width_deg: f32,
    pub diff_depth_deg: f32,

    pub particle_speed: f32,
    pub particle_size_min: f32,
    pub particle_size_max: f32,
}

pub struct SpawnOptions<'a> {
    pub id: String,
    pub emitter: Emitter,
    pub light_layout: Option<&'a wgpu::BindGroupLayout>,
    pub camera: &'a Camera,
}

impl<'a> SpawnState {
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

    pub fn handle_gui(
        &mut self,
        gfx_state: &GfxState,
        light_layout: Option<&'a wgpu::BindGroupLayout>,
        camera: &Camera,
    ) {
        self.emitter.handle_gui(&mut self.gui);

        if self.gui.recreate {
            self.recreate_spawner(gfx_state, light_layout, camera);
        }
    }

    pub fn compute(&'a self, clock: &Clock, compute_pass: &mut wgpu::ComputePass<'a>) {
        let bind_group_nr = clock.get_bindgroup_nr();

        compute_pass.set_pipeline(&self.pipeline);
        compute_pass.set_bind_group(0, &self.bind_groups[bind_group_nr], &[]);
        compute_pass.dispatch_workgroups(self.dispatch_x_count, 1, 1);

        for anim in self.animations.iter() {
            anim.compute(&self, &clock, compute_pass);
        }
    }

    pub fn render_light(
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

    pub fn render(
        &'a self,
        clock: &Clock,
        camera: &'a Camera,
        light_spawner: &'a SpawnState,
        render_pass: &mut wgpu::RenderPass<'a>,
    ) {
        let nr = clock.get_alt_bindgroup_nr();

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.diffuse_texture.bind_group, &[]);
        render_pass.set_bind_group(1, &camera.bind_group, &[]);
        render_pass.set_bind_group(2, &self.bind_groups[nr], &[]);
        render_pass.set_bind_group(3, &light_spawner.bind_groups[nr], &[]);

        render_pass.draw(0..4, 0..self.particle_count() as u32);
    }

    pub fn recreate_spawner(
        &mut self,
        gfx_state: &GfxState,
        light_layout: Option<&'a wgpu::BindGroupLayout>,
        camera: &Camera,
    ) {
        *self = gfx_state.create_spawner(SpawnOptions {
            id: self.id.clone(),
            emitter: self.emitter.clone(),
            light_layout,
            camera,
        });

        while let Some(animation) = self.animations.pop() {
            self.push_animation(animation.recreate(gfx_state, &self));
        }
    }

    pub fn push_animation(&mut self, animation: Box<dyn Animation>) {
        self.animations.push(animation);
    }

    pub fn particle_count(&self) -> u64 {
        self.emitter.particle_count()
    }
}

impl GfxState {
    pub fn create_spawner<'a>(&self, options: SpawnOptions<'a>) -> SpawnState {
        let SpawnOptions {
            id,
            emitter,
            light_layout,
            camera,
        } = options;

        let device = &self.device;
        let surface_config = &self.surface_config;

        let emitter_buf_content = emitter.create_buffer_content();
        let diffuse_texture = self.create_diffuse_texture();

        let particle_buffer_size = NonZeroU64::new(emitter.particle_buffer_size());
        let emitter_buffer_size = emitter_buf_content.cal_buffer_size();

        let visibility;

        if light_layout.is_none() {
            visibility = wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::VERTEX_FRAGMENT;
        } else {
            visibility = wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::VERTEX;
        }

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // Particles
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility,
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

        let shader = device.create_shader("emitter.wgsl", "Emitter compute");

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "main",
        });

        // Render ---------
        let shader;
        let pipeline_layout;

        if let Some(light_layout) = &light_layout {
            shader = device.create_shader("particle.wgsl", "Particle render");
            pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Particle render Pipeline Layout"),
                bind_group_layouts: &[
                    &diffuse_texture.bind_group_layout,
                    &camera.bind_group_layout,
                    &bind_group_layout,
                    &light_layout,
                ],
                push_constant_ranges: &[],
            });
        } else {
            shader = device.create_shader("light_particle.wgsl", "Particle render");
            pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Particle render Pipeline Layout"),
                bind_group_layouts: &[
                    &diffuse_texture.bind_group_layout,
                    &camera.bind_group_layout,
                    &bind_group_layout,
                ],
                push_constant_ranges: &[],
            });
        }

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DepthTexture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let gui = emitter.create_gui();

        SpawnState {
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
            id,
            gui,
            is_light: light_layout.is_none(), // Light layout incl. when not light
        }
    }
}

impl Debug for SpawnState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpawnState")
            .field("emitter", &self.emitter)
            .finish()
    }
}
