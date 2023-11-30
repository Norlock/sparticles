use crate::{
    fx::{FxOptions, FxState},
    model::{Camera, Clock, EmitterState, EmitterUniform, Events, GfxState, State},
    util::persistence::DynamicExport,
    util::ListAction,
};
use egui_wgpu::wgpu;
use egui_winit::egui::Ui;
use std::{collections::HashMap, num::NonZeroU64, slice::IterMut};

pub trait FromRGB {
    fn from_rgb(r: u8, g: u8, b: u8) -> Self;
}

pub trait FromRGBA {
    fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self;
}

pub trait CreateGui {
    fn create_gui(&self, app_state: &mut State);
}

pub trait ToVecF32 {
    fn to_vec_f32(&self) -> Vec<f32>;
}

// --------------------------- Animations ------------------------------
pub trait RegisterEmitterAnimation {
    fn tag(&self) -> &'static str;

    fn create_default(&self) -> Box<dyn EmitterAnimation>;

    fn import(&self, value: serde_json::Value) -> Box<dyn EmitterAnimation>;
}

pub trait RegisterParticleAnimation {
    fn tag(&self) -> &'static str;

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
        emitter: &'a EmitterState,
        clock: &Clock,
        compute_pass: &mut wgpu::ComputePass<'a>,
    );

    fn recreate(&self, gfx_state: &GfxState, emitter: &EmitterState) -> Box<dyn ParticleAnimation>;
    fn update(&mut self, clock: &Clock, gfx: &GfxState);
    fn draw_widget(&mut self, ui: &mut Ui) {}
}

pub trait WidgetBuilder {
    /// Root call -> from here your complete GUI can be created.
    fn draw_ui(&mut self, state: &mut State, encoder: &mut wgpu::CommandEncoder) -> Events;
}

pub trait DrawWidget<PA: ParticleAnimation>: WidgetBuilder + Sync + Send {
    /// Implementation for Particle animation so you can use GUI for dynamic dispatched animations
    fn draw_widget(&mut self, ui: &mut Ui, anim: &mut PA);
}

pub trait EmitterAnimation: HandleAction {
    fn animate(&mut self, emitter: &mut EmitterUniform, clock: &Clock);
}

// Post FX
pub trait PostFx: HandleAction {
    fn update(&mut self, gfx_state: &GfxState, camera: &mut Camera);

    fn compute<'a>(
        &'a self,
        fx_state: &'a FxState,
        gfx_state: &mut GfxState,
        c_pass: &mut wgpu::ComputePass<'a>,
    );

    fn resize(&mut self, options: &FxOptions);
}

pub trait RegisterPostFx {
    fn tag(&self) -> &'static str;
    fn create_default(&self, options: &FxOptions) -> Box<dyn PostFx>;
    fn import(&self, options: &FxOptions, value: serde_json::Value) -> Box<dyn PostFx>;
}

pub trait HandleAction {
    fn selected_action(&mut self) -> &mut ListAction;
    fn export(&self) -> DynamicExport;
    fn enabled(&self) -> bool;
}

pub trait CreateFxView {
    fn default_view(&self) -> wgpu::TextureView;
}

pub trait CalculateBufferSize {
    fn cal_buffer_size(&self) -> Option<NonZeroU64>;
}

pub trait HandleAngles {
    fn to_degrees(&self) -> Self;
    fn to_radians(&self) -> Self;
}

pub type OtherIterMut<'a, T> = std::iter::Chain<IterMut<'a, T>, IterMut<'a, T>>;

pub trait Splitting<T: std::fmt::Debug> {
    fn split_item_mut(&mut self, idx: usize) -> (&mut T, OtherIterMut<T>);
}
