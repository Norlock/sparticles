use std::collections::HashMap;
use std::path::{Path, PathBuf};

use egui_wgpu::wgpu;
use egui_winit::winit::event::KeyboardInput;

use crate::animations::color_animation::RegisterColorAnimation;
use crate::animations::diffusion_animation::RegisterDiffusionAnimation;
use crate::animations::sway_animation::RegisterSwayAnimation;
use crate::animations::{RegisterForceAnimation, RegisterGravityAnimation, RegisterStrayAnimation};
use crate::fx::bloom::RegisterBloomFx;
use crate::fx::blur::RegisterBlurFx;
use crate::fx::FxOptions;
use crate::fx::PostProcessState;
use crate::fx::RegisterColorFx;
use crate::loader::Model;
use crate::model::{
    Camera, CreateEmitterOptions, EmitterState, EmitterType, EmitterUniform, GfxState,
};
pub use crate::model::{SparEvents, SparState};
use crate::traits::*;
use crate::util::persistence::ExportEmitter;
use crate::util::{Persistence, ID};

#[derive(Default)]
pub enum DataSource {
    Json {
        path: PathBuf,
    },
    Code {
        lights: EmitterUniform,
        emitters: Vec<EmitterUniform>,
    },
    /// Will generate some default values
    #[default]
    Demo,
}

#[allow(unused)]
pub trait AppVisitor {
    fn data_source(&self) -> DataSource {
        DataSource::default()
    }

    fn model_dir(&self) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets/models")
    }

    fn exports_dir(&self) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("exports")
    }

    fn add_widget_builders(&mut self, gfx: &mut GfxState);

    fn draw_ui(&mut self, state: &mut SparState, encoder: &mut wgpu::CommandEncoder) -> SparEvents;

    fn process_events(
        &mut self,
        events: &mut SparEvents,
        input: &KeyboardInput,
        shift_pressed: bool,
    );

    /// If you want to add through code use this function otherwise use gui
    fn add_particle_anim(&self, emitter: &mut EmitterState, gfx_state: &GfxState) {}

    /// If you want to add through code use this function otherwise use gui
    fn add_emitter_anim(&self, emitter: &mut EmitterState) {}

    /// If you want to add through code use this function otherwise use gui
    fn add_post_fx(&self, options: &FxOptions, effects: &mut Vec<Box<dyn PostFx>>) {}

    /// If you want your animations available in the gui add to registry
    fn register_particle_animations(&self, registry: &mut Vec<Box<dyn RegisterParticleAnimation>>) {
    }

    /// If you want your animations available in the gui add to registry
    fn register_emitter_animations(&self, registry: &mut Vec<Box<dyn RegisterEmitterAnimation>>) {}

    /// If you want your post FX available in the gui add to registry
    fn register_post_fx(&self, registry: &mut Vec<Box<dyn RegisterPostFx>>) {}
}

pub struct Init {
    pub emitters: Vec<EmitterState>,
    pub registry_par_anims: Vec<Box<dyn RegisterParticleAnimation>>,
    pub registry_em_anims: Vec<Box<dyn RegisterEmitterAnimation>>,
    pub registry_post_fx: Vec<Box<dyn RegisterPostFx>>,
}

impl Init {
    fn code_emitters(
        lights_uniform: EmitterUniform,
        emitter_uniforms: Vec<EmitterUniform>,
        gfx: &GfxState,
        collection: &HashMap<ID, Model>,
        camera: &Camera,
    ) -> Vec<EmitterState> {
        let mut emitters: Vec<EmitterState> = Vec::new();

        assert!(!&lights_uniform.id.is_empty(), "Id can not be empty");

        let lights = EmitterState::new(CreateEmitterOptions {
            uniform: lights_uniform,
            camera,
            collection,
            gfx,
            emitter_type: EmitterType::Lights,
        });

        for emitter_uniform in emitter_uniforms {
            let is_unique = emitters
                .iter()
                .all(|emitter| emitter.uniform.id != emitter_uniform.id);

            assert!(!emitter_uniform.id.is_empty(), "Id can not be empty");
            assert!(is_unique, "Emitters require an unique ID");

            emitters.push(EmitterState::new(CreateEmitterOptions {
                uniform: emitter_uniform,
                camera,
                collection,
                gfx,
                emitter_type: EmitterType::Normal {
                    lights_layout: &lights.bg_layout,
                },
            }));
        }

        emitters.insert(0, lights);

        emitters
    }

