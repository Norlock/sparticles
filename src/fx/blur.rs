use super::post_process::CreateFxOptions;
use super::post_process::FxMetaUniform;
use super::post_process::PingPongState;
use super::FxState;
use crate::model::GfxState;
use crate::model::GuiState;
use crate::traits::*;
use crate::util::CommonBuffer;
use crate::util::DynamicExport;
use crate::util::ListAction;
use crate::util::UniformContext;
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

    selected_action: ListAction,
    enabled: bool,
}

pub struct RegisterBlurFx;

// Create default is used as single fx
impl RegisterPostFx for RegisterBlurFx {
    fn tag(&self) -> &str {
        "blur"
    }

    fn create_default(&self, options: &CreateFxOptions) -> Box<dyn PostFx> {
        let settings = BlurSettings::new(FxMetaUniform::zero());

        Box::new(Blur::new(options, settings))
    }

    fn import(&self, options: &CreateFxOptions, value: serde_json::Value) -> Box<dyn PostFx> {
        let settings = serde_json::from_value(value).expect("Can't parse blur");

        Box::new(Blur::new(options, settings))
    }
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
    pub blur_uniform: BlurUniform,
    pub meta_uniform: FxMetaUniform,
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
        ping_pong: &mut PingPongState,
        fx_state: &'a FxState,
        c_pass: &mut wgpu::ComputePass<'a>,
    ) {
        let count_x = (fx_state.count_x as f32 / self.blur_uniform.downscale as f32).ceil() as u32;
        let count_y = (fx_state.count_y as f32 / self.blur_uniform.downscale as f32).ceil() as u32;

        // Splits parts to fx tex
        c_pass.set_pipeline(&self.split_pipeline);
        c_pass.set_bind_group(0, fx_state.bind_group(ping_pong), &[]);
        c_pass.set_bind_group(1, &self.bind_group, &[]);
        c_pass.dispatch_workgroups(count_x, count_y, 1);

        ping_pong.swap(&self.meta_uniform);

        // Smoothen downscaled texture
        for _ in 0..self.passes {
            c_pass.set_pipeline(&self.blur_pipeline);
            c_pass.set_bind_group(0, fx_state.bind_group(ping_pong), &[]);
            c_pass.set_bind_group(1, &self.bind_group, &[]);
            c_pass.dispatch_workgroups(count_x, count_y, 1);

            ping_pong.swap(&self.meta_uniform);
        }

        c_pass.set_pipeline(&self.upscale_pipeline);
        c_pass.set_bind_group(0, fx_state.bind_group(ping_pong), &[]);
        c_pass.set_bind_group(1, &self.bind_group, &[]);
        c_pass.dispatch_workgroups(fx_state.count_x, fx_state.count_y, 1);

        ping_pong.swap(&self.meta_uniform);
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
        ui.add(Slider::new(&mut blur.hdr_mul, 1.0..=50.0).text("HDR multiplication"));
        ui.add(Slider::new(&mut blur.radius, 2..=5).text("Blur radius"));
        ui.add(Slider::new(&mut blur.intensity, 0.9..=1.1).text("Blur intensity"));
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
    fn selected_action(&mut self) -> &mut ListAction {
        &mut self.selected_action
    }

    fn reset_action(&mut self) {
        self.selected_action = ListAction::None;
    }

    fn export(&self) -> DynamicExport {
        let settings = BlurSettings {
            meta_uniform: self.meta_uniform,
            blur_uniform: self.blur_uniform,
            passes: self.passes,
        };

        DynamicExport {
            tag: RegisterBlurFx.tag().to_string(),
            data: serde_json::to_value(settings).expect("Can't create export for blur fx"),
        }
    }

    fn enabled(&self) -> bool {
        self.enabled
    }
}

impl BlurSettings {
    pub fn new(fx_meta: FxMetaUniform) -> Self {
        Self {
            blur_uniform: BlurUniform::default(),
            meta_uniform: fx_meta,
            passes: 8,
        }
    }
}

impl Blur {
    pub fn export(&self) -> BlurSettings {
        BlurSettings {
            blur_uniform: self.blur_uniform,
            meta_uniform: self.meta_uniform,
            passes: self.passes,
        }
    }

    pub fn new(options: &CreateFxOptions, blur_settings: BlurSettings) -> Self {
        let CreateFxOptions {
            gfx_state,
            fx_state,
        } = options;

        let device = &gfx_state.device;

        let blur_uniform = blur_settings.blur_uniform;
        let passes = blur_settings.passes;
        let meta_uniform = blur_settings.meta_uniform;

        let blur_shader = device.create_shader("fx/gaussian_blur.wgsl", "Gaussian blur");

        let blur_content = CommonBuffer::uniform_content(&blur_uniform);
        let meta_content = CommonBuffer::uniform_content(&meta_uniform);

        let UniformContext {
            mut buffers,
            bind_group,
            bind_group_layout,
        } = UniformContext::new(&[&blur_content, &meta_content], device, "Gaussian blur");

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
            enabled: true,
            upscale_pipeline,
            selected_action: ListAction::None,
        }
    }
}
