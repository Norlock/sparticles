use super::blur::BlurUniform;
use super::blur_pass::BlurPass;
use super::blur_pass::BlurPassSettings;
use super::post_process::CreateFxOptions;
use super::post_process::FxIOUniform;
use super::post_process::PingPongState;
use super::BlendPass;
use super::ColorFx;
use super::Downscale;
use super::FxState;
use crate::fx::blend::BlendSettings;
use crate::fx::blend::BlendUniform;
use crate::fx::ColorFxSettings;
use crate::fx::ColorFxUniform;
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
use serde::Deserialize;
use serde::Serialize;

pub struct Bloom {
    enabled: bool,
    update_uniform: bool,
    selected_action: ListAction,

    blur_bg: wgpu::BindGroup,
    blur_uniform: BlurUniform,
    blur_buf: wgpu::Buffer,

    split_pass: BlurPass,
    downscale_passes: Vec<DownscalePass>,
    upscale_passes: Vec<UpscalePass>,
    color: ColorFx,

    blend_uniform: BlendUniform,
    blend_buf: wgpu::Buffer,
    blend_bg: wgpu::BindGroup,
    blend: BlendPass,
}

struct DownscalePass {
    downscale: Downscale,
    blur: BlurPass,
}

struct UpscalePass {
    blend: BlendPass,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BloomSettings {
    pub blur_uniform: BlurUniform,
    pub blend_uniform: BlendUniform,
}

pub struct RegisterBloomFx;

impl RegisterPostFx for RegisterBloomFx {
    fn tag(&self) -> &str {
        "bloom"
    }

    fn import(&self, options: &CreateFxOptions, value: serde_json::Value) -> Box<dyn PostFx> {
        let bloom_settings = serde_json::from_value(value).unwrap();
        Box::new(Bloom::new(options, bloom_settings))
    }

    fn create_default(&self, options: &CreateFxOptions) -> Box<dyn PostFx> {
        Box::new(Bloom::new(
            options,
            BloomSettings {
                blur_uniform: BlurUniform::default(),
                blend_uniform: BlendUniform { io_mix: 0.5 },
            },
        ))
    }
}

impl PostFx for Bloom {
    fn compute<'a>(
        &'a self,
        ping_pong: &mut PingPongState,
        fx_state: &'a FxState,
        c_pass: &mut wgpu::ComputePass<'a>,
    ) {
        self.split_pass
            .compute_split(ping_pong, fx_state, &self.blur_bg, c_pass);

        for down in self.downscale_passes.iter() {
            down.downscale.compute(ping_pong, fx_state, c_pass);
            down.blur
                .compute_hor_ver(ping_pong, fx_state, &self.blur_bg, c_pass);
        }

        for up in self.upscale_passes.iter() {
            up.blend
                .compute_blend(ping_pong, fx_state, &self.blend_bg, c_pass);
        }

        self.color.compute_tonemap(ping_pong, fx_state, c_pass);
        self.blend
            .compute_blend(ping_pong, fx_state, &self.blend_bg, c_pass);
    }

    fn create_ui(&mut self, ui: &mut Ui, ui_state: &GuiState) {
        self.selected_action = ui_state.create_li_header(ui, "Bloom settings");
        ui.add_space(5.0);

        let mut blur = self.blur_uniform;

        GuiState::create_title(ui, "Gaussian blur");

        ui.add(Slider::new(&mut blur.brightness_threshold, 0.0..=1.0).text("Brightness threshold"));
        ui.add(Slider::new(&mut blur.sigma, 0.1..=3.0).text("Blur sigma"));
        ui.add(Slider::new(&mut blur.hdr_mul, 1.0..=50.0).text("HDR multiplication"));
        ui.add(Slider::new(&mut blur.radius, 2..=6).text("Blur radius"));
        ui.add(Slider::new(&mut blur.intensity, 0.9..=1.1).text("Blur intensity"));

        GuiState::create_title(ui, "Blend");
        ui.add(Slider::new(&mut self.blend_uniform.io_mix, 0.0..=1.0).text("IO mix"));

        ui.checkbox(&mut self.enabled, "Enabled");

        if self.blur_uniform != blur {
            self.blur_uniform = blur;
            self.update_uniform = true;
        }
    }

