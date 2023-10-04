#![allow(dead_code, unused)]
use glam::{Vec2, Vec3, Vec4};
use sparticles::{
    animations::{
        ColorUniform, DiffusionAnimation, ForceUniform, GravityUniform, GravityUniformOptions,
        StrayUniform, SwayAnimation,
    },
    init::{InitApp, SpawnInit},
    model::{Emitter, LifeCycle, Range},
    traits::*,
};

fn main() {
    let spawner_1 = get_spawner("Normal".to_owned());
    let light = get_light_spawner();

    sparticles::start(InitApp {
        show_gui: true,
        light,
        spawners: vec![spawner_1],
        //spawners: vec![],
    });
}

fn get_light_spawner() -> SpawnInit {
    let mut emitter = Emitter::default();

    emitter.box_pos.x = -3.;
    emitter.box_pos.y = -3.;
    emitter.particle_color = Vec4::from_rgb(175, 175, 255);
    emitter.particle_size = Range::new(0.25, 0.25);
    emitter.particle_speed = Range::new(5., 7.);
    emitter.spawn_count = 1;
    emitter.spawn_delay_sec = 1.;

    let emitter_sway_animation = SwayAnimation::new(
        LifeCycle {
            from_sec: 0.,
            until_sec: 4.,
            lifetime_sec: 4.,
        },
        glam::Vec2::ZERO,
        Vec2::new(30., 120.),
        glam::Vec2::ZERO,
    );

    return SpawnInit {
        id: "Light spawner".to_string(),
        emitter,
        particle_animations: vec![],
        emitter_animations: vec![Box::new(emitter_sway_animation)],
        //emitter_animations: vec![],
    };
}

fn get_spawner(id: String) -> SpawnInit {
    let stray_animation = StrayUniform {
        from_sec: 0.,
        until_sec: 100.,
        stray_radians: 5f32.to_radians(),
    };

    let color_animation = ColorUniform {
        from_sec: 0.,
        until_sec: 0.5,
        from_color: Vec4::from_rgb(0, 255, 0),
        to_color: Vec4::from_rgb(0, 0, 255),
    };

    let gravity_animation = GravityUniform::new(GravityUniformOptions {
        life_cycle: LifeCycle {
            from_sec: 0.,
            until_sec: 6.,
            lifetime_sec: 12.,
        },
        gravitational_force: 0.001,
        dead_zone: 4.,
        mass: 1_000_000.,
        start_pos: Vec3::new(-25., 8., 0.),
        end_pos: Vec3::new(25., 8., 0.),
    });

    let force_animation = ForceUniform {
        life_cycle: LifeCycle {
            from_sec: 0.,
            until_sec: 5.,
            lifetime_sec: 10.,
        },
        velocity: Vec3::new(-15., -15., 0.),
        mass_per_unit: 0.5,
    };

    let force_animation_2 = ForceUniform {
        life_cycle: LifeCycle {
            from_sec: 5.,
            until_sec: 10.,
            lifetime_sec: 10.,
        },
        velocity: Vec3::new(15., 0., 0.),
        mass_per_unit: 0.1,
    };

    let mut emitter = Emitter::default();
    emitter.spawn_count = 8;

    let diff_anim = DiffusionAnimation::new(
        LifeCycle {
            from_sec: 0.,
            until_sec: 5.,
            lifetime_sec: 5.,
        },
        Vec2::new(0., 45.),
        Vec2::new(0., 15.),
    );

    return SpawnInit {
        id,
        emitter,
        particle_animations: vec![
            Box::new(stray_animation),
            Box::new(color_animation),
            //Box::new(gravity_animation),
            Box::new(force_animation),
            Box::new(force_animation_2),
        ],
        emitter_animations: vec![Box::new(diff_anim)],
    };
}
