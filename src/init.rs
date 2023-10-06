use crate::animations::color_animation::RegisterColorAnimation;
use crate::animations::{RegisterForceAnimation, RegisterGravityAnimation, RegisterStrayAnimation};
use crate::model::{Camera, CreateEmitterOptions, EmitterState, EmitterUniform, GfxState};
use crate::traits::*;
use egui_wgpu::wgpu;

pub trait AppSettings {
    fn show_gui(&self) -> bool;
    fn light(&self) -> EmitterUniform;
    fn emitters(&self) -> Vec<EmitterUniform>;
    fn add_particle_anim(&self, emitter: &mut EmitterState, gfx_state: &GfxState);
    fn add_emitter_anim(&self, emitter: &mut EmitterState);

    fn register_custom_particle_animations(&self) -> Vec<Box<dyn RegisterParticleAnimation>> {
        vec![]
    }
}

pub struct InitSettings;

impl InitSettings {
    pub fn add_builtin_particle_animations(vector: &mut Vec<Box<dyn RegisterParticleAnimation>>) {
        vector.push(Box::new(RegisterColorAnimation));
        vector.push(Box::new(RegisterForceAnimation));
        vector.push(Box::new(RegisterGravityAnimation));
        vector.push(Box::new(RegisterStrayAnimation));
    }

    pub fn create_light_spawner(
        app_settings: &impl AppSettings,
        gfx_state: &GfxState,
        camera: &Camera,
    ) -> EmitterState {
        let mut lights = gfx_state.create_emitter_state(CreateEmitterOptions {
            camera,
            emitter_uniform: app_settings.light(),
            light_layout: None,
        });

        app_settings.add_particle_anim(&mut lights, gfx_state);
        app_settings.add_emitter_anim(&mut lights);

        lights
    }

    pub fn create_spawners(
        app_settings: &impl AppSettings,
        gfx_state: &GfxState,
        light_layout: &wgpu::BindGroupLayout,
        camera: &Camera,
    ) -> Vec<EmitterState> {
        let mut emitters: Vec<EmitterState> = Vec::new();

        for emitter_uniform in app_settings.emitters() {
            let is_unique = emitters
                .iter()
                .all(|emitter| emitter.uniform.id != emitter_uniform.id);

            assert!(!emitter_uniform.id.is_empty(), "Id can not be empty");
            assert!(is_unique, "Emitters require an unique ID");

            let mut emitter = gfx_state.create_emitter_state(CreateEmitterOptions {
                camera,
                emitter_uniform,
                light_layout: Some(light_layout),
            });

            app_settings.add_particle_anim(&mut emitter, gfx_state);
            app_settings.add_emitter_anim(&mut emitter);

            emitters.push(emitter);
        }

        emitters
    }
}
