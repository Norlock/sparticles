use egui_wgpu::wgpu;
use egui_winit::egui;

use crate::{model::GfxState, traits::PostFxChain};

use super::FxState;

pub struct ColorCorrection {
    pub gamma: f32,
    pub contrast: f32,
    pub brightness: f32,
}

impl PostFxChain for ColorCorrection {
    fn debug(&self) -> Option<&wgpu::BindGroup> {
        None
    }

    fn resize(&mut self, gfx_state: &GfxState, fx_state: &FxState) {}

    fn compute<'a>(&'a self, input: &'a wgpu::BindGroup, c_pass: &mut wgpu::ComputePass<'a>) {
        //
    }

    fn enabled(&self) -> bool {
        true
    }

    fn create_ui(&mut self, ui: &mut egui::Ui, gfx_state: &GfxState) {
        //
    }
}
