use egui_wgpu::wgpu::{self, util::DeviceExt};
use encase::ShaderType;

use crate::{
    model::{Camera, GfxState, SparState},
    shaders::{ShaderOptions, SDR_TONEMAPPING},
    traits::BufferContent,
};

#[derive(ShaderType, Debug)]
pub struct TerrainUniform {
    pub noise: f32,
    pub group_size: f32,
}

pub struct TerrainGenerator {
    pub compute_pipeline: wgpu::ComputePipeline,
    pub cube_bgs: Vec<wgpu::BindGroup>,
    pub cube_bg_layout: wgpu::BindGroupLayout,
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
const CUBE_SIZE: f32 = 2048.;

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

    pub fn cube_bg(&self) -> &wgpu::BindGroup {
        &self.cube_bgs[self.cube_bgs.len() % 2]
    }

    pub fn compute(state: &SparState, encoder: &mut wgpu::CommandEncoder) {
        let tg = &state.terrain_generator;
        //let camera = &state.camera;

        let mut c_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Terrain compute pass"),
            timestamp_writes: None,
        });

        let mut i = 0;

        for uniform_ctx in tg.uniform_ctxs.iter() {
            c_pass.set_pipeline(&tg.compute_pipeline);
            c_pass.set_bind_group(0, &tg.cube_bgs[i % 2], &[]);
            c_pass.set_bind_group(1, &uniform_ctx.bg, &[]);
            c_pass.dispatch_workgroups(uniform_ctx.count_x, uniform_ctx.count_y, 6);

            i += 1;
        }
    }

    pub fn create_group_sizes(
        gfx: &GfxState,
        bg_layout: &wgpu::BindGroupLayout,
    ) -> Vec<TerrainUniformCtx> {
        let device = &gfx.device;

        let mut ctxs = Vec::new();
        let mut tex_size = 128.;

        while tex_size <= CUBE_SIZE || tex_size <= CUBE_SIZE {
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

        let mut cube_texs = vec![];
        let mut cube_bgs = vec![];

        let cube_bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Cube bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        view_dimension: wgpu::TextureViewDimension::D2Array,
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
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        for i in 0..2 {
            let size = wgpu::Extent3d {
                width: CUBE_SIZE as u32,
                height: CUBE_SIZE as u32,
                depth_or_array_layers: 6,
            };

            cube_texs.push(device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Terrain cube"),
                size,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::STORAGE_BINDING
                    | wgpu::TextureUsages::COPY_DST,
                mip_level_count: 1, // Maybe do this
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                view_formats: &[],
            }));

            gfx.queue.write_texture(
                cube_texs[i].as_image_copy(),
                &[255, 255, 255, 255],
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4),
                    rows_per_image: None,
                },
                wgpu::Extent3d::default(),
            );
        }

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            border_color: Some(wgpu::SamplerBorderColor::OpaqueWhite),
            ..Default::default()
        });

        for i in 0..2 {
            let layers_view = cube_texs[(i + 1) % 2].create_view(&wgpu::TextureViewDescriptor {
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                array_layer_count: Some(6),
                ..Default::default()
            });

            let cube_view = cube_texs[i % 2].create_view(&wgpu::TextureViewDescriptor {
                dimension: Some(wgpu::TextureViewDimension::Cube),
                array_layer_count: Some(6),
                ..Default::default()
            });

            let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Cube bg"),
                layout: &cube_bg_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&layers_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&cube_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&sampler),
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
            cube_bg_layout,
        }
    }
}
