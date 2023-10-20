use super::post_process::CreateFxOptions;
use super::post_process::FxMetaUniform;
use super::FxState;
use crate::model::GfxState;
use crate::model::GuiState;
use crate::traits::*;
use crate::util::CommonBuffer;
use crate::util::DynamicExport;
use crate::util::ItemAction;
use crate::util::UniformCompute;
use egui_wgpu::wgpu;
use egui_winit::egui::Slider;
use egui_winit::egui::Ui;
use encase::ShaderType;
use serde::Deserialize;
use serde::Serialize;

pub struct Blur {
    blur_pipeline: wgpu::ComputePipeline,
    split_pipeline: wgpu::ComputePipeline,
    upscale_pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,

    pub bind_group_layout: wgpu::BindGroupLayout,
    pub blur_uniform: BlurUniform,
    pub blur_buffer: wgpu::Buffer,
    pub meta_uniform: FxMetaUniform,
    pub meta_buffer: wgpu::Buffer,
    pub passes: usize,
    pub update_uniform: bool,

    selected_action: ItemAction,
}

#[derive(ShaderType, Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct BlurUniform {
    pub brightness_threshold: f32,

    pub downscale: u32,

    // How far to look
    pub radius: i32,
    pub sigma: f32,
    pub hdr_mul: f32,
    pub intensity: f32,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct BlurSettings {
    pub uniform: BlurUniform,
    pub fx_meta: FxMetaUniform,
    pub passes: usize,
}

impl Default for BlurUniform {
    fn default() -> Self {
        Self {
            brightness_threshold: 0.6,
            downscale: 8,
            radius: 4,
            sigma: 1.3,
            hdr_mul: 25.,
            intensity: 1.00, // betere naam verzinnen
        }
    }
}

impl PostFx for Blur {
    fn update(&mut self, gfx_state: &GfxState) {
        if self.update_uniform {
            let queue = &gfx_state.queue;
            let buffer_content = CommonBuffer::uniform_content(&self.blur_uniform);

            queue.write_buffer(&self.blur_buffer, 0, &buffer_content);
            self.update_uniform = false;
        }
    }

    fn compute<'a>(
        &'a self,
        ping_pong_idx: &mut usize,
        fx_state: &'a FxState,
        c_pass: &mut wgpu::ComputePass<'a>,
    ) {
        let count_x = (fx_state.count_x as f32 / self.blur_uniform.downscale as f32).ceil() as u32;
        let count_y = (fx_state.count_y as f32 / self.blur_uniform.downscale as f32).ceil() as u32;

        // Splits parts to fx tex
        c_pass.set_pipeline(&self.split_pipeline);
        c_pass.set_bind_group(0, fx_state.bind_group(*ping_pong_idx), &[]);
        c_pass.set_bind_group(1, &self.bind_group, &[]);
        c_pass.dispatch_workgroups(count_x, count_y, 1);

        *ping_pong_idx += 1;

        // Smoothen downscaled texture
        for _ in 0..self.passes {
            c_pass.set_pipeline(&self.blur_pipeline);
            c_pass.set_bind_group(0, fx_state.bind_group(*ping_pong_idx), &[]);
            c_pass.set_bind_group(1, &self.bind_group, &[]);
            c_pass.dispatch_workgroups(count_x, count_y, 1);

            *ping_pong_idx += 1;
        }

        c_pass.set_pipeline(&self.upscale_pipeline);
        c_pass.set_bind_group(0, fx_state.bind_group(*ping_pong_idx), &[]);
        c_pass.set_bind_group(1, &self.bind_group, &[]);
        c_pass.dispatch_workgroups(fx_state.count_x, fx_state.count_y, 1);

        *ping_pong_idx += 1;
    }

    fn create_ui(&mut self, ui: &mut Ui, _: &GuiState) {
        let mut blur = self.blur_uniform;

        ui.label("Gaussian blur");
        ui.add(Slider::new(&mut blur.brightness_threshold, 0.0..=1.0).text("Brightness threshold"));
        ui.add(
            Slider::new(&mut blur.downscale, 4..=32)
                .step_by(2.)
                .text("Downscale"),
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

        if self.blur_uniform != blur {
            self.blur_uniform = blur;
            self.update_uniform = true;
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

impl BlurSettings {
    pub fn new(fx_meta: FxMetaUniform) -> Self {
        Self {
            uniform: BlurUniform::default(),
            fx_meta,
            passes: 8,
        }
    }
}

impl Blur {
    pub fn export(&self) -> BlurSettings {
        BlurSettings {
            uniform: self.blur_uniform,
            fx_meta: self.meta_uniform,
            passes: self.passes,
        }
    }

    pub fn new(options: &CreateFxOptions, blur_settings: BlurSettings) -> Self {
        let CreateFxOptions {
            gfx_state,
            fx_state,
        } = options;

        let device = &gfx_state.device;

        let blur_uniform = blur_settings.uniform;
        let passes = blur_settings.passes;
        let meta_uniform = blur_settings.fx_meta;

        let blur_shader = device.create_shader("fx/gaussian_blur.wgsl", "Gaussian blur");

        let blur_content = CommonBuffer::uniform_content(&blur_uniform);
        let meta_content = CommonBuffer::uniform_content(&meta_uniform);

        let UniformCompute {
            mut buffers,
            bind_group,
            bind_group_layout,
        } = UniformCompute::new(&[&blur_content, &meta_content], device, "Gaussian blur");

        let meta_buffer = buffers.pop().unwrap();
        let blur_buffer = buffers.pop().unwrap();

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Split layout"),
            bind_group_layouts: &[
                &fx_state.bind_group_layout, // input
                &bind_group_layout,          // blur + meta
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
        let upscale_pipeline = new_pipeline("upscale");

        Self {
            blur_pipeline,
            bind_group_layout,
            bind_group,
            blur_buffer,
            blur_uniform,
            split_pipeline,
            meta_buffer,
            meta_uniform,
            passes,
            update_uniform: false,
            upscale_pipeline,
            selected_action: ItemAction::None,
        }
    }
}
