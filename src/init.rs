use crate::animations::color_animation::RegisterColorAnimation;
use crate::animations::{RegisterForceAnimation, RegisterGravityAnimation, RegisterStrayAnimation};
use crate::model::{Camera, CreateEmitterOptions, EmitterState, EmitterUniform, GfxState};
use crate::traits::*;
use crate::util::persistence::ExportEmitter;
use crate::util::Persistence;
use egui_wgpu::wgpu;

pub enum JsonImportMode {
    /// (Default) Will replace existing emitters with files
    Replace,
    /// Will ignore json file and use code only
    Ignore,
}

pub trait AppSettings {
    fn show_gui(&self) -> bool;
    fn light(&self) -> EmitterUniform;
    fn emitters(&self) -> Vec<EmitterUniform>;
    fn add_particle_anim(&self, emitter: &mut EmitterState, gfx_state: &GfxState);
    fn add_emitter_anim(&self, emitter: &mut EmitterState);

    fn import_mode(&self) -> JsonImportMode {
        JsonImportMode::Replace
    }

    fn register_custom_particle_animations(&self) -> Vec<Box<dyn RegisterParticleAnimation>> {
        vec![]
    }
}

pub struct InitEmitters {
    pub lights: EmitterState,
    pub emitters: Vec<EmitterState>,
    pub registered_par_anims: Vec<Box<dyn RegisterParticleAnimation>>,
}

pub struct InitSettings;

impl InitSettings {
    pub fn add_builtin_particle_animations(anims: &mut Vec<Box<dyn RegisterParticleAnimation>>) {
        anims.push(Box::new(RegisterColorAnimation));
        anims.push(Box::new(RegisterForceAnimation));
        anims.push(Box::new(RegisterGravityAnimation));
        anims.push(Box::new(RegisterStrayAnimation));
    }

    fn create_light_spawner(
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

    fn create_spawners(
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

    pub fn create_emitters(
        app_settings: &impl AppSettings,
        gfx_state: &GfxState,
        camera: &Camera,
    ) -> InitEmitters {
        let mut registered_par_anims = app_settings.register_custom_particle_animations();
        InitSettings::add_builtin_particle_animations(&mut registered_par_anims);

        match app_settings.import_mode() {
            JsonImportMode::Ignore => {
                let lights = InitSettings::create_light_spawner(app_settings, gfx_state, camera);

                let emitters = InitSettings::create_spawners(
                    app_settings,
                    gfx_state,
                    &lights.bind_group_layout,
                    camera,
                );

                InitEmitters {
                    registered_par_anims,
                    lights,
                    emitters,
                }
            }
            JsonImportMode::Replace => match Persistence::import_emitter_states() {
                Ok(val) => Self::import(val, gfx_state, camera, registered_par_anims),
                Err(err) => panic!("{:?}", err.msg),
            },
        }
    }

    fn import(
        mut import_emitters: Vec<ExportEmitter>,
        gfx_state: &GfxState,
        camera: &Camera,
        registered_par_anims: Vec<Box<dyn RegisterParticleAnimation>>,
    ) -> InitEmitters {
        let mut emitters = Vec::new();
        let mut lights_export: Option<ExportEmitter> = None;

        for i in 0..import_emitters.len() {
            if import_emitters[i].is_light {
                lights_export = Some(import_emitters.remove(i));
                break;
            }
        }

        assert!(
            lights_export.is_some(),
            "Lights is not in JSON export please remove exports file"
        );

        let lights_export = lights_export.unwrap();

        let mut lights = gfx_state.create_emitter_state(CreateEmitterOptions {
            emitter_uniform: lights_export.emitter,
            light_layout: None,
            camera,
        });

        // Import animations
        for export_animation in lights_export.particle_animations {
            for reg in registered_par_anims.iter() {
                if export_animation.animation_tag == reg.tag() {
                    let anim = reg.import(gfx_state, &lights, export_animation.animation);
                    lights.push_particle_animation(anim);
                    break;
                }
            }
        }

        for emitter_export in import_emitters {
            let mut emitter = gfx_state.create_emitter_state(CreateEmitterOptions {
                emitter_uniform: emitter_export.emitter,
                light_layout: Some(&lights.bind_group_layout),
                camera,
            });

            for export_animation in emitter_export.particle_animations {
                for reg in registered_par_anims.iter() {
                    if export_animation.animation_tag == reg.tag() {
                        let anim = reg.import(gfx_state, &emitter, export_animation.animation);
                        emitter.push_particle_animation(anim);
                        break;
                    }
                }
            }

            emitters.push(emitter);
        }

        InitEmitters {
            registered_par_anims,
            lights,
            emitters,
        }
    }
}
