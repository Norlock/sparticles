use crate::animations::animation::AnimationHandler;
use crate::animations::color_animation::DuoColorAnimation;
use crate::animations::diffusion_animation::DiffusionAnimation;
use crate::animations::emitter_animation::EmitterAnimate;
use crate::animations::emitter_animation::EmitterAnimationHandler;
use crate::animations::sway_animation::SwayAnimation;

use crate::animations::size_animation::SizeAnimation;
use crate::animations::stray_animation::StrayAnimation;
use crate::forces::accelerating_force::AcceleratingForce;
use crate::forces::force::ForceHandler;
use crate::forces::lerp_force::LerpForce;
use crate::instance::color::Color;
use crate::instance::emitter::Emitter;
use std::time::Duration;

pub fn simple_emitter() -> Emitter {
    let mut emitter = Emitter::default();
    emitter.particle_size = 0.05;
    emitter.particle_speed = 20.;
    //emitter.particles_per_emission = 10000;

    let forces_length = Duration::from_secs(6).as_millis();
    let mut force_handler = ForceHandler::new(forces_length);

    force_handler.add(Box::new(AcceleratingForce {
        from_ms: 0,
        until_ms: 1000,
        nx: 50.,
        ny: 50.,
        nz: 0.,
        max_vx: 300.,
        max_vy: 300.,
        max_vz: 0.,
    }));

    force_handler.add(Box::new(LerpForce {
        from_ms: 2_000,
        until_ms: 3_000,
        min_nx: 0.,
        min_ny: -10.,
        min_nz: 0.,
        max_nx: 0.,
        max_ny: -40.,
        max_nz: 0.,
    }));

    //force_handler.add(Box::new(GravitationalForce {
    //from_ms: 0,
    //until_ms: 5000,
    //gravitational_force: 0.015,
    //dead_zone: 10.,
    //mass: 1000_000.,
    //start_pos: cgmath::Vector3::new(-1., 1., 0.),
    //end_pos: cgmath::Vector3::new(1., -1., 0.),
    //}));

    emitter.force_handler = Some(force_handler);

    let mut animation_handler = AnimationHandler::new(6000);

    animation_handler.add(Box::new(DuoColorAnimation {
        color_from: Color::rgb(0, 255, 0),
        color_to: Color::rgb(0, 0, 255),
        from_ms: 0000,
        until_ms: 3000,
    }));

    animation_handler.add(Box::new(DuoColorAnimation {
        color_from: Color::rgb(0, 0, 255),
        color_to: Color::rgb(255, 0, 0),
        from_ms: 3000,
        until_ms: 6000,
    }));

    animation_handler.add(Box::new(StrayAnimation::new(0, 5000, 7.)));

    animation_handler.add(Box::new(SizeAnimation {
        from_ms: 2000,
        until_ms: 3000,
        start_size: 0.05,
        end_size: 0.08,
    }));

    emitter.animation_handler = Some(animation_handler);

    //animation_handler.add(Box::new(DuoColorAnimation {
    //color_from: Color::rgba(0., 0., 1., 1.),
    //color_to: Color::rgba(0., 0., 1., 0.),
    //from_ms: 3000,
    //until_ms: 4000,
    //}));

    //animation_handler.add(Box::new(DuoColorAnimation {
    //color_from: Color::rgba(0., 0., 1., 0.),
    //color_to: Color::rgba(0., 0., 1., 1.),
    //from_ms: 4000,
    //until_ms: 5000,
    //}));

    let loop_ms = 12000;
    let mut emitter_animations: Vec<Box<dyn EmitterAnimate>> = Vec::new();
    emitter_animations.push(Box::new(DiffusionAnimation {
        from_ms: 0,
        until_ms: loop_ms,
        start_elevation_degrees: 10_f32,
        end_elevation_degrees: 90_f32,
        start_bearing_degrees: 10_f32,
        end_bearing_degrees: 90_f32,
    }));

    emitter_animations.push(Box::new(SwayAnimation {
        from_ms: 0,
        until_ms: 6000,
        start_elevation_degrees: 0_f32,
        end_elevation_degrees: 130_f32,
        start_bearing_degrees: 10_f32,
        end_bearing_degrees: 60_f32,
    }));

    emitter_animations.push(Box::new(SwayAnimation {
        from_ms: 6000,
        until_ms: 12000,
        start_elevation_degrees: 130_f32,
        end_elevation_degrees: 0_f32,
        start_bearing_degrees: 60_f32,
        end_bearing_degrees: 10_f32,
    }));

    emitter.emitter_animation_handler =
        Some(EmitterAnimationHandler::new(emitter_animations, loop_ms));

    emitter
}
