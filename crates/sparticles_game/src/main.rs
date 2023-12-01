#![allow(dead_code, unused)]
use glam::{f32::Vec4, vec4, Vec2, Vec3};
use sparticles_app::{
    animations::{
        ColorAnimation, ColorUniform, DiffusionAnimation, ForceUniform, GravityUniform,
        GravityUniformOptions, RegisterColorAnimation, RegisterForceAnimation,
        RegisterGravityAnimation, RegisterStrayAnimation, StrayUniform, SwayAnimation,
    },
    gui::{winit::event::KeyboardInput, State},
    init::{AppVisitor, DataSource},
    loader::{BUILTIN_ID, CIRCLE_MAT_ID, CIRCLE_MESH_ID},
    model::{
        emitter::{MaterialRef, MeshRef},
        Boundry, EmitterState, EmitterUniform, GfxState, LifeCycle, SparEvents, SparState,
    },
    traits::*,
    wgpu::CommandEncoder,
};
use sparticles_editor::Editor;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

struct GameState {
    widget_builders: Vec<Box<dyn WidgetBuilder>>,
}

impl GameState {
    fn new() -> Self {
        Self {
            widget_builders: vec![],
        }
    }
}

const LIGHT_ID: &str = "Light";
const PARTICLE_ID: &str = "Particles";

impl AppVisitor for GameState {
    //fn lights(&self) -> EmitterUniform {
    //let mut emitter = EmitterUniform::new(LIGHT_ID.to_string());

    //emitter.box_position.x = -3.;
    //emitter.box_position.y = -3.;
    //emitter.particle_color = Vec4::new(0.9, 0.9, 0.9, 1.0);
    //emitter.hdr_mul = 5.0;
    //emitter.particle_size = Boundry::new(0.25, 0.25);
    //emitter.particle_speed = Boundry::new(5., 7.);
    //emitter.spawn_count = 1;
    //emitter.spawn_delay_sec = 1.;

    //emitter
    //}

    //fn emitters(&self) -> Vec<EmitterUniform> {
    //let mut emitter = EmitterUniform::new(PARTICLE_ID.to_string());
    //emitter.spawn_count = 1;
    //emitter.spawn_delay_sec = 2.0;

    ////emitter.mesh = MeshRef {
    ////collection_id: "drone.glb".to_string(),
    ////mesh_id: "RetopoGroup2".to_string(),
    //////collection_id: "StarSparrow.glb".to_string(),
    //////mesh_id: "Mesh.001".to_string(),
    ////};

    ////emitter.material = MaterialRef {
    ////collection_id: "drone.glb".to_string(),
    ////material_id: "Material.001".to_string(),
    //////collection_id: "StarSparrow.glb".to_string(),
    //////material_id: "StarSparrowRed".to_string(),
    ////};

    //vec![emitter]
    //}
    fn draw_ui(
        &mut self,
        state: &mut SparState,
        encoder: &mut CommandEncoder,
    ) -> sparticles_app::model::SparEvents {
        self.widget_builders[0].draw_ui(state, encoder)
    }

    fn add_widget_builders(&mut self, gfx: &mut GfxState) {
        self.widget_builders.push(Box::new(Editor::new(gfx)));
    }

    fn process_events(
        &mut self,
        events: &mut SparEvents,
        input: &KeyboardInput,
        shift_pressed: bool,
    ) {
        self.widget_builders[0].process_input(events, &input, shift_pressed);
    }

    fn add_emitter_anim(&self, emitter: &mut EmitterState) {
        if &emitter.id() == &LIGHT_ID {
            let sway_animation = SwayAnimation::new(
                LifeCycle {
                    from_sec: 0.,
                    until_sec: 4.,
                    lifetime_sec: 4.,
                },
                glam::Vec2::ZERO,
                Vec2::new(30., 120.),
                glam::Vec2::ZERO,
            );

            emitter.push_emitter_animation(Box::new(sway_animation));
        } else if emitter.id() == PARTICLE_ID {
            let diff_anim = DiffusionAnimation::new(
                LifeCycle {
                    from_sec: 0.,
                    until_sec: 5.,
                    lifetime_sec: 5.,
                },
                [0., 45.].into(),
                [0., 15.].into(),
            );

            emitter.push_emitter_animation(Box::new(diff_anim));
        }
    }

    fn add_particle_anim(&self, emitter: &mut EmitterState, gfx_state: &GfxState) {
        RegisterStrayAnimation::append(
            StrayUniform {
                from_sec: 0.,
                until_sec: 100.,
                stray_radians: 5f32.to_radians(),
            },
            emitter,
            gfx_state,
        );

        if emitter.id() == PARTICLE_ID {
            let color_anim = Box::new(ColorAnimation::new(
                ColorUniform {
                    from_sec: 0.,
                    until_sec: 0.5,
                    from_color: Vec4::from_rgb(0, 255, 0),
                    to_color: Vec4::from_rgb(0, 0, 255),
                },
                emitter,
                gfx_state,
            ));

            emitter.push_particle_animation(color_anim);

            RegisterForceAnimation::append(
                ForceUniform {
                    life_cycle: LifeCycle {
                        from_sec: 0.,
                        until_sec: 5.,
                        lifetime_sec: 10.,
                    },
                    velocity: [-15., -15., 0.].into(),
                    mass_per_unit: 8.5,
                },
                emitter,
                gfx_state,
            );

            RegisterForceAnimation::append(
                ForceUniform {
                    life_cycle: LifeCycle {
                        from_sec: 5.,
                        until_sec: 10.,
                        lifetime_sec: 10.,
                    },
                    velocity: [15., 0., 0.].into(),
                    mass_per_unit: 3.5,
                },
                emitter,
                gfx_state,
            );
        } else if emitter.id() == LIGHT_ID {
            RegisterGravityAnimation::append(
                GravityUniform::new(GravityUniformOptions {
                    life_cycle: LifeCycle {
                        from_sec: 1.,
                        until_sec: 6.,
                        lifetime_sec: 12.,
                    },
                    gravitational_force: 0.0015,
                    dead_zone: 4.,
                    mass: 100_000.,
                    start_pos: [-25., 8., 0.].into(),
                    end_pos: [25., 8., 0.].into(),
                }),
                emitter,
                gfx_state,
            );
        }
    }
}

fn main() {
    sparticles_app::start(GameState::new());
}