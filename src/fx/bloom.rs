use super::BlendPass;
use super::ColorFx;
use super::Downscale;
use super::FxIOUniform;
use super::FxOptions;
use super::FxState;
use crate::fx::blend::BlendSettings;
use crate::fx::blend::BlendUniform;
use crate::fx::ColorFxSettings;
use crate::fx::ColorFxUniform;
use crate::model::Camera;
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

enum UIAction {
    UpdateBuffer(usize),
}

pub struct Bloom {
    enabled: bool,
    update_event: Option<UIAction>,
    selected_action: ListAction,

    downscale_passes: Vec<DownscalePass>,
    upscale_passes: Vec<UpscalePass>,
    color: ColorFx,

    blend_uniform: BlendUniform,
    blend_ctx: UniformContext,
    blend: BlendPass,

    bloom_treshold: f32,
}

struct DownscalePass {
    downscale: Downscale,
}

struct UpscalePass {
    blend: BlendPass,
    blend_uniform: BlendUniform,
    blend_ctx: UniformContext,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BloomSettings {
    final_blend: BlendUniform,
    upscale_blends: Vec<BlendUniform>,
    color: ColorFxUniform,
    bloom_treshold: f32,
}

pub struct RegisterBloomFx;

impl RegisterPostFx for RegisterBloomFx {
    fn tag(&self) -> &'static str {
        "bloom"
    }

    fn import(&self, options: &FxOptions, value: serde_json::Value) -> Box<dyn PostFx> {
        let bloom_settings = serde_json::from_value(value).unwrap();
        Box::new(Bloom::new(options, bloom_settings))
    }

    fn create_default(&self, options: &FxOptions) -> Box<dyn PostFx> {
        let mut upscale_blends = Vec::new();

        for _ in 0..5 {
            upscale_blends.push(BlendUniform { io_mix: 0.5 });
        }

        Box::new(Bloom::new(
            options,
            BloomSettings {
                color: ColorFxUniform::default_srgb(),
                final_blend: BlendUniform { io_mix: 0.5 },
                bloom_treshold: 1.0,
                upscale_blends,
            },
        ))
    }
}

impl PostFx for Bloom {
    fn resize(&mut self, options: &FxOptions) {
        self.blend.resize(options);
        self.color.resize(options);

        for dp in self.downscale_passes.iter_mut() {
            dp.downscale.resize(options);
        }

        for up in self.upscale_passes.iter_mut() {
            up.blend.resize(options);
        }
    }

    fn compute<'a>(
        &'a self,
        fx_state: &'a FxState,
        gfx_state: &mut GfxState,
        c_pass: &mut wgpu::ComputePass<'a>,
    ) {
        gfx_state.begin_scope("Bloom Fx", c_pass);

        for down in self.downscale_passes.iter() {
            down.downscale.compute(fx_state, gfx_state, c_pass);
        }

        for up in self.upscale_passes.iter() {
            up.blend
                .lerp_upscale(fx_state, gfx_state, &up.blend_ctx.bg, c_pass);
        }

        self.color.compute_tonemap(fx_state, gfx_state, c_pass);

        self.blend
            .lerp_upscale(fx_state, gfx_state, &self.blend_ctx.bg, c_pass);

        gfx_state.end_scope(c_pass);
    }

    fn create_ui(&mut self, ui: &mut Ui, ui_state: &GuiState) {
        self.selected_action = ui_state.create_li_header(ui, "Bloom settings");
        ui.add_space(5.0);

        ui.add(Slider::new(&mut self.bloom_treshold, 0.0..=10.0).text("Brightness treshold"));

        for (i, up) in self.upscale_passes.iter_mut().enumerate() {
            let io_uniform = up.blend.io();
            let text = format!(
                "IO mix from downscale {} to {}",
                io_uniform.in_downscale, io_uniform.out_downscale
            );

            if ui
                .add(Slider::new(&mut up.blend_uniform.io_mix, 0.0..=1.0).text(&text))
                .changed()
            {
                self.update_event = Some(UIAction::UpdateBuffer(i));
            }
        }

        GuiState::create_title(ui, "Blend");
        if ui
            .add(
                Slider::new(&mut self.blend_uniform.io_mix, 0.0..=1.0)
                    .text("IO mix bloom to frame"),
            )
            .changed()
        {
            self.update_event = Some(UIAction::UpdateBuffer(self.upscale_passes.len()));
        }

        GuiState::create_title(ui, "Color correction");
        self.color.ui_gamma(ui);

        ui.checkbox(&mut self.enabled, "Enabled");
    }

    fn update(&mut self, gfx_state: &GfxState, camera: &mut Camera) {
        camera.bloom_treshold = glam::Vec3::splat(self.bloom_treshold);

        match self.update_event.take() {
            Some(UIAction::UpdateBuffer(i)) => {
                let queue = &gfx_state.queue;

                if let Some(up) = self.upscale_passes.get_mut(i) {
                    let io_content = CommonBuffer::uniform_content(&up.blend_uniform);
                    queue.write_buffer(&up.blend_ctx.buf, 0, &io_content);
                } else {
                    let io_content = CommonBuffer::uniform_content(&self.blend_uniform);
                    queue.write_buffer(&self.blend_ctx.buf, 0, &io_content);
                }
            }
            None => {}
        };

        self.color.update(gfx_state, camera);
    }
}

