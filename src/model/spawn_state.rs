use super::{Camera, EmitterUniform, GfxState, State};
use crate::traits::{EmitterAnimation, ParticleAnimation};
use crate::{
    texture::DiffuseTexture,
    traits::{CalculateBufferSize, CustomShader},
};
use egui_wgpu::wgpu;
use egui_winit::egui::Ui;
use glam::Vec3;
use std::{
    fmt::{Debug, Formatter},
    num::NonZeroU64,
};
use wgpu::util::DeviceExt;

#[allow(dead_code)]
pub struct EmitterState {
    pipeline: wgpu::ComputePipeline,
    particle_buffers: Vec<wgpu::Buffer>,
    emitter_buffer: wgpu::Buffer,
    diffuse_texture: DiffuseTexture,
    render_pipeline: wgpu::RenderPipeline,
    particle_animations: Vec<Box<dyn ParticleAnimation>>,
    emitter_animations: Vec<Box<dyn EmitterAnimation>>,

    pub uniform: EmitterUniform,
    pub dispatch_x_count: u32,
    pub bind_groups: Vec<wgpu::BindGroup>,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub is_light: bool,
    pub gui: EmitterGuiState,
}

pub struct EmitterGuiState {
    pub spawn_count: u32,
    pub spawn_delay_sec: f32,
    pub particle_lifetime_sec: f32,
    pub recreate: bool,

    pub box_position: Vec3,
    pub box_dimensions: Vec3,
    pub box_rotation_deg: Vec3,

    pub diff_width_deg: f32,
    pub diff_depth_deg: f32,

    pub particle_speed_min: f32,
    pub particle_speed_max: f32,
    pub particle_size_min: f32,
    pub particle_size_max: f32,
}

pub struct CreateEmitterOptions<'a> {
    pub emitter_uniform: EmitterUniform,
    pub light_layout: Option<&'a wgpu::BindGroupLayout>,
    pub camera: &'a Camera,
}

impl<'a> EmitterState {
    pub fn id(&self) -> &str {
        &self.uniform.id
    }

    pub fn update_spawners(state: &mut State) {
        state
            .emitters
            .iter_mut()
            .chain(vec![&mut state.lights])
            .for_each(|spawner| {
                let queue = &state.gfx_state.queue;

                spawner.uniform.update(&state.clock);

                for anim in spawner.emitter_animations.iter_mut() {
                    anim.animate(&mut spawner.uniform, &state.clock);
                }

                let buffer_content_raw = spawner.uniform.create_buffer_content();
                let buffer_content = bytemuck::cast_slice(&buffer_content_raw);

                queue.write_buffer(&spawner.emitter_buffer, 0, buffer_content);

                for anim in spawner.particle_animations.iter_mut() {
                    anim.update(&state.clock, &state.gfx_state);
                }
            });
    }

    pub fn compute_particles(state: &'a State, encoder: &'a mut wgpu::CommandEncoder) {
        let State {
            clock,
            lights,
            emitters,
            ..
        } = state;

        let nr = clock.get_bindgroup_nr();

        let mut c_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Compute pipeline"),
        });

        let compute = |c_pass: &mut wgpu::ComputePass<'a>, emitter: &'a EmitterState| {
            c_pass.set_pipeline(&emitter.pipeline);
            c_pass.set_bind_group(0, &emitter.bind_groups[nr], &[]);
            c_pass.dispatch_workgroups(emitter.dispatch_x_count, 1, 1);

            for anim in emitter.particle_animations.iter() {
                anim.compute(emitter, clock, c_pass);
            }
        };

        compute(&mut c_pass, lights);

        for spawner in emitters.iter() {
            compute(&mut c_pass, spawner);
        }
    }

    pub fn render_particles(state: &State, encoder: &mut wgpu::CommandEncoder) {
        let mut r_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: state.frame_view(),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: state.depth_view(),
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        let State {
            camera,
            clock,
            lights,
            emitters,
            ..
        } = state;

        let nr = clock.get_alt_bindgroup_nr();

        // Light
        r_pass.set_pipeline(&lights.render_pipeline);
        r_pass.set_bind_group(0, &camera.bind_group, &[]);
        r_pass.set_bind_group(1, &lights.bind_groups[nr], &[]);
        r_pass.draw(0..4, 0..lights.particle_count() as u32);

        // Normal
        for spawner in emitters.iter() {
            r_pass.set_pipeline(&spawner.render_pipeline);
            r_pass.set_bind_group(0, &camera.bind_group, &[]);
            r_pass.set_bind_group(1, &spawner.bind_groups[nr], &[]);
            r_pass.set_bind_group(2, &lights.bind_groups[nr], &[]);
            r_pass.draw(0..4, 0..spawner.particle_count() as u32);
        }
    }

    pub fn recreate_spawner(
        &mut self,
        gfx_state: &GfxState,
        light_layout: Option<&'a wgpu::BindGroupLayout>,
        camera: &Camera,
    ) {
        let mut new_self = gfx_state.create_emitter_state(CreateEmitterOptions {
            emitter_uniform: self.uniform.clone(),
            light_layout,
            camera,
        });

        while let Some(animation) = self.particle_animations.pop() {
            new_self.push_particle_animation(animation.recreate(gfx_state, &new_self));
        }

        while let Some(animation) = self.emitter_animations.pop() {
            new_self.push_emitter_animation(animation);
        }

        *self = new_self;
    }

    pub fn push_particle_animation(&mut self, animation: Box<dyn ParticleAnimation>) {
        self.particle_animations.push(animation);
    }

    pub fn push_emitter_animation(&mut self, animation: Box<dyn EmitterAnimation>) {
        self.emitter_animations.push(animation);
    }

    pub fn gui_emitter_animations(&mut self, ui: &mut Ui) {
        for anim in self.emitter_animations.iter_mut() {
            anim.create_gui(ui);
            ui.separator();
        }
    }

    pub fn gui_particle_animations(&mut self, ui: &mut Ui) {
        for anim in self.particle_animations.iter_mut() {
            anim.create_gui(ui);
            ui.separator();
        }
    }

    pub fn particle_count(&self) -> u64 {
        self.uniform.particle_count()
    }
}

