use crate::{
    fx::post_process::{CreateFxOptions, FxState},
    model::{Clock, EmitterState, EmitterUniform, GfxState, GuiState, State},
    util::persistence::DynamicExport,
    util::ItemAction,
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
    fn create_gui(&self, app_state: &mut State);
}

pub trait ToVecF32 {
    fn to_vec_f32(&self) -> Vec<f32>;
}

pub trait CreateAspect {
    fn aspect(&self) -> f32;
}

// --------------------------- Animations ------------------------------
pub trait RegisterEmitterAnimation {
    fn tag(&self) -> &str;

    fn create_default(&self) -> Box<dyn EmitterAnimation>;

    fn import(&self, value: serde_json::Value) -> Box<dyn EmitterAnimation>;
}

pub trait RegisterParticleAnimation {
    fn tag(&self) -> &str;

    fn create_default(
        &self,
        gfx_state: &GfxState,
        emitter: &EmitterState,
    ) -> Box<dyn ParticleAnimation>;

    fn import(
        &self,
        gfx_state: &GfxState,
        emitter: &EmitterState,
        value: serde_json::Value,
    ) -> Box<dyn ParticleAnimation>;
}

impl PartialEq for dyn RegisterParticleAnimation {
    fn eq(&self, other: &Self) -> bool {
        self.tag() == other.tag()
    }
}

pub trait ParticleAnimation: HandleAction {
    fn compute<'a>(
        &'a self,
        spawner: &'a EmitterState,
        clock: &Clock,
        compute_pass: &mut wgpu::ComputePass<'a>,
    );

    fn recreate(
        self: Box<Self>,
        gfx_state: &GfxState,
        spawner: &EmitterState,
    ) -> Box<dyn ParticleAnimation>;

    fn update(&mut self, clock: &Clock, gfx_state: &GfxState);
    fn create_ui(&mut self, ui: &mut Ui, gui: &GuiState);
}

pub trait EmitterAnimation: HandleAction {
    fn animate(&mut self, emitter: &mut EmitterUniform, clock: &Clock);
    fn create_ui(&mut self, ui: &mut Ui, gui: &GuiState);
}

// Post FX
pub trait PostFx: HandleAction {
    fn update(&mut self, gfx_state: &GfxState) {}

    fn compute<'a>(
        &'a self,
        ping_pong_idx: &mut usize,
        fx_state: &'a FxState,
        c_pass: &mut wgpu::ComputePass<'a>,
    );

    fn create_ui(&mut self, ui: &mut Ui, ui_state: &GuiState);
}

pub trait RegisterPostFx {
    fn tag(&self) -> &str;

    fn create_default(&self, options: &CreateFxOptions) -> Box<dyn PostFx>;

    fn import(&self, options: &CreateFxOptions, value: serde_json::Value) -> Box<dyn PostFx>;
}

pub trait HandleAction {
    fn selected_action(&mut self) -> &mut ItemAction;
    fn reset_action(&mut self);
    fn export(&self) -> DynamicExport;
    fn enabled(&self) -> bool;
}

pub trait CreateFxView {
    fn default_view(&self) -> wgpu::TextureView;
}

pub trait FxDimensions {
    fn fx_dimensions(&self) -> [u32; 2];
    fn fx_offset(&self) -> u32;
}

pub trait CalculateBufferSize {
    fn cal_buffer_size(&self) -> Option<NonZeroU64>;
}

pub trait HandleAngles {
    fn to_degrees(&self) -> Self;
    fn to_radians(&self) -> Self;
}
