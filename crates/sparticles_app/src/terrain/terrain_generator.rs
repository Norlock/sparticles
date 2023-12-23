use egui_wgpu::wgpu::{self, util::DeviceExt};
use encase::ShaderType;

use crate::{
    model::{Camera, GfxState, SparState},
    shaders::{ShaderOptions, SDR_TONEMAPPING},
    traits::{BufferContent, CreateFxView},
};

#[derive(ShaderType)]
pub struct TerrainUniform {
    pub noise: f32,
    pub group_size: f32,
}

pub struct TerrainGenerator {
    pub compute_pipeline: wgpu::ComputePipeline,
    pub cube_bgs: Vec<wgpu::BindGroup>,
    pub uniform_ctxs: Vec<TerrainUniformCtx>,
}

pub struct TerrainUniformCtx {
    pub buf: wgpu::Buffer,
    pub bg: wgpu::BindGroup,
    pub uniform: TerrainUniform,
    pub count_x: u32,
    pub count_y: u32,
}

const SDR_NOISE: &str = "noise.wgsl";
const SDR_TERRAIN: &str = "terrain/terrain.wgsl";

impl TerrainGenerator {
    pub async fn update(state: &mut SparState) {
        //let clock = &state.clock;
        //let gfx = state.gfx.read().await;
        //let tg = &mut state.terrain_generator;

        //gfx.queue
        //.write_buffer(&tg.buf, 0, &tg.uniform.buffer_content());
    }

    pub fn resize(state: &mut SparState) {
        //
    }

    pub fn compute(state: &SparState, encoder: &mut wgpu::CommandEncoder) {
        let tg = &state.terrain_generator;
        let pp = &state.post_process;
        let camera = &state.camera;

        let mut c_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Terrain compute pass"),
            timestamp_writes: None,
        });

        let mut i = 0;
        for uniform_ctx in tg.uniform_ctxs.iter() {
            c_pass.set_pipeline(&tg.compute_pipeline);
            c_pass.set_bind_group(0, &tg.cube_bgs[i % 2], &[]);
            c_pass.set_bind_group(1, &uniform_ctx.bg, &[]);
            c_pass.dispatch_workgroups(uniform_ctx.count_x, uniform_ctx.count_y, 1);

            i += 1;
        }
    }

    pub fn create_group_sizes(
        gfx: &GfxState,
        bg_layout: &wgpu::BindGroupLayout,
    ) -> Vec<TerrainUniformCtx> {
        let device = &gfx.device;
        let (width, height) = gfx.dimensions();

        let mut tex_size = 128.;
        let mut ctxs = Vec::new();

        while tex_size < width || tex_size < height {
            let uniform = TerrainUniform {
                noise: 0.5,
                group_size: tex_size,
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

            let count = (tex_size / 16.) as u32;

            ctxs.push(TerrainUniformCtx {
                buf,
                bg,
                uniform,
                count_x: count,
                count_y: count,
            });

            tex_size *= 2.;
        }

        ctxs
    }

    pub async fn new(gfx: &GfxState, _camera: &Camera) -> Self {
        let device = &gfx.device;

        let shader = gfx.create_shader_builtin(ShaderOptions {
            if_directives: &[],
            files: &[SDR_TONEMAPPING, SDR_NOISE, SDR_TERRAIN],
            label: "Terrain SDR",
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

        let (width, height) = gfx.dimensions();

        let mut cube_texs = vec![];
        let mut cube_bgs = vec![];
        let mut cube_views = vec![];

        let cube_bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Cube bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        view_dimension: wgpu::TextureViewDimension::Cube,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        access: wgpu::StorageTextureAccess::WriteOnly,
                    },
                    count: None,
                },
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
            ],
        });

        for i in 0..2 {
            cube_texs.push(device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Terrain cube"),
                size: wgpu::Extent3d {
                    width: width as u32,
                    height: height as u32,
                    depth_or_array_layers: 0,
                },
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                view_formats: &[],
            }));

            cube_views.push(cube_texs[i].default_view());
        }

        for i in 0..2 {
            let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Cube bg"),
                layout: &cube_bg_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&cube_views[i & 2]),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&cube_views[(i + 1) % 2]),
                    },
                ],
            });

            cube_bgs.push(bg);
        }

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Terrain Generator Pipeline Layout"),
            bind_group_layouts: &[&cube_bg_layout, &uniform_bg_layout],
            push_constant_ranges: &[],
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Terrain compute pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "generate_terrain",
        });

        Self {
            compute_pipeline,
            cube_bgs,
            uniform_ctxs,
        }
    }
}
