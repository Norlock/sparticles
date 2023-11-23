use super::state::FastFetch;
use super::{
    Camera, EmitterUniform, GfxState, GuiState, Material, Mesh, MeshRef, ModelVertex, State,
};
use crate::fx::PostProcessState;
use crate::loader::{Model, BUILTIN_ID};
use crate::traits::{CustomShader, Splitting};
use crate::traits::{EmitterAnimation, ParticleAnimation};
use crate::util::persistence::{ExportEmitter, ExportType};
use crate::util::ListAction;
use crate::util::{Persistence, ID};
use egui_wgpu::wgpu::{self, ShaderModule};
use egui_winit::egui::{ScrollArea, Ui};
use std::collections::HashMap;
use std::path::PathBuf;
use std::{
    fmt::{Debug, Formatter},
    num::NonZeroU64,
};
use wgpu::util::DeviceExt;

#[allow(unused)]
pub struct EmitterState {
    pipeline: wgpu::ComputePipeline,
    pipeline_layout: wgpu::PipelineLayout,
    render_pipeline: wgpu::RenderPipeline,

    emitter_buffer: wgpu::Buffer,
    particle_buffers: Vec<wgpu::Buffer>,
    particle_animations: Vec<Box<dyn ParticleAnimation>>,
    emitter_animations: Vec<Box<dyn EmitterAnimation>>,
    shader: ShaderModule,

    pub uniform: EmitterUniform,
    pub dispatch_x_count: u32,
    pub bgs: Vec<wgpu::BindGroup>,
    pub bg_layout: wgpu::BindGroupLayout,
    pub is_light: bool,
}

pub enum EmitterType<'a> {
    Lights,
    Normal {
        lights_layout: &'a wgpu::BindGroupLayout,
    },
}

pub struct CreateEmitterOptions<'a> {
    pub uniform: EmitterUniform,
    pub gfx_state: &'a GfxState,
    pub camera: &'a Camera,
    pub collection: &'a HashMap<ID, Model>,
    pub emitter_type: EmitterType<'a>,
}

pub struct RecreateEmitterOptions<'a> {
    pub old_self: &'a mut EmitterState,
    pub gfx_state: &'a GfxState,
    pub camera: &'a Camera,
    pub collection: &'a HashMap<ID, Model>,
    pub emitter_type: EmitterType<'a>,
}

