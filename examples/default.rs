use glam::{Vec3, Vec4};
use sparticles::{
    animations::{ColorAnimation, GravityAnimation, GravityAnimationOptions, StrayAnimation},
    model::{emitter::Emitter, LifeCycle},
    traits::FromRGB,
};

fn main() {
    let stray_animation = StrayAnimation {
        from_sec: 0.,
        until_sec: 100.,
        stray_radians: 5f32.to_radians(),
    };

    let color_animation = ColorAnimation {
        from_sec: 0.,
        until_sec: 0.5,
        from_color: Vec4::from_rgb(0, 255, 0),
        to_color: Vec4::from_rgb(0, 0, 255),
    };

    let gravity_animation = GravityAnimation::new(GravityAnimationOptions {
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

    sparticles::start(sparticles::InitialiseApp {
        emitter: Emitter::new(),
        show_gui: true,
        particle_animations: vec![
            Box::new(stray_animation),
            Box::new(color_animation),
            Box::new(gravity_animation),
        ],
    });
}