impl GfxState {
    pub fn create_emitter_state(&self, options: CreateEmitterOptions<'_>) -> EmitterState {
        let CreateEmitterOptions {
            emitter_uniform: uniform,
            light_layout,
            camera,
        } = options;

        let device = &self.device;
        let surface_config = &self.surface_config;

        let emitter_buf_content = uniform.create_buffer_content();
        let diffuse_texture = self.create_diffuse_texture(&uniform.texture_image);

        let particle_buffer_size = NonZeroU64::new(uniform.particle_buffer_size());
        let emitter_buffer_size = emitter_buf_content.cal_buffer_size();

        let visibility = if light_layout.is_none() {
            wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::VERTEX_FRAGMENT
        } else {
            wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::VERTEX
        };

        // Compute ---------
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
                    visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: emitter_buffer_size,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
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
                size: uniform.particle_buffer_size(),
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
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                    },
                ],
                label: None,
            }));
        }

        let particle_count = uniform.particle_count() as f64;
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
        let is_light;

        if let Some(light_layout) = &light_layout {
            is_light = false;
            shader = device.create_shader("particle.wgsl", "Particle render");
            pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Particle render Pipeline Layout"),
                bind_group_layouts: &[&camera.bind_group_layout, &bind_group_layout, light_layout],
                push_constant_ranges: &[],
            });
        } else {
            is_light = true;
            shader = device.create_shader("light_particle.wgsl", "Light particle render");
            pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Light particle render Pipeline Layout"),
                bind_group_layouts: &[&camera.bind_group_layout, &bind_group_layout],
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
                format: GfxState::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: true,
            },
            multiview: None,
        });

        let gui = uniform.create_gui();

        EmitterState {
            uniform,
            pipeline,
            render_pipeline,
            bind_group_layout,
            bind_groups,
            particle_buffers,
            emitter_buffer,
            dispatch_x_count,
            diffuse_texture,
            particle_animations: vec![],
            emitter_animations: vec![],
            is_light,
            gui,
        }
    }
}

impl Debug for EmitterState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpawnState")
            .field("emitter", &self.uniform)
            .finish()
    }
}
