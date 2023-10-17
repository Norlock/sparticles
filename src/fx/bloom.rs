use super::blur::Blur;
use super::blur::BlurSettings;
use super::post_process::CreateFxOptions;
use super::post_process::FxMetaUniform;
use super::Blend;
use super::BlendType;
use super::FxState;
use crate::model::GuiState;
use crate::traits::*;
use crate::util::DynamicExport;
use crate::util::ItemAction;
use egui_wgpu::wgpu;
use egui_winit::egui::Ui;
use serde::Deserialize;
use serde::Serialize;

pub struct Bloom {
    blur: Blur,
    blend: Blend,
    enabled: bool,
    selected_action: ItemAction,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BloomSettings {
    pub blur: BlurSettings,
    pub blend: FxMetaUniform,
}

impl Default for BloomSettings {
    fn default() -> Self {
        Self {
            blur: BlurSettings::new(FxMetaUniform {
                input_idx: 0,
                output_idx: 1,
            }),
            blend: FxMetaUniform {
                input_idx: 1,
                output_idx: 0,
            },
        }
    }
}

pub struct RegisterBloomFx;

impl RegisterPostFx for RegisterBloomFx {
    fn tag(&self) -> &str {
        "bloom"
    }

    fn import(&self, options: &CreateFxOptions, value: serde_json::Value) -> Box<dyn PostFx> {
        let blur_data = serde_json::from_value(value).unwrap();
        Box::new(Bloom::new(options, blur_data))
    }

    fn create_default(&self, options: &CreateFxOptions) -> Box<dyn PostFx> {
        Box::new(Bloom::new(options, BloomSettings::default()))
    }
}

impl PostFx for Bloom {
    fn compute<'a>(
        &'a self,
        ping_pong_idx: &mut usize,
        fx_state: &'a FxState,
        c_pass: &mut wgpu::ComputePass<'a>,
    ) {
        self.blur.compute(ping_pong_idx, fx_state, c_pass);
        self.blend.compute(ping_pong_idx, fx_state, c_pass);
    }

    fn create_ui(&mut self, ui: &mut Ui, ui_state: &GuiState) {
        GuiState::create_title(ui, "Bloom settings");
        ui.add_space(5.0);

        self.blur.create_ui(ui, ui_state);

        ui.checkbox(&mut self.enabled, "Enabled");

        if ui.button("Delete").clicked() {
            self.selected_action = ItemAction::Delete;
        }
    }
}

impl HandleAction for Bloom {
    fn selected_action(&mut self) -> &mut ItemAction {
        &mut self.selected_action
    }

    fn reset_action(&mut self) {
        todo!()
    }

    fn export(&self) -> DynamicExport {
        todo!()
    }

    fn enabled(&self) -> bool {
        self.enabled
    }
}

impl Bloom {
    pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

    pub fn new(options: &CreateFxOptions, bloom_settings: BloomSettings) -> Self {
        let blur = Blur::new(options, bloom_settings.blur);
        let blend = Blend::new(options, BlendType::ADDITIVE, bloom_settings.blend);

        Self {
            blur,
            blend,
            selected_action: ItemAction::None,
            enabled: true,
        }
    }
}
