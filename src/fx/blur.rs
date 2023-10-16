use super::post_process::CreateFxOptions;
use super::FxState;
use crate::animations::ItemAction;
use crate::model::GuiState;
use crate::traits::*;
use crate::util::DynamicExport;
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

    passes: usize,
}

#[derive(ShaderType, Debug, Clone, Copy, Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct BlurData {
    pub uniform: BlurUniform,
    pub passes: usize,
}

impl Default for BlurData {
    fn default() -> Self {
        Self {
            uniform: BlurUniform::default(),
            passes: 8,
        }
    }
}

impl Default for BlurUniform {
    fn default() -> Self {
        Self {
            brightness_threshold: 0.6,
            kernel_size: 16,
            radius: 4,
            sigma: 1.3,
            hdr_mul: 25.,
            intensity: 1.00, // betere naam verzinnen
        }
    }
}

impl BlurUniform {
    pub fn create_buffer_content(&self) -> Vec<u8> {
        let mut buffer = UniformBuffer::new(Vec::new());
        buffer.write(&self).unwrap();
        buffer.into_inner()
    }
}

impl PostFx for Blur {
    fn compute<'a>(
        &'a self,
        ping_pong_idx: &mut usize,
        fx_state: &'a FxState,
        c_pass: &mut wgpu::ComputePass<'a>,
    ) {
        //let output = &self.fx_state;

        //// Splits parts to fx tex
        //c_pass.set_pipeline(&self.split_pipeline);
        //c_pass.set_bind_group(0, fx_inputs[0], &[]);
        //c_pass.set_bind_group(1, output.bind_group(1), &[]);
        //c_pass.set_bind_group(2, &self.blur_bind_group, &[]);
        //c_pass.dispatch_workgroups(output.count_x, output.count_y, 1);

        //// Smoothen downscaled texture
        //for i in 0..self.passes {
        //c_pass.set_pipeline(&self.blur_pipeline);
        //c_pass.set_bind_group(0, fx_inputs[0], &[]);
        //c_pass.set_bind_group(1, output.bind_group(i), &[]);
        //c_pass.set_bind_group(2, &self.blur_bind_group, &[]);
        //c_pass.dispatch_workgroups(output.count_x, output.count_y, 1);
        //}
    }

    fn reserved_space(&self) -> usize {
        1
    }

    fn create_ui(&mut self, ui: &mut Ui, ui_state: &GuiState) {
        let blur = &mut self.blur;
        let mut kernel_size = blur.kernel_size;

        ui.label("Gaussian blur");
        ui.add(Slider::new(&mut blur.brightness_threshold, 0.0..=1.0).text("Brightness threshold"));
        ui.add(
            Slider::new(&mut kernel_size, 4..=32)
                .step_by(2.)
                .text("Kernel size"),
        );
        ui.add(Slider::new(&mut blur.sigma, 0.1..=3.0).text("Blur sigma"));
        ui.add(Slider::new(&mut blur.hdr_mul, 0.1..=15.0).text("HDR multiplication"));
        ui.add(Slider::new(&mut blur.radius, 2..=5).text("Blur radius"));
        ui.add(Slider::new(&mut blur.intensity, 0.8..=1.2).text("Blur intensity"));
        ui.add(
            Slider::new(&mut self.passes, 2..=50)
                .step_by(2.)
                .text("Amount of passes"),
        );

        if kernel_size != blur.kernel_size {
            blur.kernel_size = kernel_size;
        }
    }
}

impl HandleAction for Blur {
    fn selected_action(&mut self) -> &mut ItemAction {
        todo!()
    }

    fn reset_action(&mut self) {
        todo!()
    }

    fn export(&self) -> DynamicExport {
        todo!()
    }

    fn enabled(&self) -> bool {
        todo!()
    }
}

impl Blur {
    fn tex_dimensions(config: &wgpu::SurfaceConfiguration, kernel_size: u32) -> [u32; 2] {
        let fx_dim = config.fx_dimensions();
        let tex_width = (fx_dim[0] as f32 / kernel_size as f32).ceil() as u32;
        let tex_height = (fx_dim[1] as f32 / kernel_size as f32).ceil() as u32;

        [tex_width, tex_height]
    }

    pub fn export(&self) -> BlurData {
        BlurData {
            uniform: self.blur,
            passes: self.passes,
        }
    }

    pub fn new(options: &CreateFxOptions, data: BlurData) -> Self {
        let CreateFxOptions {
            gfx_state,
            fx_state,
        } = options;

        let device = &gfx_state.device;
        let config = &gfx_state.surface_config;

        let blur = data.uniform;
        let passes = data.passes;

        let buffer_content = blur.create_buffer_content();
        let min_binding_size = NonZeroU64::new(buffer_content.len() as u64);

        let blur_shader = device.create_shader("fx/gaussian_blur.wgsl", "Gaussian blur");

        let blur_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Blur uniform"),
            contents: &buffer_content,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
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
            ],
        });

        let blur_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Blur uniform bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: blur_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Split layout"),
            bind_group_layouts: &[
                &fx_state.bind_group_layout, // input
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
            split_pipeline,
            passes,
        }
    }
}
