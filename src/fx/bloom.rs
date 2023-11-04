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

    blur_uniform: BlurUniform,
    blur_ctx: UniformContext,

    split_pass: BlurPass,
    downscale_passes: Vec<DownscalePass>,
    upscale_passes: Vec<UpscalePass>,
    color: ColorFx,

    blend_uniform: BlendUniform,
    blend_ctx: UniformContext,
    blend: BlendPass,
}

struct DownscalePass {
    downscale: Downscale,
    blur: Option<BlurPass>,
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
    fn resize(&mut self, options: &CreateFxOptions) {
        self.blend.resize(options);
        self.color.resize(options);
        self.split_pass.resize(options);

        for dp in self.downscale_passes.iter_mut() {
            dp.downscale.resize(options);
        }

        for up in self.upscale_passes.iter_mut() {
            up.blend.resize(options);
        }
    }

    fn compute<'a>(
        &'a self,
        ping_pong: &mut PingPongState,
        fx_state: &'a FxState,
        gfx_state: &mut GfxState,
        c_pass: &mut wgpu::ComputePass<'a>,
    ) {
        let profiler = &mut gfx_state.profiler;
        let device = &gfx_state.device;

        profiler.begin_scope("Bloom Fx", c_pass, &gfx_state.device);
        profiler.begin_scope("Split", c_pass, &gfx_state.device);

        self.split_pass
            .compute_split(ping_pong, fx_state, &self.blur_ctx.bg, c_pass);

        profiler.end_scope(c_pass).unwrap();

        for (i, down) in self.downscale_passes.iter().enumerate() {
            profiler.begin_scope(&format!("Downscale {}", i), c_pass, device);
            down.downscale.compute(ping_pong, fx_state, c_pass);
            profiler.end_scope(c_pass).unwrap();

            //if let Some(blur) = &down.blur {
            //profiler.begin_scope(&format!("Blur {}", i), c_pass, device);
            //blur.compute_hor_ver(ping_pong, fx_state, &self.blur_bg, c_pass);
            //profiler.end_scope(c_pass).unwrap();
            //}
        }

        for up in self.upscale_passes.iter() {
            profiler.begin_scope("Upscale (blend)", c_pass, device);
            up.blend.lerp_blend(fx_state, &self.blend_ctx.bg, c_pass);
            profiler.end_scope(c_pass).unwrap();
        }

        profiler.begin_scope("Tonemapping", c_pass, device);
        self.color.compute_tonemap(ping_pong, fx_state, c_pass);
        profiler.end_scope(c_pass).unwrap();

        profiler.begin_scope("Blend", c_pass, device);
        self.blend.lerp_blend(fx_state, &self.blend_ctx.bg, c_pass);
        profiler.end_scope(c_pass).unwrap();

        profiler.end_scope(c_pass).unwrap();
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
        queue.write_buffer(&self.blend_ctx.buf, 0, &io_content);

        if self.update_uniform {
            let buffer_content = CommonBuffer::uniform_content(&self.blur_uniform);

            queue.write_buffer(&self.blur_ctx.buf, 0, &buffer_content);
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
    pub fn new(options: &CreateFxOptions, settings: BloomSettings) -> Self {
        let CreateFxOptions {
            gfx_state,
            fx_state,
        } = options;

        let device = &gfx_state.device;

        let blur_uniform = settings.blur_uniform;
        let blur_ctx = UniformContext::from_uniform(&blur_uniform, device, "Blur");

        let split_pass = BlurPass::new(
            options,
            BlurPassSettings {
                io_uniform: FxIOUniform::asymetric_unscaled(&options.fx_state, 0, 1),
                blur_layout: &blur_ctx.bg_layout,
            },
        );

        let mut downscale_passes = Vec::new();
        let mut upscale_passes = Vec::new();

        let downscale_list =
            FxIOUniform::create_downscale_list(&mut Vec::new(), &fx_state.tex_size, 5, 1, 1);
        let upscale_list = FxIOUniform::reverse_list(&downscale_list);

        println!("");

        let blend_uniform = settings.blend_uniform;
        let blend_ctx = UniformContext::from_uniform(&blend_uniform, device, "blend");

        for io_uniform in downscale_list {
            println!("downscale {:?}", &io_uniform);

            downscale_passes.push(DownscalePass {
                downscale: Downscale::new(options, io_uniform),
                blur: None,
            });
        }

        for io_uniform in upscale_list {
            println!("upscale {:?}", io_uniform);

            upscale_passes.push(UpscalePass {
                blend: BlendPass::new(
                    options,
                    BlendSettings {
                        io_uniform,
                        blend_layout: &blend_ctx.bg_layout,
                    },
                ),
            });
        }

        //for i in (1..=downscale_passes.len() as i32).rev() {
        //println!("blend {:?}", &blend_io);
        //let blend = BlendPass::new(
        //options,
        //BlendSettings {
        //blend_layout: &blend_ctx.bg_layout,
        //io_uniform: blend_io,
        //},
        //);

        //upscale_passes.push(UpscalePass { blend });
        //}

        let color = ColorFx::new(
            options,
            ColorFxSettings {
                io_uniform: FxIOUniform::symetric_unscaled(options.fx_state, 1),
                color_uniform: ColorFxUniform::default_srgb(),
            },
        );

        let blend = BlendPass::new(
            options,
            BlendSettings {
                io_uniform: FxIOUniform::asymetric_unscaled(options.fx_state, 1, 0),
                blend_layout: &blend_ctx.bg_layout,
            },
        );

        Self {
            split_pass,
            downscale_passes,
            upscale_passes,
            blur_ctx,
            blur_uniform,
            selected_action: ListAction::None,
            enabled: true,
            update_uniform: false,
            blend,
            blend_ctx,
            blend_uniform,
            color,
        }
    }
}
