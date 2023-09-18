use super::{
    bloom::BloomExport,
    blur::{BlurExport, BlurUniform},
    color_processing::ColorProcessingUniform,
    Bloom, ColorProcessing,
};
use crate::model::GfxState;
use crate::traits::*;
use egui_wgpu::wgpu::{self, util::DeviceExt};
use encase::{ShaderType, UniformBuffer};
use serde::{Deserialize, Serialize};
use std::{num::NonZeroU64, rc::Rc};

pub struct PostProcessState {
    pub frame_state: FrameState,
    pub post_fx: Vec<Box<dyn PostFxChain>>,
    pub fx_state: FxState,
    pub selected_view: String,
    pub views: Vec<FxView>,

    frame_group_layout: wgpu::BindGroupLayout,
    initialize_pipeline: wgpu::ComputePipeline,
    finalize_pipeline: wgpu::RenderPipeline,
    uniform: OffsetUniform,
    offset_buffer: wgpu::Buffer,
}

pub struct FrameState {
    pub depth_view: wgpu::TextureView,
    pub frame_view: wgpu::TextureView,
    bind_group: Rc<wgpu::BindGroup>,
}

#[derive(ShaderType, Clone)]
pub struct OffsetUniform {
    offset: i32,
    view_width: f32,
    view_height: f32,
}

pub struct FxView {
    pub tag: String,
    pub bind_group: Rc<wgpu::BindGroup>,
}

impl PartialEq for FxView {
    fn eq(&self, other: &Self) -> bool {
        self.tag == other.tag
    }

    fn ne(&self, other: &Self) -> bool {
        self.tag != other.tag
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum FxPersistenceType {
    Bloom(BloomExport),
    ColorProcessing(ColorProcessingUniform),
}

pub struct CreateFxOptions<'a> {
    pub gfx_state: &'a GfxState,
    pub fx_state: &'a FxState,
    pub depth_view: &'a wgpu::TextureView,
}

impl OffsetUniform {
    fn new(config: &wgpu::SurfaceConfiguration) -> Self {
        Self {
            offset: config.fx_offset() as i32,
            view_width: config.width as f32,
            view_height: config.height as f32,
        }
    }

    fn buffer_content(&self) -> Vec<u8> {
        let mut buffer = UniformBuffer::new(Vec::new());
        buffer.write(&self).unwrap();
        buffer.into_inner()
    }
}

pub const WORK_GROUP_SIZE: [f32; 2] = [8., 8.];

impl PostProcessState {
    pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

    pub fn output(&self) -> &Rc<wgpu::BindGroup> {
        let nr = self.post_fx.iter().filter(|fx| fx.enabled()).count();

        self.fx_state.bind_group(nr)
    }

    pub fn default_view(&self) -> FxView {
        FxView {
            tag: "Default".to_string(),
            bind_group: self.output().clone(),
        }
    }

    pub fn recreate_views(&mut self) {
        self.views.clear();
        self.views.push(self.default_view());

        for (idx, fx) in self.post_fx.iter().enumerate() {
            fx.add_views(&mut self.views, idx);
        }
    }

    pub fn resize(&mut self, gfx_state: &GfxState) {
        let config = &gfx_state.surface_config;
        let queue = &gfx_state.queue;

        self.uniform = OffsetUniform::new(config);
        queue.write_buffer(&self.offset_buffer, 0, &self.uniform.buffer_content());

        self.frame_state =
            FrameState::new(gfx_state, &self.frame_group_layout, &self.offset_buffer);
        self.fx_state.resize(config.fx_dimensions(), gfx_state);

        for pfx in self.post_fx.iter_mut() {
            pfx.resize(&gfx_state, &self.fx_state);
        }

        self.recreate_views();
    }

    pub fn compute(&self, encoder: &mut wgpu::CommandEncoder) {
        let input = self.fx_state.bind_group(1);

        let mut c_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Post process pipeline"),
        });

        c_pass.set_pipeline(&self.initialize_pipeline);
        c_pass.set_bind_group(0, &input, &[]);
        c_pass.set_bind_group(1, &self.frame_state.bind_group, &[]);
        c_pass.dispatch_workgroups(self.fx_state.count_x, self.fx_state.count_y, 1);

