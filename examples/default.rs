#![allow(dead_code, unused)]
use glam::{Vec2, Vec3, Vec4};
use sparticles::{
    animations::{
        ColorUniform, DiffusionAnimation, ForceUniform, GravityUniform, GravityUniformOptions,
        RegisterColorAnimation, RegisterForceAnimation, RegisterGravityAnimation,
        RegisterStrayAnimation, StrayUniform, SwayAnimation,
    },
    init::{AppSettings, JsonImportMode},
    math::{SparVec3, SparVec4},
    model::{EmitterState, EmitterUniform, GfxState, LifeCycle, Range},
    traits::*,
};

struct CustomSettings;

const LIGHT_ID: &str = "Light";
const PARTICLE_ID: &str = "Particles";

impl AppSettings for CustomSettings {
    fn light(&self) -> EmitterUniform {
        let mut emitter = EmitterUniform::new(LIGHT_ID.to_string());

        emitter.box_pos.x = -3.;
        emitter.box_pos.y = -3.;
        emitter.particle_color = SparVec4::from_rgb(175, 175, 255);
        emitter.particle_size = Range::new(0.25, 0.25);
        emitter.particle_speed = Range::new(5., 7.);
        emitter.spawn_count = 1;
        emitter.spawn_delay_sec = 1.;

        emitter
    }

    fn emitters(&self) -> Vec<EmitterUniform> {
        let mut emitter = EmitterUniform::new(PARTICLE_ID.to_string());
        emitter.spawn_count = 8;

        vec![emitter]
    }

    fn show_gui(&self) -> bool {
        true
    }

    fn import_mode(&self) -> JsonImportMode {
        JsonImportMode::Replace
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
            RegisterColorAnimation::append(
                ColorUniform {
                    from_sec: 0.,
                    until_sec: 0.5,
                    from_color: Vec4::from_rgb(0, 255, 0),
                    to_color: Vec4::from_rgb(0, 0, 255),
                },
                emitter,
                gfx_state,
            );

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
    sparticles::start(CustomSettings);
}
