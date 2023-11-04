use super::post_process::FxIOUniform;
use super::post_process::FxOptions;
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

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum BlurType {
    GaussianHorVer,
    Box,
    Sharpen,
}

pub struct Blur {
    blur_pipeline_x: wgpu::ComputePipeline,
    blur_pipeline_y: wgpu::ComputePipeline,

    io_uniform: FxIOUniform,
    io_ctx: UniformContext,

    blur_uniform: BlurUniform,
    blur_bg: wgpu::BindGroup,

    blur_buffer: wgpu::Buffer,
    blur_type: BlurType,
    update_uniform: bool,

    selected_action: ListAction,
    enabled: bool,
}

pub struct RegisterBlurFx;

// Create default is used as single fx
impl RegisterPostFx for RegisterBlurFx {
    fn tag(&self) -> &str {
        "blur"
    }

    fn create_default(&self, options: &FxOptions) -> Box<dyn PostFx> {
        let settings = BlurSettings {
            blur_uniform: BlurUniform::default(),
            blur_type: BlurType::GaussianHorVer,
        };

        Box::new(Blur::new(options, settings))
    }

    fn import(&self, options: &FxOptions, value: serde_json::Value) -> Box<dyn PostFx> {
        let settings = serde_json::from_value(value).expect("Can't parse blur");

        Box::new(Blur::new(options, settings))
    }
}

#[derive(ShaderType, Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct BlurUniform {
    pub brightness_threshold: f32,

    // How far to look
    pub radius: i32,
    pub sigma: f32,
    pub hdr_mul: f32,
    pub intensity: f32,
}

impl Default for BlurUniform {
    fn default() -> Self {
        Self {
            brightness_threshold: 0.6,
            radius: 4,
            sigma: 1.3,
            hdr_mul: 25.,
            intensity: 1.00, // betere naam verzinnen
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct BlurSettings {
    pub blur_uniform: BlurUniform,
    pub blur_type: BlurType,
}

impl PostFx for Blur {
    fn resize(&mut self, options: &FxOptions) {
        self.io_uniform.resize(&self.io_ctx, options);
    }

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
        fx_state: &'a FxState,
        gfx_state: &mut GfxState,
        c_pass: &mut wgpu::ComputePass<'a>,
    ) {
        let (count_x, count_y) = fx_state.count_out(&self.io_uniform);

        let mut dispatch = |pipeline: &'a wgpu::ComputePipeline| {
            c_pass.set_pipeline(pipeline);
            c_pass.set_bind_group(0, &fx_state.bg, &[]);
            c_pass.set_bind_group(1, &self.io_ctx.bg, &[]);
            c_pass.set_bind_group(2, &self.blur_bg, &[]);
            c_pass.dispatch_workgroups(count_x, count_y, 1);
        };

        match self.blur_type {
            BlurType::GaussianHorVer => {
                // TODO use pass this will become just for gui API.
                dispatch(&self.blur_pipeline_x);
                dispatch(&self.blur_pipeline_y);
            }
            _ => {}
        }
    }

    fn create_ui(&mut self, ui: &mut Ui, ui_state: &GuiState) {
        let mut blur = self.blur_uniform;

        self.selected_action = ui_state.create_li_header(ui, "Gaussian blur");

        ui.add(Slider::new(&mut blur.brightness_threshold, 0.0..=1.0).text("Brightness threshold"));
        ui.add(Slider::new(&mut blur.sigma, 0.1..=3.0).text("Blur sigma"));
        ui.add(Slider::new(&mut blur.hdr_mul, 1.0..=50.0).text("HDR multiplication"));
        ui.add(Slider::new(&mut blur.radius, 2..=5).text("Blur radius"));
        ui.add(Slider::new(&mut blur.intensity, 0.9..=1.1).text("Blur intensity"));

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
            blur_uniform: self.blur_uniform,
            blur_type: self.blur_type,
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

impl Blur {
    pub fn new(options: &FxOptions, blur_settings: BlurSettings) -> Self {
        let FxOptions {
            gfx_state,
            fx_state,
        } = options;

        let device = &gfx_state.device;

        let BlurSettings {
            blur_uniform,
            blur_type,
        } = blur_settings;

        let io_uniform = FxIOUniform::zero(options.fx_state);
        let blur_shader = device.create_shader("fx/gaussian_blur.wgsl", "Gaussian blur");

        let io_ctx = UniformContext::from_uniform(&io_uniform, device, "IO");
        let blur_ctx = UniformContext::from_uniform(&blur_uniform, device, "Blur");

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Blur layout"),
            bind_group_layouts: &[&fx_state.bg_layout, &io_ctx.bg_layout, &blur_ctx.bg_layout],
            push_constant_ranges: &[],
        });

        let new_pipeline = |entry_point: &str| -> wgpu::ComputePipeline {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Blur pipeline"),
                layout: Some(&pipeline_layout),
                module: &blur_shader,
                entry_point,
            })
        };

        let blur_pipeline_x = new_pipeline("apply_blur_x");
        let blur_pipeline_y = new_pipeline("apply_blur_y");

        Self {
            blur_pipeline_x,
            blur_pipeline_y,

            blur_buffer: blur_ctx.buf,
            blur_uniform,
            blur_bg: blur_ctx.bg,

            io_uniform,
            io_ctx,
            blur_type,
            update_uniform: false,
            enabled: true,
            selected_action: ListAction::None,
        }
    }
}