        for (i, pfx) in self.post_fx.iter().filter(|fx| fx.enabled()).enumerate() {
            let input = self.fx_state.bind_group(i);
            pfx.compute(input, &mut c_pass);
        }
    }

    pub fn render<'a>(&'a self, r_pass: &mut wgpu::RenderPass<'a>) {
        let fx_bind_group = self
            .views
            .iter()
            .find(|vw| vw.tag == self.selected_view)
            .map_or(self.output(), |vw| &vw.bind_group);

        r_pass.set_pipeline(&self.finalize_pipeline);
        r_pass.set_bind_group(0, fx_bind_group, &[]);
        r_pass.set_bind_group(1, &self.frame_state.output(), &[]);
        r_pass.draw(0..3, 0..1);
    }

    pub fn create_fx_layout(
        device: &wgpu::Device,
        offset: &OffsetUniform,
    ) -> wgpu::BindGroupLayout {
        let min_binding_size = NonZeroU64::new(offset.buffer_content().len() as u64);

        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Frame layout"),
            entries: &[
                // Frame
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        multisampled: false,
                    },
                    count: None,
                },
                // Offset uniform
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size,
                    },
                    count: None,
                },
            ],
        })
    }

    pub fn new(gfx_state: &GfxState) -> Self {
        let device = &gfx_state.device;
        let config = &gfx_state.surface_config;

        let initialize_shader = device.create_shader("fx/initialize.wgsl", "Init post fx");
        let finalize_shader = device.create_shader("fx/finalize.wgsl", "Finalize post fx");

        let uniform = OffsetUniform::new(config);
        let buffer_content = uniform.buffer_content();

        let offset_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Offset"),
            contents: &buffer_content,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let frame_group_layout = Self::create_fx_layout(&device, &uniform);
        let frame_state = FrameState::new(gfx_state, &frame_group_layout, &offset_buffer);

        let fx_state = FxState::new(FxStateOptions {
            label: "Post process start".to_string(),
            tex_dimensions: config.fx_dimensions(),
            gfx_state,
        });

        let fx_group_layout = &fx_state.bind_group_layout;

        let compute_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Init layout"),
            bind_group_layouts: &[&fx_group_layout, &frame_group_layout],
            push_constant_ranges: &[],
        });

        let initialize_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Init pipeline"),
                layout: Some(&compute_layout),
                module: &initialize_shader,
                entry_point: "init",
            });

        let render_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Post fx render"),
            bind_group_layouts: &[&fx_group_layout, &frame_group_layout],
            push_constant_ranges: &[],
        });

        let finalize_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Finalize pipeline"),
            layout: Some(&render_layout),
            vertex: wgpu::VertexState {
                module: &finalize_shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &finalize_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        Self {
            frame_state,
            fx_state,
            post_fx: vec![],
            frame_group_layout,
            initialize_pipeline,
            finalize_pipeline,
            offset_buffer,
            uniform,
            views: Vec::new(),
            selected_view: "Default".to_string(),
        }
    }

    pub fn create_fx_options<'a>(&'a self, gfx_state: &'a GfxState) -> CreateFxOptions {
        CreateFxOptions {
            gfx_state,
            fx_state: &self.fx_state,
            depth_view: &self.frame_state.depth_view,
        }
    }

    pub fn add_default_fx(&mut self, gfx_state: &GfxState) {
        let options = self.create_fx_options(gfx_state);
        let blur_export = BlurExport {
            uniform: BlurUniform::new(),
            passes: 8,
        };

        let bloom = Bloom::new(&options, BloomExport { blur: blur_export });
        let col_cor = ColorProcessing::new(&options, ColorProcessingUniform::new());

        self.post_fx.push(Box::new(bloom));
        self.post_fx.push(Box::new(col_cor));

        self.recreate_views();
    }

    pub fn import_fx(&mut self, gfx_state: &GfxState, fx_types: Vec<FxPersistenceType>) {
        for item in fx_types.into_iter() {
            match item {
                FxPersistenceType::Bloom(export) => {
                    let options = self.create_fx_options(gfx_state);
                    let bloom = Bloom::new(&options, export);

                    self.post_fx.push(Box::new(bloom));
                }
                FxPersistenceType::ColorProcessing(export) => {
                    let options = self.create_fx_options(gfx_state);
                    let fx = ColorProcessing::new(&options, export);

                    self.post_fx.push(Box::new(fx));
                }
            }
        }

        self.recreate_views();
    }
}

