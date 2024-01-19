use crate::{
    model::{gfx_state::Profiler, Camera, GfxState, SparState},
    shaders::ShaderOptions,
    traits::BufferContent,
};
use egui_wgpu::wgpu::{self, util::DeviceExt};
use encase::ShaderType;

#[derive(ShaderType, Debug)]
pub struct TerrainUniform {
    pub noise: f32,
    pub tex_size: u32,
}

pub struct IrradianceFace {
    pub view: wgpu::TextureView,
    pub camera: Camera,
}

pub struct TerrainGenerator {
    pub compute_pipeline: wgpu::ComputePipeline,
    pub irradiance_render_pipeline: wgpu::RenderPipeline,
    pub terrain_render_pipeline: wgpu::RenderPipeline,
    pub env_bindings: Vec<TerrainBinding>,
    pub env_bg_layout: wgpu::BindGroupLayout,
    pub env_texs: Vec<wgpu::Texture>,
    pub uniform_ctxs: Vec<TerrainUniformCtx>,
    pub has_been_executed: bool,
    pub irradiance_faces: Vec<IrradianceFace>,
    pub irradiance_tex: wgpu::Texture,
    pub irradiance_idx: usize,
}

pub struct TerrainBinding {
    pub bg: wgpu::BindGroup,
    pub view: wgpu::TextureView,
}

pub struct TerrainUniformCtx {
    pub buf: wgpu::Buffer,
    pub bg: wgpu::BindGroup,
    pub uniform: TerrainUniform,
    pub count_x: u32,
    pub count_y: u32,
}

const CUBE_SIZE: u32 = 2048;
const SDR_NOISE: &str = "noise.wgsl";
const SDR_TONEMAPPING: &str = "pbr/tonemapping.wgsl";
const SDR_CREATE_TERRAIN: &str = "terrain/create_terrain.wgsl";
const SDR_RENDER_TERRAIN: &str = "terrain/render_terrain.wgsl";
const TERRAIN_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

impl TerrainGenerator {
    pub fn environment_bg(&self) -> &wgpu::BindGroup {
        &self.env_bindings[self.uniform_ctxs.len() % 2].bg
    }

    pub fn irradiance_bg(&self) -> &wgpu::BindGroup {
        &self.env_bindings[self.uniform_ctxs.len() % 2].bg
    }

    pub fn environment_view(&self) -> &wgpu::TextureView {
        &self.env_bindings[self.uniform_ctxs.len() % 2].view
    }

    pub async fn compute(state: &mut SparState, encoder: &mut wgpu::CommandEncoder) {
        let tg = &mut state.terrain_generator;

        if !tg.has_been_executed {
            let mut c_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Terrain compute pass"),
                timestamp_writes: None,
            });

            let mut i = 0;

            let camera = &state.camera;

            for uniform_ctx in tg.uniform_ctxs.iter() {
                c_pass.set_pipeline(&tg.compute_pipeline);
                c_pass.set_bind_group(0, &tg.env_bindings[i % 2].bg, &[]);
                c_pass.set_bind_group(1, &uniform_ctx.bg, &[]);
                c_pass.set_bind_group(2, &camera.bg(), &[]);
                c_pass.dispatch_workgroups(uniform_ctx.count_x, uniform_ctx.count_y, 6);

                i += 1;
            }

