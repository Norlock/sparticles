use super::post_process::CreateFxOptions;
use super::post_process::FxState;
use super::post_process::FxStateOptions;
use crate::traits::*;
use crate::GfxState;
use egui_wgpu::wgpu::{self, util::DeviceExt};
use egui_winit::egui::Slider;
use egui_winit::egui::Ui;
use encase::{ShaderType, UniformBuffer};
use serde::Deserialize;
use serde::Serialize;
use std::num::NonZeroU64;

pub struct Blur {
    blur_pipeline: wgpu::ComputePipeline,
    split_pipeline: wgpu::ComputePipeline,

    blur_bind_group: wgpu::BindGroup,

    pub bind_group_layout: wgpu::BindGroupLayout,
    pub blur: BlurUniform,
    pub blur_buffer: wgpu::Buffer,

    fx_state: FxState,
    passes: usize,
}

#[derive(ShaderType, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct BlurUniform {
    /// 0.10 - 0.15 is reasonable
    pub brightness_threshold: f32,

    /// Kernel size (8 default) too high or too low slows down performance
    /// Lower is more precise (pow of 2 values is better) (TODO maybe downscale attr? instead of kernel_size)
    pub kernel_size: u32,

    // How far to look
    pub radius: i32,
    pub sigma: f32,
    pub hdr_mul: f32,
    pub intensity: f32,
}

impl BlurUniform {
    pub fn new() -> Self {
        Self {
            brightness_threshold: 0.6,
            kernel_size: 16,
            radius: 4,
            sigma: 1.3,
            hdr_mul: 25.,
            intensity: 0.9, // betere naam verzinnen
        }
    }

    pub fn create_buffer_content(&self) -> Vec<u8> {
        let mut buffer = UniformBuffer::new(Vec::new());
        buffer.write(&self).unwrap();
        buffer.into_inner()
    }
}

impl PostFx for Blur {
    fn compute<'a>(
        &'a self,
        fx_inputs: Vec<&'a wgpu::BindGroup>,
        c_pass: &mut wgpu::ComputePass<'a>,
    ) {
        let output = &self.fx_state;
        let input = fx_inputs[0];

        // Splits parts to fx tex
        c_pass.set_pipeline(&self.split_pipeline);
        c_pass.set_bind_group(0, input, &[]);
        c_pass.set_bind_group(1, &output.bind_group(1), &[]);
        c_pass.set_bind_group(2, &self.blur_bind_group, &[]);
        c_pass.dispatch_workgroups(output.count_x, output.count_y, 1);

        // Smoothen downscaled texture
        for i in 0..self.passes {
            c_pass.set_pipeline(&self.blur_pipeline);
            c_pass.set_bind_group(0, input, &[]);
            c_pass.set_bind_group(1, &output.bind_group(i), &[]);
            c_pass.set_bind_group(2, &self.blur_bind_group, &[]);
            c_pass.dispatch_workgroups(output.count_x, output.count_y, 1);
        }
    }

    fn resize(&mut self, gfx_state: &GfxState) {
        let dims = Self::tex_dimensions(&gfx_state.surface_config, self.blur.kernel_size);
        self.fx_state.resize(dims, gfx_state);
    }

    fn fx_state(&self) -> &FxState {
        &self.fx_state
    }

    fn output(&self) -> &wgpu::BindGroup {
        self.fx_state.bind_group(self.passes % 2)
    }

    fn create_ui(&mut self, ui: &mut Ui, gfx_state: &GfxState) {
        let queue = &gfx_state.queue;
        let config = &gfx_state.surface_config;
        let blur = &mut self.blur;
        let mut kernel_size = blur.kernel_size;

        ui.label("Gaussian blur");
        ui.add(Slider::new(&mut blur.brightness_threshold, 0.0..=1.0).text("Brightness threshold"));
        ui.add(
            Slider::new(&mut kernel_size, 4..=32)
                .step_by(2.)
                .text("Kernel size"),
        );
        ui.add(Slider::new(&mut blur.sigma, 0.1..=8.0).text("Blur sigma"));
        ui.add(Slider::new(&mut blur.hdr_mul, 0.1..=10.0).text("HDR multiplication"));
        ui.add(Slider::new(&mut blur.radius, 2..=10).text("Blur radius"));
        ui.add(Slider::new(&mut blur.intensity, 0.1..=2.).text("Blur intensity"));
        ui.add(
            Slider::new(&mut self.passes, 2..=100)
                .step_by(2.)
                .text("Amount of passes"),
        );

        queue.write_buffer(&self.blur_buffer, 0, &blur.create_buffer_content());

        if kernel_size != blur.kernel_size {
            blur.kernel_size = kernel_size;

            self.fx_state = FxState::new(FxStateOptions {
                label: "Blur".to_string(),
                tex_dimensions: Self::tex_dimensions(config, kernel_size),
                gfx_state,
            });
        }
    }
}

impl Blur {
    fn tex_dimensions(config: &wgpu::SurfaceConfiguration, kernel_size: u32) -> [u32; 2] {
        let fx_dim = config.fx_dimensions();
        let tex_width = (fx_dim[0] as f32 / kernel_size as f32).ceil() as u32;
        let tex_height = (fx_dim[1] as f32 / kernel_size as f32).ceil() as u32;

        [tex_width, tex_height]
    }

    pub fn export(&self) -> BlurUniform {
        self.blur
    }

    pub fn import(&mut self, uniform: BlurUniform) {
        self.blur = uniform;
    }

    pub fn new(options: &CreateFxOptions, uniform: Option<BlurUniform>) -> Self {
        let CreateFxOptions {
            gfx_state,
            fx_state,
            depth_view,
        } = options;

        let device = &gfx_state.device;
        let config = &gfx_state.surface_config;

        let blur = BlurUniform::new();
        let buffer_content = blur.create_buffer_content();
        let min_binding_size = NonZeroU64::new(buffer_content.len() as u64);

        let blur_shader = device.create_shader("fx/gaussian_blur.wgsl", "Gaussian blur");

        let blur_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Blur uniform"),
            contents: &buffer_content,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let passes = 8;

        let fx_state = FxState::new(FxStateOptions {
            label: "Blur".to_string(),
            tex_dimensions: Self::tex_dimensions(config, blur.kernel_size),
            gfx_state,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Blur uniform layout"),
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
                // Depth
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        let blur_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Blur uniform bind group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: blur_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(depth_view),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Split layout"),
            bind_group_layouts: &[
                &fx_state.bind_group_layout, // input
                &fx_state.bind_group_layout, // output
                &bind_group_layout,          // globals + depth
            ],
            push_constant_ranges: &[],
        });

        let new_pipeline = |entry_point: &str| -> wgpu::ComputePipeline {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Gaussian blur pipeline"),
                layout: Some(&pipeline_layout),
                module: &blur_shader,
                entry_point,
            })
        };

        let blur_pipeline = new_pipeline("apply_blur");
        let split_pipeline = new_pipeline("split_bloom");

        Self {
            blur_pipeline,
            bind_group_layout,
            blur_bind_group,
            blur_buffer,
            blur,
            fx_state,
            split_pipeline,
            passes,
        }
    }
}
