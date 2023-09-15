use super::{post_process::FxPersistenceType, FxState};
use crate::{
    model::GfxState,
    traits::{CustomShader, PostFxChain},
};
use egui_wgpu::wgpu::{self, util::DeviceExt};
use egui_winit::egui::{self, Slider};
use encase::{ShaderType, UniformBuffer};
use std::num::NonZeroU64;

#[allow(unused)]
pub struct ColorCorrection {
    uniform: ColorCorrectionUniform,
    buffer: wgpu::Buffer,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::ComputePipeline,
    count_x: u32,
    count_y: u32,
    enabled: bool,
}

#[derive(ShaderType)]
pub struct ColorCorrectionUniform {
    pub gamma: f32,
    pub contrast: f32,
    pub brightness: f32,
}

impl PostFxChain for ColorCorrection {
    fn debug(&self) -> Option<&wgpu::BindGroup> {
        None
    }

    fn resize(&mut self, _gfx_state: &GfxState, fx_state: &FxState) {
        self.count_x = fx_state.count_x;
        self.count_y = fx_state.count_y;
    }

    fn compute<'a>(&'a self, input: &'a wgpu::BindGroup, c_pass: &mut wgpu::ComputePass<'a>) {
        c_pass.set_pipeline(&self.pipeline);
        c_pass.set_bind_group(0, input, &[]);
        c_pass.set_bind_group(1, &self.bind_group, &[]);
        c_pass.dispatch_workgroups(self.count_x, self.count_y, 1);
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn create_ui(&mut self, ui: &mut egui::Ui, gfx_state: &GfxState) {
        let uniform = &mut self.uniform;
        let queue = &gfx_state.queue;

        ui.label("Color correction");
        ui.add(Slider::new(&mut uniform.gamma, 0.1..=4.0).text("Gamma"));
        ui.add(Slider::new(&mut uniform.contrast, 0.1..=4.0).text("Contrast"));
        ui.add(Slider::new(&mut uniform.brightness, 0.01..=1.0).text("Brightness"));
        ui.checkbox(&mut self.enabled, "Enabled");

        queue.write_buffer(&self.buffer, 0, &self.uniform.create_buffer_content());
    }

    fn export(&self, to_export: &mut Vec<FxPersistenceType>) {
        //
    }
}

impl ColorCorrectionUniform {
    pub fn new() -> Self {
        Self {
            gamma: 1.0,
            contrast: 1.0,
            brightness: 0.5,
        }
    }

    fn import(&mut self, to_import: &mut Vec<FxPersistenceType>) {
        //
    }

    // TODO default trait
    pub fn create_buffer_content(&self) -> Vec<u8> {
        let mut buffer = UniformBuffer::new(Vec::new());
        buffer.write(&self).unwrap();
        buffer.into_inner()
    }
}

impl ColorCorrection {
    pub fn new(gfx_state: &GfxState, fx_state: &FxState) -> Self {
        let device = &gfx_state.device;

        let uniform = ColorCorrectionUniform::new();
        let buffer_content = uniform.create_buffer_content();
        let min_binding_size = NonZeroU64::new(buffer_content.len() as u64);

        let shader = device.create_shader("fx/color_correction.wgsl", "Color correction");

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Color correction uniform"),
            contents: &buffer_content,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Color bind group layout"),
            entries: &[
                // Globals
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Blur uniform bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Split layout"),
            bind_group_layouts: &[
                &fx_state.bind_group_layout, // input / output
                &bind_group_layout,          // globals
            ],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Gaussian blur pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "main",
        });

        Self {
            uniform,
            buffer,
            bind_group_layout,
            bind_group,
            pipeline,
            count_x: fx_state.count_x,
            count_y: fx_state.count_y,
            enabled: true,
        }
    }
}