            tg.has_been_executed = true;
        }

        //if tg.irradiance_idx < 6 {
        //let face = &tg.irradiance_faces[tg.irradiance_idx];
        //tg.irradiance_idx += 1;

        //let mut r_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        //label: Some("Terrain irradiance render"),
        //color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        //view: &face.view,
        //resolve_target: None,
        //ops: wgpu::Operations {
        //load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
        //store: wgpu::StoreOp::Store,
        //},
        //})],
        //depth_stencil_attachment: None,
        //occlusion_query_set: None,
        //timestamp_writes: None,
        //});

        //Profiler::begin_scope(gfx, "Render irradiance", &mut r_pass).await;
        //r_pass.set_pipeline(&tg.irradiance_render_pipeline);
        //r_pass.set_bind_group(0, &tg.environment_bg(), &[]);
        //r_pass.set_bind_group(1, face.camera.bg(), &[]);
        //r_pass.draw(0..3, 0..1);
        //Profiler::end_scope(gfx, &mut r_pass).await;
        //}

        //if tg.irradiance_idx == 6 {
        //encoder.copy_texture_to_texture(
        //wgpu::ImageCopyTextureBase {
        //texture: &tg.irradiance_tex,
        //aspect: wgpu::TextureAspect::All,
        //origin: wgpu::Origin3d::ZERO,
        //mip_level: 0,
        //},
        //wgpu::ImageCopyTextureBase {
        //texture: &tg.env_texs[1],
        //aspect: wgpu::TextureAspect::All,
        //origin: wgpu::Origin3d::ZERO,
        //mip_level: 0,
        //},
        //wgpu::Extent3d {
        //width: 2048,
        //height: 2048,
        //depth_or_array_layers: 6,
        //},
        //);

        //tg.irradiance_idx = usize::MAX;
        //}
    }

    pub async fn render(state: &SparState, encoder: &mut wgpu::CommandEncoder) {
        let tg = &state.terrain_generator;
        let pp = &state.post_process;
        let camera = &state.camera;
        let gfx = &state.gfx;

        {
            let mut r_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Post process render"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: pp.frame_view(),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: pp.depth_view(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            Profiler::begin_scope(gfx, "Render terrain", &mut r_pass).await;
            r_pass.set_pipeline(&tg.terrain_render_pipeline);
            r_pass.set_bind_group(0, &tg.environment_bg(), &[]);
            r_pass.set_bind_group(1, camera.bg(), &[]);
            r_pass.draw(0..3, 0..1);
            Profiler::end_scope(gfx, &mut r_pass).await;
        }
    }

    pub fn create_group_sizes(
        gfx: &GfxState,
        bg_layout: &wgpu::BindGroupLayout,
    ) -> Vec<TerrainUniformCtx> {
        let device = &gfx.device;

        let mut ctxs = Vec::new();
        let mut tex_size = 128;

        while tex_size <= CUBE_SIZE || tex_size <= CUBE_SIZE {
            let uniform = TerrainUniform {
                noise: 0.5,
                tex_size,
            };

            let contents = uniform.buffer_content();

            let buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Terrain uniform buffer"),
                contents: &contents,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            });

            let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Terrain uniform bind group"),
                layout: &bg_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        size: Some(uniform.size()),
                        buffer: &buf,
                        offset: 0,
                    }),
                }],
            });

            let count = tex_size / 16;

            ctxs.push(TerrainUniformCtx {
                buf,
                bg,
                uniform,
                count_x: count,
                count_y: count,
            });

            tex_size *= 2;
        }

        ctxs
    }

    fn create_cube_tex(gfx: &GfxState, is_render_attachment: bool) -> wgpu::Texture {
        let usage = if is_render_attachment {
            wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT
        } else {
            wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST
        };

        gfx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Environment cube"),
            size: wgpu::Extent3d {
                width: CUBE_SIZE as u32,
                height: CUBE_SIZE as u32,
                depth_or_array_layers: 6,
            },
            format: TERRAIN_FORMAT,
            usage,
            mip_level_count: 1, // Maybe do this
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            view_formats: &[],
        })
    }

    fn create_irradiance_faces(gfx: &GfxState) -> (wgpu::Texture, Vec<IrradianceFace>) {
        let irradiance_tex = Self::create_cube_tex(gfx, true);
        let mut irradiance_faces = Vec::new();

        let cameras = [
            glam::Vec3::new(1., 0., 0.),
            glam::Vec3::new(-1., 0., 0.),
            glam::Vec3::new(0., 1., 0.),
            glam::Vec3::new(0., -1., 0.),
            glam::Vec3::new(0., 0., 1.),
            glam::Vec3::new(0., 0., -1.),
        ];

        for i in 0..6 {
            let view = irradiance_tex.create_view(&wgpu::TextureViewDescriptor {
                dimension: Some(wgpu::TextureViewDimension::D2),
                array_layer_count: Some(1 as u32),
                base_array_layer: i as u32,
                ..Default::default()
            });

            let camera: Camera = Camera::new(gfx)
                .with_pos(glam::Vec3::ZERO)
                .with_view_dir(cameras[i].clone());

            irradiance_faces.push(IrradianceFace { camera, view });
        }

        (irradiance_tex, irradiance_faces)
    }

    pub fn new(gfx: &GfxState, camera: &Camera) -> Self {
        let device = &gfx.device;

        let create_shader = gfx.create_shader_builtin(ShaderOptions {
            if_directives: &[],
            files: &[SDR_NOISE, SDR_CREATE_TERRAIN],
            label: "Create Terrain SDR",
        });

        let uniform_bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(TerrainUniform::min_size()),
                },
                count: None,
            }],
        });

        let uniform_ctxs = Self::create_group_sizes(gfx, &uniform_bg_layout);

        let mut env_bindings = vec![];

        let env_bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Cube bind group layout"),
            entries: &[
                // Terrain write
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        format: TERRAIN_FORMAT,
                        access: wgpu::StorageTextureAccess::WriteOnly,
                    },
                    count: None,
                },
                // Terrain read
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        view_dimension: wgpu::TextureViewDimension::Cube,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        multisampled: false,
                    },
                    count: None,
                },
                // Terrain sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let cube_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let env_texs = vec![
            Self::create_cube_tex(gfx, false),
            Self::create_cube_tex(gfx, false),
        ];

        for i in 0..2 {
            let write_view = env_texs[i % 2].create_view(&wgpu::TextureViewDescriptor {
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                array_layer_count: Some(6),
                ..Default::default()
            });

            let read_view = env_texs[(i + 1) % 2].create_view(&wgpu::TextureViewDescriptor {
                dimension: Some(wgpu::TextureViewDimension::Cube),
                array_layer_count: Some(6),
                ..Default::default()
            });

            let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Cube bg"),
                layout: &env_bg_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&write_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&read_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&cube_sampler),
                    },
                ],
            });

            env_bindings.push(TerrainBinding {
                bg,
                view: read_view,
            });
        }

        // Create terrain
        let c_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Terrain Generator Pipeline Layout"),
            bind_group_layouts: &[&env_bg_layout, &uniform_bg_layout, &camera.bg_layout],
            push_constant_ranges: &[],
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Terrain compute pipeline"),
            layout: Some(&c_pipeline_layout),
            module: &create_shader,
            entry_point: "generate_terrain",
        });

        // Render terrain
        let render_shader = gfx.create_shader_builtin(ShaderOptions {
            if_directives: &[],
            files: &[SDR_TONEMAPPING, SDR_NOISE, SDR_RENDER_TERRAIN],
            label: "Render Terrain SDR",
        });

        let r_terrain_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Terrain Generator Pipeline Layout"),
                bind_group_layouts: &[&env_bg_layout, &camera.bg_layout],
                push_constant_ranges: &[],
            });

        let create_render_pipeline =
            |entry_point: &str, has_depth_stencil: bool| -> wgpu::RenderPipeline {
                let depth_stencil = if has_depth_stencil {
                    Some(wgpu::DepthStencilState {
                        format: GfxState::DEPTH_FORMAT,
                        depth_write_enabled: false,
                        depth_compare: wgpu::CompareFunction::LessEqual,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    })
                } else {
                    None
                };

                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Terrain render pipeline"),
                    layout: Some(&r_terrain_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &render_shader,
                        entry_point: "vs_main",
                        buffers: &[],
                    },
                    primitive: wgpu::PrimitiveState::default(),
                    depth_stencil,
                    multisample: wgpu::MultisampleState::default(),
                    fragment: Some(wgpu::FragmentState {
                        module: &render_shader,
                        entry_point,
                        targets: &[Some(wgpu::ColorTargetState {
                            format: GfxState::HDR_TEX_FORMAT,
                            blend: None,
                            write_mask: wgpu::ColorWrites::COLOR,
                        })],
                    }),
                    multiview: None,
                })
            };

        let terrain_render_pipeline = create_render_pipeline("fs_draw_terrain", true);
        let irradiance_render_pipeline = create_render_pipeline("fs_irradiance_convolution", false);

        let (irradiance_tex, irradiance_faces) = Self::create_irradiance_faces(gfx);

        Self {
            compute_pipeline,
            irradiance_render_pipeline,
            irradiance_faces,
            irradiance_tex,
            terrain_render_pipeline,
            env_bindings,
            uniform_ctxs,
            env_bg_layout,
            has_been_executed: false,
            env_texs,
            irradiance_idx: 0,
        }
    }
}