impl FrameState {
    pub fn new(
        gfx_state: &GfxState,
        bind_group_layout: &wgpu::BindGroupLayout,
        buffer: &wgpu::Buffer,
    ) -> Self {
        let device = &gfx_state.device;

        let frame_view = gfx_state.create_frame_view();
        let depth_view = gfx_state.create_depth_view();

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&frame_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            frame_view,
            depth_view,
            bind_group: bind_group.into(),
        }
    }

    pub fn output(&self) -> &Rc<wgpu::BindGroup> {
        &self.bind_group
    }
}

pub struct FxState {
    bind_groups: Vec<Rc<wgpu::BindGroup>>,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub count_x: u32,
    pub count_y: u32,
    pub label: String,
}

pub struct FxStateOptions<'a> {
    /// For debugging purposes
    pub label: String,
    pub tex_dimensions: Dimensions,
    pub gfx_state: &'a GfxState,
}

pub type Dimensions = [u32; 2];

impl FxState {
    pub fn bind_group(&self, idx: usize) -> &Rc<wgpu::BindGroup> {
        &self.bind_groups[idx % 2]
    }

    fn create_bind_groups(
        dimensions: Dimensions,
        layout: &wgpu::BindGroupLayout,
        gfx_state: &GfxState,
    ) -> Vec<Rc<wgpu::BindGroup>> {
        let device = &gfx_state.device;

        let mut bind_groups = Vec::new();
        let fx_view_1 = gfx_state.create_fx_view(dimensions);
        let fx_view_2 = gfx_state.create_fx_view(dimensions);
        let fx_views = vec![fx_view_1, fx_view_2];

        for i in 0..2 {
            let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&fx_views[i % 2]),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&fx_views[(i + 1) % 2]),
                    },
                ],
            });

            bind_groups.push(Rc::new(bg));
        }

        return bind_groups;
    }

    fn get_dispatch_counts(dimensions: Dimensions) -> [u32; 2] {
        let count_x = (dimensions[0] as f32 / WORK_GROUP_SIZE[0]).ceil() as u32;
        let count_y = (dimensions[1] as f32 / WORK_GROUP_SIZE[1]).ceil() as u32;

        return [count_x, count_y];
    }

    pub fn resize(&mut self, dimensions: Dimensions, gfx_state: &GfxState) {
        let counts = Self::get_dispatch_counts(dimensions);

        self.bind_groups = Self::create_bind_groups(dimensions, &self.bind_group_layout, gfx_state);
        self.count_x = counts[0];
        self.count_y = counts[1];
    }

    pub fn new(options: FxStateOptions) -> Self {
        let FxStateOptions {
            label,
            tex_dimensions,
            gfx_state,
        } = options;

        let device = &gfx_state.device;

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bloom textures layout"),
            entries: &[
                // FX Write
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        view_dimension: wgpu::TextureViewDimension::D2,
                        format: PostProcessState::TEXTURE_FORMAT,
                        access: wgpu::StorageTextureAccess::WriteOnly,
                    },
                    count: None,
                },
                // FX Read
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        let bind_groups = Self::create_bind_groups(tex_dimensions, &bind_group_layout, gfx_state);
        let counts = Self::get_dispatch_counts(tex_dimensions);

        Self {
            label,
            bind_groups,
            bind_group_layout,
            count_x: counts[0],
            count_y: counts[1],
        }
    }
}

impl FxDimensions for wgpu::SurfaceConfiguration {
    fn fx_dimensions(&self) -> Dimensions {
        let expand = self.fx_offset() * 2;

        [self.width + expand, self.height + expand]
    }

    fn fx_offset(&self) -> u32 {
        0 //(self.width / 60).max(32)
    }
}