    fn update(&mut self, gfx_state: &GfxState) {
        let queue = &gfx_state.queue;
        let io_content = CommonBuffer::uniform_content(&self.blend_uniform);
        queue.write_buffer(&self.blend_buf, 0, &io_content);

        if self.update_uniform {
            let buffer_content = CommonBuffer::uniform_content(&self.blur_uniform);

            queue.write_buffer(&self.blur_buf, 0, &buffer_content);
            self.update_uniform = false;
        }
    }
}

impl HandleAction for Bloom {
    fn selected_action(&mut self) -> &mut ListAction {
        &mut self.selected_action
    }

    fn reset_action(&mut self) {
        self.selected_action = ListAction::None
    }

    fn export(&self) -> DynamicExport {
        let bloom_settings = BloomSettings {
            blur_uniform: self.blur_uniform,
            blend_uniform: self.blend_uniform,
        };

        DynamicExport {
            tag: RegisterBloomFx.tag().to_string(),
            data: serde_json::to_value(bloom_settings).unwrap(),
        }
    }

    fn enabled(&self) -> bool {
        self.enabled
    }
}

impl Bloom {
    pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

    pub fn new(options: &CreateFxOptions, settings: BloomSettings) -> Self {
        let CreateFxOptions { gfx_state, .. } = options;

        let device = &gfx_state.device;

        let blur_uniform = settings.blur_uniform;
        let blur_ctx = UniformContext::from_uniform(&blur_uniform, device, "Blur");

        let split_pass = BlurPass::new(
            options,
            BlurPassSettings {
                io_uniform: FxIOUniform::asymetric_unscaled(0, 1),
                blur_layout: &blur_ctx.bg_layout,
            },
        );

        let mut width = gfx_state.surface_config.width;
        let mut height = gfx_state.surface_config.height;

        let mut add_downscale_pass = true;
        let mut idx = 1;
        let mut downscale = 1.0;

        let mut downscale_passes = Vec::new();
        let mut upscale_passes = Vec::new();

        while add_downscale_pass {
            let in_idx = idx;
            let out_idx = idx + 1;
            let out_downscale = downscale * 2.;

            let downscale_io = FxIOUniform {
                in_idx,
                in_downscale: downscale,
                out_idx,
                out_downscale,
            };

            downscale = out_downscale;

            println!("downscale: {:?}", &downscale_io);
            let downscale = Downscale::new(options, downscale_io);

            let blur_io = FxIOUniform::symetric_downscaled(out_idx, out_downscale);
            println!("blur: {:?}", &blur_io);

            let blur = BlurPass::new(
                options,
                BlurPassSettings {
                    io_uniform: blur_io,
                    blur_layout: &blur_ctx.bg_layout,
                },
            );

            downscale_passes.push(DownscalePass { downscale, blur });

            width = (width as f32 / 2.).ceil() as u32;
            height = (height as f32 / 2.).ceil() as u32;

            idx += 1;
            add_downscale_pass = 10 < width && 10 < height;
        }

        println!("");

        let blend_uniform = settings.blend_uniform;
        let blend_ctx = UniformContext::from_uniform(&blend_uniform, device, "blend");

        for i in (1..=downscale_passes.len()).rev() {
            let blend_io = FxIOUniform {
                in_idx: (i + 1) as u32,
                in_downscale: 2f32.powi(i as i32),
                out_idx: i as u32,
                out_downscale: 2f32.powi(i as i32 - 1),
            };

            println!("blend {:?}", &blend_io);
            let blend = BlendPass::new(
                options,
                BlendSettings {
                    blend_layout: &blend_ctx.bg_layout,
                    io_uniform: blend_io,
                },
            );

            upscale_passes.push(UpscalePass { blend });
        }

        let color = ColorFx::new(
            options,
            ColorFxSettings {
                io_uniform: FxIOUniform::symetric_unscaled(1),
                color_uniform: ColorFxUniform::default_srgb(),
            },
        );

        let blend = BlendPass::new(
            options,
            BlendSettings {
                io_uniform: FxIOUniform::asymetric_unscaled(1, 0),
                blend_layout: &blend_ctx.bg_layout,
            },
        );

        Self {
            split_pass,
            downscale_passes,
            upscale_passes,
            blur_bg: blur_ctx.bg,
            blur_buf: blur_ctx.buf,
            blur_uniform,
            selected_action: ListAction::None,
            enabled: true,
            update_uniform: false,
            blend,
            blend_buf: blend_ctx.buf,
            blend_bg: blend_ctx.bg,
            blend_uniform,
            color,
        }
    }
}
