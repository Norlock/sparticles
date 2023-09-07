use crate::{
    fx::post_process::{FxChainOutput, FxState},
    model::{gfx_state::GfxState, AppState, Clock, Emitter, SpawnState},
};
use egui_wgpu::wgpu;
use egui_winit::egui::Ui;
use std::num::NonZeroU64;

pub trait FromRGB {
    fn from_rgb(r: u8, g: u8, b: u8) -> Self;
}

pub trait FromRGBA {
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
    fn into_animation(
        self: Box<Self>,
        gfx_state: &GfxState,
        spawner: &SpawnState,
    ) -> Box<dyn Animation>;
}

pub trait Animation {
    fn update(&mut self, clock: &Clock, gfx_state: &GfxState);

    fn compute<'a>(
        &'a self,
        spawner: &'a SpawnState,
        clock: &Clock,
        compute_pass: &mut wgpu::ComputePass<'a>,
    );

    fn recreate(&self, gfx_state: &GfxState, spawner: &SpawnState) -> Box<dyn Animation>;
}

pub trait EmitterAnimation {
    fn animate(&mut self, emitter: &mut Emitter, clock: &Clock);
}

pub trait CalculateBufferSize {
    fn cal_buffer_size(&self) -> Option<NonZeroU64>;
}

pub trait HandleAngles {
    fn to_degrees(&self) -> Self;
    fn to_radians(&self) -> Self;
}

pub trait CreateSpawner {
    fn create_emitter(&self) -> Emitter;
    fn create_id(&self) -> String;
}

pub trait PostFx {
    fn compute<'a>(
        &'a self,
        fx_inputs: Vec<&'a wgpu::BindGroup>,
        c_pass: &mut wgpu::ComputePass<'a>,
    );
    fn resize(&mut self, gfx_state: &GfxState);
    fn fx_state(&self) -> &FxState;
    fn output(&self) -> &wgpu::BindGroup;
    fn create_ui(&mut self, ui: &mut Ui, gfx_state: &GfxState);
}

pub trait PostFxChain {
    fn compute<'a>(
        &'a self,
        input: &'a wgpu::BindGroup,
        c_pass: &mut wgpu::ComputePass<'a>,
    ) -> FxChainOutput;

    fn resize(&mut self, gfx_state: &GfxState);
    fn enabled(&self) -> bool;
    fn create_ui(&mut self, ui: &mut Ui, gfx_state: &GfxState);
}

pub trait CreateFxView {
    fn into_view(&self) -> wgpu::TextureView;
}

pub trait FxDimensions {
    fn fx_dimensions(&self) -> [u32; 2];
    fn fx_offset(&self) -> u32;
}