impl HandleAction for Bloom {
    fn selected_action(&mut self) -> &mut ListAction {
        &mut self.selected_action
    }

    fn export(&self) -> DynamicExport {
        let bloom_settings = BloomSettings {
            color: self.color.color_uniform,
            final_blend: self.blend_uniform,
            bloom_treshold: self.bloom_treshold,
            upscale_blends: self
                .upscale_passes
                .iter()
                .map(|up| up.blend_uniform)
                .collect(),
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
    pub fn new(options: &FxOptions, settings: BloomSettings) -> Self {
        let FxOptions {
            gfx_state,
            fx_state,
        } = options;

        let device = &gfx_state.device;

        let mut downscale_passes = Vec::new();
        let mut upscale_passes = Vec::new();

        let downscale_count = settings.upscale_blends.len() as i32;
        let downscale_list = FxIOUniform::create_downscale_list(
            &mut Vec::new(),
            &fx_state.tex_size,
            downscale_count,
            1,
            1,
        );

        let upscale_list = FxIOUniform::reverse_list(&downscale_list);

        let blend_uniform = settings.final_blend;
        let blend_ctx = UniformContext::from_uniform(&blend_uniform, device, "blend");

        for io_uniform in downscale_list {
            downscale_passes.push(DownscalePass {
                downscale: Downscale::new(options, io_uniform),
            });
        }

        for (i, io_uniform) in upscale_list.into_iter().enumerate() {
            let blend_uniform = settings.upscale_blends[i];
            let blend_ctx = UniformContext::from_uniform(&blend_uniform, device, "blend");

            upscale_passes.push(UpscalePass {
                blend: BlendPass::new(
                    options,
                    BlendSettings {
                        io_uniform,
                        blend_layout: &blend_ctx.bg_layout,
                        if_directives: &[],
                    },
                ),
                blend_uniform,
                blend_ctx,
            });
        }

        let color = ColorFx::new(
            options,
            ColorFxSettings {
                io_uniform: FxIOUniform::symetric_unscaled(options.fx_state, 1),
                color_uniform: settings.color,
            },
        );

        let blend = BlendPass::new(
            options,
            BlendSettings {
                io_uniform: FxIOUniform::asymetric_unscaled(options.fx_state, 1, 0),
                blend_layout: &blend_ctx.bg_layout,
                if_directives: &[],
            },
        );

        Self {
            downscale_passes,
            upscale_passes,
            enabled: true,
            blend,
            blend_ctx,
            blend_uniform,
            color,
            update_event: None,
            selected_action: ListAction::None,
            bloom_treshold: settings.bloom_treshold,
        }
    }
}