    pub fn new(
        app_visitor: &impl AppVisitor,
        gfx: &GfxState,
        camera: &Camera,
        collection: &HashMap<ID, Model>,
        pp: &mut PostProcessState,
    ) -> Init {
        let mut registry_par_anims: Vec<Box<dyn RegisterParticleAnimation>> = vec![];
        registry_par_anims.push(Box::new(RegisterColorAnimation));
        registry_par_anims.push(Box::new(RegisterForceAnimation));
        registry_par_anims.push(Box::new(RegisterGravityAnimation));
        registry_par_anims.push(Box::new(RegisterStrayAnimation));
        app_visitor.register_particle_animations(&mut registry_par_anims);

        let mut registry_em_anims: Vec<Box<dyn RegisterEmitterAnimation>> = vec![];
        registry_em_anims.push(Box::new(RegisterSwayAnimation));
        registry_em_anims.push(Box::new(RegisterDiffusionAnimation));
        app_visitor.register_emitter_animations(&mut registry_em_anims);

        let mut registry_post_fx: Vec<Box<dyn RegisterPostFx>> = vec![];
        registry_post_fx.push(Box::new(RegisterBloomFx));
        registry_post_fx.push(Box::new(RegisterColorFx));
        registry_post_fx.push(Box::new(RegisterBlurFx));
        app_visitor.register_post_fx(&mut registry_post_fx);

        match app_visitor.data_source() {
            DataSource::Code { lights, emitters } => {
                let mut emitters = Self::code_emitters(lights, emitters, gfx, collection, camera);

                for emitter in emitters.iter_mut() {
                    app_visitor.add_particle_anim(emitter, gfx);
                    app_visitor.add_emitter_anim(emitter);
                }

                Init {
                    emitters,
                    registry_em_anims,
                    registry_par_anims,
                    registry_post_fx,
                }
            }
            DataSource::Demo => {
                let lights = EmitterUniform::new("lights".to_string());
                let emitters = Self::code_emitters(lights, vec![], gfx, collection, camera);

                Self {
                    emitters,
                    registry_par_anims,
                    registry_em_anims,
                    registry_post_fx,
                }
            }
            DataSource::Json { path } => match Persistence::import_emitter_states(path) {
                Ok(exported_emitters) => {
                    match Persistence::import_post_fx() {
                        Ok(val) => pp.import_fx(gfx, &registry_post_fx, val),
                        Err(err) => println!("{}", err.msg),
                    }

                    Self::json_emitters(
                        exported_emitters,
                        gfx,
                        camera,
                        collection,
                        registry_par_anims,
                        registry_em_anims,
                        registry_post_fx,
                    )
                }
                Err(err) => {
                    panic!("{}", err.msg);
                }
            },
        }
    }

    fn json_emitters(
        mut emitters_export: Vec<ExportEmitter>,
        gfx: &GfxState,
        camera: &Camera,
        collection: &HashMap<ID, Model>,
        registry_par_anims: Vec<Box<dyn RegisterParticleAnimation>>,
        registry_em_anims: Vec<Box<dyn RegisterEmitterAnimation>>,
        registry_post_fx: Vec<Box<dyn RegisterPostFx>>,
    ) -> Self {
        let mut emitters = Vec::new();

        let lights_export = emitters_export.remove(0);

        assert!(
            lights_export.is_light,
            "Lights is not in JSON export please remove exports emitters.json file"
        );

        let mut lights = EmitterState::new(CreateEmitterOptions {
            uniform: lights_export.emitter,
            camera,
            collection,
            gfx,
            emitter_type: EmitterType::Lights,
        });

        // Import animations
        for export_animation in lights_export.particle_animations {
            for reg in registry_par_anims.iter() {
                if export_animation.tag == reg.tag() {
                    let anim = reg.import(gfx, &lights, export_animation.data);
                    lights.push_particle_animation(anim);
                    break;
                }
            }
        }

        for export_animation in lights_export.emitter_animations {
            for reg in registry_em_anims.iter() {
                if export_animation.tag == reg.tag() {
                    let anim = reg.import(export_animation.data);
                    lights.push_emitter_animation(anim);
                    break;
                }
            }
        }

        for emitter_export in emitters_export {
            let mut emitter = EmitterState::new(CreateEmitterOptions {
                uniform: emitter_export.emitter,
                camera,
                collection,
                gfx,
                emitter_type: EmitterType::Normal {
                    lights_layout: &lights.bg_layout,
                },
            });

            for export_animation in emitter_export.particle_animations {
                for reg in registry_par_anims.iter() {
                    if export_animation.tag == reg.tag() {
                        let anim = reg.import(gfx, &emitter, export_animation.data);
                        emitter.push_particle_animation(anim);
                        break;
                    }
                }
            }

            for export_animation in emitter_export.emitter_animations {
                for reg in registry_em_anims.iter() {
                    if export_animation.tag == reg.tag() {
                        let anim = reg.import(export_animation.data);
                        emitter.push_emitter_animation(anim);
                        break;
                    }
                }
            }

            emitters.push(emitter);
        }

        emitters.insert(0, lights);

        Init {
            emitters,
            registry_em_anims,
            registry_par_anims,
            registry_post_fx,
        }
    }
}
