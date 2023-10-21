use crate::animations::color_animation::RegisterColorAnimation;
use crate::animations::diffusion_animation::RegisterDiffusionAnimation;
use crate::animations::sway_animation::RegisterSwayAnimation;
use crate::animations::{RegisterForceAnimation, RegisterGravityAnimation, RegisterStrayAnimation};
use crate::fx::bloom::RegisterBloomFx;
use crate::fx::blur::RegisterBlurFx;
use crate::fx::color_processing::RegisterColorProcessingFx;
use crate::fx::post_process::CreateFxOptions;
use crate::fx::PostProcessState;
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

#[allow(unused)]
pub trait AppSettings {
    fn show_gui(&self) -> bool;
    fn light(&self) -> EmitterUniform;
    fn emitters(&self) -> Vec<EmitterUniform>;

    /// If you want to add through code use this function otherwise use gui
    fn add_particle_anim(&self, emitter: &mut EmitterState, gfx_state: &GfxState) {}

    /// If you want to add through code use this function otherwise use gui
    fn add_emitter_anim(&self, emitter: &mut EmitterState) {}

    /// If you want to add through code use this function otherwise use gui
    fn add_post_fx(&self, options: &CreateFxOptions) -> Vec<Box<dyn PostFx>> {
        vec![]
    }

    fn import_mode(&self) -> JsonImportMode {
        JsonImportMode::Replace
    }

    /// If you want your animations available in the gui add to register
    fn register_custom_particle_animations(&self) -> Vec<Box<dyn RegisterParticleAnimation>> {
        vec![]
    }

    /// If you want your animations available in the gui add to register
    fn register_custom_emitter_animations(&self) -> Vec<Box<dyn RegisterEmitterAnimation>> {
        vec![]
    }

    fn register_custom_post_fx(&self) -> Vec<Box<dyn RegisterPostFx>> {
        vec![]
    }
}

pub struct InitEmitters {
    pub lights: EmitterState,
    pub emitters: Vec<EmitterState>,
    pub registered_par_anims: Vec<Box<dyn RegisterParticleAnimation>>,
    pub registered_em_anims: Vec<Box<dyn RegisterEmitterAnimation>>,
}

pub struct InitSettings;

impl InitSettings {
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

    fn code_emitters(
        app_settings: &impl AppSettings,
        gfx_state: &GfxState,
        camera: &Camera,
        registered_par_anims: Vec<Box<dyn RegisterParticleAnimation>>,
        registered_em_anims: Vec<Box<dyn RegisterEmitterAnimation>>,
    ) -> InitEmitters {
        let lights = InitSettings::create_light_spawner(app_settings, gfx_state, camera);

        let emitters = InitSettings::create_spawners(
            app_settings,
            gfx_state,
            &lights.bind_group_layout,
            camera,
        );

        InitEmitters {
            lights,
            emitters,
            registered_em_anims,
            registered_par_anims,
        }
    }

    pub fn create_post_fx(
        app_settings: &impl AppSettings,
        gfx_state: &GfxState,
        pp: &mut PostProcessState,
    ) -> Vec<Box<dyn RegisterPostFx>> {
        let mut registered_effects = app_settings.register_custom_post_fx();
        registered_effects.push(Box::new(RegisterBloomFx));
        registered_effects.push(Box::new(RegisterColorProcessingFx));
        registered_effects.push(Box::new(RegisterBlurFx));

        match app_settings.import_mode() {
            // TODO make from code possible
            JsonImportMode::Ignore => {}
            JsonImportMode::Replace => match Persistence::import_post_fx() {
                Ok(val) => pp.import_fx(gfx_state, &registered_effects, val),
                Err(err) => println!("{}", err.msg),
            },
        }

        registered_effects
    }

    pub fn create_emitters(
        app_settings: &impl AppSettings,
        gfx_state: &GfxState,
        camera: &Camera,
    ) -> InitEmitters {
        let mut registered_par_anims = app_settings.register_custom_particle_animations();
        registered_par_anims.push(Box::new(RegisterColorAnimation));
        registered_par_anims.push(Box::new(RegisterForceAnimation));
        registered_par_anims.push(Box::new(RegisterGravityAnimation));
        registered_par_anims.push(Box::new(RegisterStrayAnimation));

        let mut registered_em_anims = app_settings.register_custom_emitter_animations();
        registered_em_anims.push(Box::new(RegisterSwayAnimation));
        registered_em_anims.push(Box::new(RegisterDiffusionAnimation));

        match app_settings.import_mode() {
            JsonImportMode::Ignore => Self::code_emitters(
                app_settings,
                gfx_state,
                camera,
                registered_par_anims,
                registered_em_anims,
            ),
            JsonImportMode::Replace => match Persistence::import_emitter_states() {
                Ok(val) => Self::json_emitters(
                    val,
                    gfx_state,
                    camera,
                    registered_par_anims,
                    registered_em_anims,
                ),
                Err(err) => {
                    println!("{}", err.msg);
                    Self::code_emitters(
                        app_settings,
                        gfx_state,
                        camera,
                        registered_par_anims,
                        registered_em_anims,
                    )
                }
            },
        }
    }

    fn json_emitters(
        mut import_emitters: Vec<ExportEmitter>,
        gfx_state: &GfxState,
        camera: &Camera,
        registered_par_anims: Vec<Box<dyn RegisterParticleAnimation>>,
        registered_em_anims: Vec<Box<dyn RegisterEmitterAnimation>>,
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
                if export_animation.tag == reg.tag() {
                    let anim = reg.import(gfx_state, &lights, export_animation.data);
                    lights.push_particle_animation(anim);
                    break;
                }
            }
        }

        for export_animation in lights_export.emitter_animations {
            for reg in registered_em_anims.iter() {
                if export_animation.tag == reg.tag() {
                    let anim = reg.import(export_animation.data);
                    lights.push_emitter_animation(anim);
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
                    if export_animation.tag == reg.tag() {
                        let anim = reg.import(gfx_state, &emitter, export_animation.data);
                        emitter.push_particle_animation(anim);
                        break;
                    }
                }
            }

            for export_animation in emitter_export.emitter_animations {
                for reg in registered_em_anims.iter() {
                    if export_animation.tag == reg.tag() {
                        let anim = reg.import(export_animation.data);
                        emitter.push_emitter_animation(anim);
                        break;
                    }
                }
            }

            emitters.push(emitter);
        }

        InitEmitters {
            lights,
            emitters,
            registered_em_anims,
            registered_par_anims,
        }
    }
}
