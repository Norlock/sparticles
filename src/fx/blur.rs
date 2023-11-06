use super::blur_pass::BlurPass;
use super::blur_pass::BlurPassSettings;
use super::FxOptions;
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

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub enum BlurType {
    Gaussian,
    Box,
    Sharpen,
}

pub struct Blur {
    blur_uniform: BlurUniform,
    blur_ctx: UniformContext,
    blur_type: BlurType,
    blur_pass: BlurPass,

    update_uniform: Option<bool>,

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
            blur_type: BlurType::Gaussian,
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
        self.blur_pass.resize(options);
    }

    fn update(&mut self, gfx_state: &GfxState) {
        if let Some(_) = self.update_uniform.take() {
            let queue = &gfx_state.queue;

            let buffer_content = CommonBuffer::uniform_content(&self.blur_uniform);
            queue.write_buffer(&self.blur_ctx.buf, 0, &buffer_content);
        }
    }

    fn compute<'a>(
        &'a self,
        fx_state: &'a FxState,
        gfx_state: &mut GfxState,
        c_pass: &mut wgpu::ComputePass<'a>,
    ) {
        let bp = &self.blur_pass;

        match self.blur_type {
            BlurType::Gaussian => {
                bp.compute_gaussian(fx_state, gfx_state, &self.blur_ctx.bg, c_pass);
            }
            _ => {}
        }
    }

    fn create_ui(&mut self, ui: &mut Ui, ui_state: &GuiState) {
        let mut blur = self.blur_uniform;

        self.selected_action = ui_state.create_li_header(ui, "Gaussian blur");

        if self.blur_type == BlurType::Gaussian {
            ui.add(Slider::new(&mut blur.sigma, 0.1..=3.0).text("Blur sigma"));
            ui.add(Slider::new(&mut blur.radius, 2..=6).text("Blur radius"));
            ui.add(Slider::new(&mut blur.intensity, 0.9..=1.1).text("Blur intensity"));
        } else {
            ui.add(
                Slider::new(&mut blur.brightness_threshold, 0.0..=1.0).text("Brightness threshold"),
            );
            ui.add(Slider::new(&mut blur.sigma, 0.1..=3.0).text("Blur sigma"));
            ui.add(Slider::new(&mut blur.hdr_mul, 1.0..=50.0).text("HDR multiplication"));
            ui.add(Slider::new(&mut blur.radius, 2..=5).text("Blur radius"));
            ui.add(Slider::new(&mut blur.intensity, 0.9..=1.1).text("Blur intensity"));
        }

        ui.checkbox(&mut self.enabled, "Enabled");

        if self.blur_uniform != blur {
            self.blur_uniform = blur;
            self.update_uniform = Some(true);
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
        let FxOptions { gfx_state, .. } = options;

        let device = &gfx_state.device;

        let BlurSettings {
            blur_uniform,
            blur_type,
        } = blur_settings;

        let blur_ctx = UniformContext::from_uniform(&blur_uniform, device, "Blur");

        let blur_pass = BlurPass::new(
            options,
            BlurPassSettings {
                blur_layout: &blur_ctx.bg_layout,
                io_idx: (0, 1),
                downscale: 1.,
            },
        );

        Self {
            blur_ctx,
            blur_uniform,
            blur_type,
            blur_pass,

            update_uniform: None,
            enabled: true,
            selected_action: ListAction::None,
        }
    }
}
