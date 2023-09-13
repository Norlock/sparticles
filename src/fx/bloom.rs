use super::blur::Blur;
use super::Blend;
use super::FxState;
use super::Upscale;
use crate::traits::*;
use crate::GfxState;
use egui_wgpu::wgpu;
use egui_winit::egui::ComboBox;
use egui_winit::egui::Ui;

pub struct Bloom {
    blur: Blur,
    upscale: Upscale,
    blend: Blend,
    enabled: bool,
    debug: Debug,
}

#[derive(PartialEq, Debug)]
enum Debug {
    Blur,
    Upscale,
    None,
}

impl PostFxChain for Bloom {
    fn debug(&self) -> Option<&wgpu::BindGroup> {
        match self.debug {
            Debug::None => None,
            Debug::Blur => Some(self.blur.output()),
            Debug::Upscale => Some(self.upscale.output()),
        }
    }

    fn compute<'a>(&'a self, input: &'a wgpu::BindGroup, c_pass: &mut wgpu::ComputePass<'a>) {
        self.blur.compute(vec![input], c_pass);
        self.upscale.compute(vec![self.blur.output()], c_pass);
        self.blend.add(self.upscale.output(), input, c_pass);
    }

    fn resize(&mut self, gfx_state: &GfxState, fx_state: &FxState) {
        self.blur.resize(gfx_state);
        self.upscale.resize(gfx_state);
        self.blend.resize(fx_state);
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn create_ui(&mut self, ui: &mut Ui, gfx_state: &GfxState) {
        ui.label("Bloom settings");
        ui.add_space(5.0);

        self.blur.create_ui(ui, gfx_state);
        self.upscale.create_ui(ui, gfx_state);

        ui.checkbox(&mut self.enabled, "Enabled");

        // Dropdown
        ComboBox::from_label("Select debug type")
            .selected_text(format!("{:?}", &self.debug))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.debug, Debug::None, "None");
                ui.selectable_value(&mut self.debug, Debug::Blur, "Blur");
                ui.selectable_value(&mut self.debug, Debug::Upscale, "Upscale");
            });
    }
}

impl Bloom {
    pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

    pub fn new(gfx_state: &GfxState, fx_state: &FxState, depth_view: &wgpu::TextureView) -> Self {
        let blur = Blur::new(gfx_state, depth_view, "split_bloom");
        let upscale = Upscale::new(gfx_state);
        let blend = Blend::new(gfx_state, fx_state);

        Self {
            blur,
            blend,
            upscale,
            enabled: true,
            debug: Debug::None,
        }
    }
}