impl<'a> EmitterState {
    pub fn id(&self) -> &str {
        &self.uniform.id
    }

    pub fn update_emitter(state: &mut State, encoder: &mut wgpu::CommandEncoder) {
        let State {
            camera,
            emitters,
            gfx_state,
            gui,
            collection,
            ..
        } = state;

        let settings = gui.emitter_settings.as_ref().unwrap();

        let (em, mut others) = emitters.split_item_mut(gui.selected_emitter_id);

        em.uniform.update_settings(settings);

        if settings.recreate {
            if em.is_light {
                *em = EmitterState::recreate_emitter(
                    RecreateEmitterOptions {
                        old_self: em,
                        gfx_state,
                        camera,
                        collection,
                        emitter_type: EmitterType::Lights,
                    },
                    encoder,
                );

                for other in others {
                    *other = EmitterState::recreate_emitter(
                        RecreateEmitterOptions {
                            old_self: other,
                            gfx_state,
                            camera,
                            collection,
                            emitter_type: EmitterType::Normal {
                                lights_layout: &em.bg_layout,
                            },
                        },
                        encoder,
                    );
                }
            } else {
                let lights = others.next().unwrap();

                EmitterState::recreate_emitter(
                    RecreateEmitterOptions {
                        old_self: em,
                        gfx_state,
                        camera,
                        collection,
                        emitter_type: EmitterType::Normal {
                            lights_layout: &lights.bg_layout,
                        },
                    },
                    encoder,
                );
            }
        }
    }

    pub fn update(state: &mut State) {
        let State {
            clock,
            emitters,
            gfx_state,
            camera,
            events,
            collection,
            ..
        } = state;

        if let Some(tag) = events.delete_emitter() {
            emitters.retain(|em| em.id() != &tag);
        } else if let Some(id) = events.create_emitter() {
            let options = CreateEmitterOptions {
                camera,
                uniform: EmitterUniform::new(id),
                collection,
                emitter_type: EmitterType::Normal {
                    lights_layout: &emitters[0].bg_layout,
                },
                gfx_state,
            };

            emitters.push(Self::new(options));
        }

        if let Some(model) = collection.get_mut(BUILTIN_ID) {
            let queue = &mut gfx_state.queue;
            Mesh::update(&mut model.meshes, queue, &camera);
        }

        for emitter in emitters.iter_mut() {
            emitter.uniform.update(clock);

            ListAction::update_list(&mut emitter.emitter_animations);

            for anim in emitter
                .emitter_animations
                .iter_mut()
                .filter(|item| item.enabled())
            {
                anim.animate(&mut emitter.uniform, clock);
            }

            let buffer_content_raw = emitter.uniform.create_buffer_content(collection);
            let buffer_content = bytemuck::cast_slice(&buffer_content_raw);

            gfx_state
                .queue
                .write_buffer(&emitter.emitter_buffer, 0, buffer_content);

            ListAction::update_list(&mut emitter.particle_animations);

            for anim in emitter.particle_animations.iter_mut() {
                anim.update(clock, gfx_state);
            }
        }
    }

    pub fn compute_particles(state: &'a mut State, encoder: &'a mut wgpu::CommandEncoder) {
        let State {
            clock,
            emitters,
            gfx_state,
            ..
        } = state;

        let nr = clock.get_bindgroup_nr();

        let mut c_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Compute pipeline"),
            timestamp_writes: None,
        });

        gfx_state.begin_scope("Compute", &mut c_pass);

        for emitter in emitters.iter() {
            gfx_state.begin_scope(&format!("Compute emitter: {}", emitter.id()), &mut c_pass);
            c_pass.set_pipeline(&emitter.pipeline);
            c_pass.set_bind_group(0, &emitter.bgs[nr], &[]);
            c_pass.dispatch_workgroups(emitter.dispatch_x_count, 1, 1);
            gfx_state.end_scope(&mut c_pass);

            gfx_state.begin_scope("Compute particle animations", &mut c_pass);
            for anim in emitter
                .particle_animations
                .iter()
                .filter(|item| item.enabled())
            {
                anim.compute(emitter, clock, &mut c_pass);
            }
            gfx_state.end_scope(&mut c_pass);
        }

        gfx_state.end_scope(&mut c_pass);
    }

    pub fn render_particles(state: &mut State, encoder: &mut wgpu::CommandEncoder) {
        let pp = &state.post_process;

        let mut r_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view: pp.frame_view(),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                }),
                Some(wgpu::RenderPassColorAttachment {
                    view: pp.split_view(),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                }),
            ],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: pp.depth_view(),
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        let clock = &state.clock;
        let emitters = &state.emitters;
        let camera = &state.camera;
        let gfx_state = &mut state.gfx_state;
        let collection = &mut state.collection;

        let nr = clock.get_alt_bindgroup_nr();

        gfx_state.begin_scope("Render", &mut r_pass);

        for em in emitters.iter() {
            let mesh = collection.get_mesh(&em.uniform.mesh);
            let mat = collection.get_mat(&em.uniform.material);

            gfx_state.begin_scope(&format!("Emitter: {}", em.id()), &mut r_pass);
            r_pass.set_pipeline(&em.render_pipeline);
            r_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            r_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            r_pass.set_bind_group(0, &camera.bg(), &[]);
            r_pass.set_bind_group(1, &mat.bg, &[]);
            r_pass.set_bind_group(2, &em.bgs[nr], &[]);

            if !em.is_light {
                r_pass.set_bind_group(3, &emitters[0].bgs[nr], &[]);
            }

            r_pass.draw_indexed(mesh.indices_range(), 0, 0..em.particle_count() as u32);
            gfx_state.end_scope(&mut r_pass);
        }

        gfx_state.end_scope(&mut r_pass);
    }

    pub fn recreate_emitter(
        options: RecreateEmitterOptions,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Self {
        let old_self = options.old_self;

        let mut new_self = Self::new(CreateEmitterOptions {
            uniform: old_self.uniform.clone(),
            gfx_state: options.gfx_state,
            camera: options.camera,
            collection: options.collection,
            emitter_type: options.emitter_type,
        });

        let gfx_state = options.gfx_state;

        for i in 0..2 {
            let old_buf = &old_self.particle_buffers[i];
            let new_buf = &new_self.particle_buffers[i];
            let buf_size = old_buf.size().min(new_buf.size());
            encoder.copy_buffer_to_buffer(old_buf, 0, new_buf, 0, buf_size);
        }

        for i in 0..old_self.particle_animations.len() {
            let animation = old_self.particle_animations[i].recreate(gfx_state, &new_self);
            new_self.push_particle_animation(animation);
        }

        std::mem::swap(
            &mut new_self.emitter_animations,
            &mut old_self.emitter_animations,
        );

        new_self
    }

    pub fn push_particle_animation(&mut self, animation: Box<dyn ParticleAnimation>) {
        self.particle_animations.push(animation);
    }

    pub fn push_emitter_animation(&mut self, animation: Box<dyn EmitterAnimation>) {
        self.emitter_animations.push(animation);
    }

    pub fn ui_emitter_animations(&mut self, ui: &mut Ui, gui_state: &GuiState) {
        ScrollArea::vertical()
            .auto_shrink([true; 2])
            .vscroll(true)
            .max_height(500.)
            .show(ui, |ui| {
                for anim in self.emitter_animations.iter_mut() {
                    ui.group(|ui| {
                        anim.create_ui(ui, gui_state);
                    });
                }
            });
    }

    pub fn ui_particle_animations(&mut self, ui: &mut Ui, gui_state: &GuiState) {
        ScrollArea::vertical()
            .auto_shrink([true; 2])
            .vscroll(true)
            .max_height(500.)
            .show(ui, |ui| {
                for anim in self.particle_animations.iter_mut() {
                    ui.group(|ui| {
                        anim.create_ui(ui, gui_state);
                    });
                }
            });
    }

    pub fn update_diffuse(&mut self, gfx_state: &GfxState, path: &mut PathBuf) {
        // TODO think about diffuse textures change without a model

        //self.uniform.texture_image = path.to_path_buf();
        //let tex =
        //gfx_state.diffuse_from_string(path.to_str().expect("Failed to convert pathbuf to str"));
        //self.diffuse_ctx = gfx_state.create_diffuse_context(&tex);
    }

    pub fn particle_count(&self) -> u64 {
        self.uniform.particle_count()
    }

    pub fn export(emitters: &[EmitterState]) {
        let mut to_export = Vec::new();

        for emitter in emitters.iter() {
            to_export.push(ExportEmitter {
                particle_animations: emitter
                    .particle_animations
                    .iter()
                    .map(|anim| anim.export())
                    .collect(),
                emitter: emitter.uniform.clone(),
                is_light: false,
                emitter_animations: emitter
                    .emitter_animations
                    .iter()
                    .map(|anim| anim.export())
                    .collect(),
            });
        }

        Persistence::write_to_file(to_export, ExportType::EmitterStates);
    }

    pub fn new(options: CreateEmitterOptions) -> Self {
        let gfx_state = options.gfx_state;
        let camera = options.camera;
        let uniform = options.uniform;
        let collection = options.collection;

        let device = &gfx_state.device;
        let emitter_buf_content = uniform.create_buffer_content(collection);

        let mut particle_buffers = Vec::<wgpu::Buffer>::new();
        let mut bind_groups = Vec::<wgpu::BindGroup>::new();

        for i in 0..2 {
            particle_buffers.push(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("Particle Buffer {}", i)),
                mapped_at_creation: false,
                size: uniform.particle_buffer_size(),
                usage: wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST,
            }));
        }

        let particle_buffer_size = NonZeroU64::new(particle_buffers[0].size());
        let emitter_buffer_size = NonZeroU64::new(emitter_buf_content.len() as u64 * 4);

        let visibility = match &options.emitter_type {
            EmitterType::Lights => {
                wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::VERTEX_FRAGMENT
            }
            EmitterType::Normal { lights_layout: _ } => {
                wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::VERTEX
            }
        };

        // Compute ---------
        let bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                    visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::VERTEX,
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

        let emitter_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Emitters buffer"),
            contents: bytemuck::cast_slice(&emitter_buf_content),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        for i in 0..2 {
            bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bg_layout,
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

        let particle_count = uniform.particle_count() as f64;
        let workgroup_size = 128f64;
        let dispatch_x_count = (particle_count / workgroup_size).ceil() as u32;

        let shader = device.create_shader("emitter.wgsl", "Emitter compute");

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute layout"),
            bind_group_layouts: &[&bg_layout],
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

        let material = collection.get_mat(&uniform.material);

        match &options.emitter_type {
            EmitterType::Lights => {
                shader = device.create_shader("light_particle.wgsl", "Light particle render");
                pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Light particle render Pipeline Layout"),
                    bind_group_layouts: &[&camera.bg_layout, &material.bg_layout, &bg_layout],
                    push_constant_ranges: &[],
                });
                is_light = true;
            }
            EmitterType::Normal { lights_layout } => {
                shader = device.create_shader("particle.wgsl", "Particle render");
                pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Particle render Pipeline Layout"),
                    bind_group_layouts: &[
                        &camera.bg_layout,
                        &material.bg_layout,
                        &bg_layout,
                        lights_layout,
                    ],
                    push_constant_ranges: &[],
                });
                is_light = false;
            }
        }

        let render_pipeline =
            Self::create_pipeline(&shader, &pipeline_layout, &uniform.mesh, material, device);

        EmitterState {
            uniform,
            pipeline,
            render_pipeline,
            pipeline_layout,
            bg_layout,
            bgs: bind_groups,
            particle_buffers,
            emitter_buffer,
            dispatch_x_count,
            particle_animations: vec![],
            emitter_animations: vec![],
            shader,
            is_light,
        }
    }

    fn create_pipeline(
        shader: &ShaderModule,
        layout: &wgpu::PipelineLayout,
        mesh_ref: &MeshRef,
        material: &Material,
        device: &wgpu::Device,
    ) -> wgpu::RenderPipeline {
        let fs_entry = if mesh_ref.collection_id == BUILTIN_ID {
            // TODO also different builtin shapes
            "fs_circle"
        } else {
            "fs_model"
        };

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[ModelVertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: fs_entry,
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format: PostProcessState::TEXTURE_FORMAT,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    Some(wgpu::ColorTargetState {
                        format: PostProcessState::TEXTURE_FORMAT,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::COLOR,
                    }),
                ],
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: material.ctx.cull_mode.clone(),
                ..Default::default()
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
        })
    }
}

impl GfxState {}

impl Debug for EmitterState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpawnState")
            .field("emitter", &self.uniform)
            .finish()
    }
}
