use glam::{Vec3, Vec4};
use sparticles::{
    animations::{ColorAnimation, StrayAnimation},
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

    sparticles::start(sparticles::InitialiseApp {
        show_gui: true,
        particle_animations: vec![Box::new(stray_animation), Box::new(color_animation)],
    });
}
