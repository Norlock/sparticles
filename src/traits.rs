use egui_wgpu_backend::wgpu;

use crate::model::{gfx_state::GfxState, AppState, Clock, ComputeState};

pub trait FromRGB {
    fn from_rgb(r: u8, g: u8, b: u8) -> Self;
    fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self;
}

pub trait CustomShader {
    fn create_shader(&self, shader_str_raw: &str, label: &str) -> wgpu::ShaderModule;
}

pub trait CreateGui {
    fn create_gui(&self, app_state: &mut AppState);
}

pub trait ToVecF32 {
    fn to_vec_f32(&self) -> Vec<f32>;
}

pub trait CreateAspect {
    fn aspect(&self) -> f32;
}

pub trait CreateAnimation {
    fn create_animation(
        self: Box<Self>,
        gfx_state: &GfxState,
        compute: &ComputeState,
    ) -> Box<dyn Animation>;
}

pub trait Animation {
    fn update(&mut self, clock: &Clock, gfx_state: &GfxState);

    fn compute<'a>(&'a self, compute: &'a ComputeState, compute_pass: &mut wgpu::ComputePass<'a>);
}
