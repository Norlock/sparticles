use crate::{
    fx::{FxOptions, FxState},
    model::{Camera, Clock, EmitterState, EmitterUniform, GfxState, SparEvents, SparState},
    util::persistence::DynamicExport,
    util::ListAction,
};
use egui_wgpu::wgpu;
use egui_winit::{egui::Ui, winit::event::KeyboardInput};
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    num::NonZeroU64,
    slice::IterMut,
};

pub trait FromRGB {
    fn from_rgb(r: u8, g: u8, b: u8) -> Self;
}

pub trait FromRGBA {
    fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self;
}

pub trait CreateGui {
    fn create_gui(&self, app_state: &mut SparState);
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

#[allow(unused)]
pub trait ParticleAnimation: HandleAction {
    fn compute<'a>(
        &'a self,
        emitter: &'a EmitterState,
        clock: &Clock,
        compute_pass: &mut wgpu::ComputePass<'a>,
    );

    fn as_any(&mut self) -> &mut dyn Any;

    fn recreate(&self, gfx_state: &GfxState, emitter: &EmitterState) -> Box<dyn ParticleAnimation>;
    fn update(&mut self, clock: &Clock, gfx: &GfxState);
}

pub trait WidgetBuilder {
    fn id(&self) -> &'static str;

    fn as_any(&mut self) -> &mut dyn Any;

    /// Pass the type id of the animation (e.g. ColorAnimation::type_id())
    fn draw_pa_widget(&mut self, anim: &mut Box<dyn ParticleAnimation>, ui: &mut Ui);
    fn draw_em_widget(&mut self, anim: &mut Box<dyn EmitterAnimation>, ui: &mut Ui);
    fn draw_fx_widget(&mut self, anim: &mut Box<dyn PostFx>, ui: &mut Ui);

    /// Root call -> from here your complete GUI can be created.
    fn draw_ui(&mut self, state: &mut SparState, encoder: &mut wgpu::CommandEncoder) -> SparEvents;

    fn process_input(
        &mut self,
        events: &mut SparEvents,
        input: &KeyboardInput,
        shift_pressed: bool,
    ) -> bool;
}

pub trait DrawWidget<PA: ParticleAnimation>: Sync + Send {
    /// Implementation for Particle animation so you can use GUI for dynamic dispatched animations
    fn draw_widget(&self, wb: &mut dyn WidgetBuilder, anim: &mut PA, ui: &mut Ui);
}

pub trait EmitterAnimation: HandleAction {
    fn animate(&mut self, emitter: &mut EmitterUniform, clock: &Clock);
    fn as_any(&mut self) -> &mut dyn Any;
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
    fn as_any(&mut self) -> &mut dyn Any;
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