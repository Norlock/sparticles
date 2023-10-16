use super::blur::Blur;
use super::blur::BlurData;
use super::post_process::CreateFxOptions;
use super::Blend;
use super::FxState;
use crate::animations::ItemAction;
use crate::model::GuiState;
use crate::traits::*;
use crate::util::DynamicExport;
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
    pub blur: BlurData,
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
        Box::new(Bloom::new(options, BlurData::default()))
    }
}

impl PostFx for Bloom {
    fn compute<'a>(
        &'a self,
        ping_pong_idx: &mut usize,
        fx_state: &'a FxState,
        c_pass: &mut wgpu::ComputePass<'a>,
    ) {
        //self.blur.compute(vec![input], c_pass);
        //self.blend.add(self.blur.output(), input, c_pass);
    }

    fn create_ui(&mut self, ui: &mut Ui, ui_state: &GuiState) {
        GuiState::create_title(ui, "Bloom settings");
        ui.add_space(5.0);

        self.blur.create_ui(ui, ui_state);

        ui.checkbox(&mut self.enabled, "Enabled");

        if ui.button("Delete").clicked() {
            self.delete = true;
        }
    }

    fn reserved_space(&self) -> usize {
        todo!()
    }
}

impl HandleAction for Bloom {
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

impl Bloom {
    pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

    pub fn new(options: &CreateFxOptions, blur_data: BlurData) -> Self {
        let CreateFxOptions {
            gfx_state,
            fx_state,
        } = options;

        let blur = Blur::new(options, blur_data);
        let blend = Blend::new(gfx_state, fx_state);

        Self {
            blur,
            blend,
            enabled: true,
            delete: false,
        }
    }
}
