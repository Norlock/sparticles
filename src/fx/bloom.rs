use std::rc::Rc;

use super::blur::Blur;
use super::blur::BlurExport;
use super::post_process::CreateFxOptions;
use super::post_process::FxPersistenceType;
use super::post_process::FxView;
use super::Blend;
use super::FxState;
use crate::model::GuiState;
use crate::traits::*;
use crate::GfxState;
use egui_wgpu::wgpu;
use egui_winit::egui::Ui;
use serde::Deserialize;
use serde::Serialize;

pub struct Bloom {
    blur: Blur,
    blend: Blend,
    enabled: bool,
    delete: bool,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct BloomExport {
    pub blur: BlurExport,
}

impl PostFxChain for Bloom {
    fn add_views(&self, bind_groups: &mut Vec<FxView>, idx: usize) {
        bind_groups.push(FxView {
            tag: format!("Blur-{}", idx),
            bind_group: self.blur.output().clone(),
        });
    }

    fn compute<'a>(&'a self, input: &'a Rc<wgpu::BindGroup>, c_pass: &mut wgpu::ComputePass<'a>) {
        self.blur.compute(vec![input], c_pass);
        self.blend.add(self.blur.output(), input, c_pass);
    }

    fn resize(&mut self, gfx_state: &GfxState, fx_state: &FxState) {
        self.blur.resize(gfx_state);
        self.blend.resize(fx_state);
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn export(&self) -> FxPersistenceType {
        FxPersistenceType::Bloom(BloomExport {
            blur: self.blur.export(),
        })
    }

    fn create_ui(&mut self, ui: &mut Ui, gfx_state: &GfxState) {
        GuiState::create_title(ui, "Bloom settings");
        ui.add_space(5.0);

        self.blur.create_ui(ui, gfx_state);

        ui.checkbox(&mut self.enabled, "Enabled");

        if ui.button("Delete").clicked() {
            self.delete = true;
        }
    }

    fn delete(&self) -> bool {
        self.delete
    }
}

impl Bloom {
    pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

    pub fn new(options: &CreateFxOptions, export: BloomExport) -> Self {
        let CreateFxOptions {
            gfx_state,
            fx_state,
            ..
        } = options;

        let blur = Blur::new(options, export.blur);
        let blend = Blend::new(gfx_state, fx_state);

        Self {
            blur,
            blend,
            enabled: true,
            delete: false,
        }
    }
}
