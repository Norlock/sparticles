use sparticles::animations::StrayAnimation;

fn main() {
    let stray_animation = StrayAnimation {
        from_sec: 0.,
        until_sec: 100.,
        stray_radians: 5f32.to_radians(),
    };

    sparticles::start(sparticles::InitialiseApp {
        show_gui: true,
        particle_animations: vec![Box::new(stray_animation)],
    });
}
