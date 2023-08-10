use glam::{Vec3, Vec4};
use sparticles::{
    animations::{
        sway_animation::SwayAnimation, ColorUniform, ForceUniform, GravityUniform,
        GravityUniformOptions, StrayUniform,
    },
    model::{emitter::Emitter, LifeCycle},
    traits::FromRGB,
    SpawnerInit,
};

#[allow(dead_code, unused)]
fn main() {
    let spawner = get_spawner();
    let spawner_2 = get_simple_spawner();

    sparticles::start(sparticles::InitialiseApp {
        show_gui: true,
        spawners: vec![spawner, spawner_2],
    });
}

fn get_simple_spawner() -> SpawnerInit {
    let mut emitter = Emitter::new();

    emitter.box_dimensions.x = -5.;
    emitter.box_dimensions.y = 5.;
    emitter.particle_color = Vec4::from_rgb(255, 0, 0);
    emitter.particle_size_min = 0.01;
    emitter.particle_size_min = 0.03;
    //emitter

    return SpawnerInit {
        id: "other".to_string(),
        emitter,
        particle_animations: vec![],
        emitter_animations: vec![],
    };
}

fn get_spawner() -> SpawnerInit {
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
            lifetime_sec: 7.,
        },
        velocity: Vec3::new(-15., -15., 0.),
        mass_per_unit: 0.20,
    };

    let force_animation_2 = ForceUniform {
        life_cycle: LifeCycle {
            from_sec: 5.,
            until_sec: 9.,
            lifetime_sec: 9.,
        },
        velocity: Vec3::new(15., 0., 0.),
        mass_per_unit: 0.30,
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

    return SpawnerInit {
        id: "Simple".to_string(),
        emitter: Emitter::new(),
        particle_animations: vec![
            Box::new(stray_animation),
            Box::new(color_animation),
            Box::new(gravity_animation),
            Box::new(force_animation),
            Box::new(force_animation_2),
        ],
        emitter_animations: vec![Box::new(emitter_sway_animation)],
    };
}
