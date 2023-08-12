#![allow(dead_code, unused)]
use glam::{Vec3, Vec4};
use sparticles::{
    animations::{
        sway_animation::SwayAnimation, ColorUniform, ForceUniform, GravityUniform,
        GravityUniformOptions, StrayUniform,
    },
    init::{InitApp, SpawnInit},
    model::{emitter::Emitter, LifeCycle},
    traits::FromRGB,
};

fn main() {
    let spawner_1 = get_spawner("Jep".to_owned());
    let light = get_light_spawner();

    sparticles::start(InitApp {
        show_gui: true,
        light,
        spawners: vec![spawner_1],
    });
}

fn get_light_spawner() -> SpawnInit {
    let mut emitter = Emitter::new();

    emitter.box_dimensions.x = -5.;
    emitter.box_dimensions.y = 5.;
    emitter.particle_color = Vec4::from_rgb(255, 255, 255);
    emitter.particle_size_min = 0.15;
    emitter.particle_size_max = 0.15;
    emitter.spawn_count = 1;
    emitter.particle_speed = 5.;

    let gravity_animation = GravityUniform::new(GravityUniformOptions {
        life_cycle: LifeCycle {
            from_sec: 0.,
            until_sec: 6.,
            lifetime_sec: 6.,
        },
        gravitational_force: 0.002,
        dead_zone: 0.3,
        mass: 100_000.,
        start_pos: Vec3::new(0., 8., 0.),
        end_pos: Vec3::new(-3., -5., 2.),
    });

    return SpawnInit {
        id: "other".to_string(),
        emitter,
        //particle_animations: vec![Box::new(gravity_animation)],
        particle_animations: vec![],
        emitter_animations: vec![],
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

    let emitter_sway_animation = SwayAnimation {
        life_cycle: LifeCycle {
            from_sec: 0.,
            until_sec: 2.,
            lifetime_sec: 4.,
        },
        yaw: -45f32.to_radians(),
        pitch: -130f32.to_radians(),
        roll: 0f32.to_radians(),
    };

    let mut emitter = Emitter::new();
    emitter.spawn_count = 7;

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
        emitter_animations: vec![Box::new(emitter_sway_animation)],
    };
}
