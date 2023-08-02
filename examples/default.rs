use sparticles::{animations::StrayAnimation, traits::CreateAnimation};

fn main() {
    let stray_animation = StrayAnimation {
        from_sec: 0.,
        until_sec: 100.,
        stray_factor: 0.05,
    };

    sparticles::start(sparticles::InitialiseApp {
        show_gui: true,
        particle_animations: vec![Box::new(stray_animation)],
    });
}
