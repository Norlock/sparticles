use crate::instance::emitter::Emitter;

pub fn simple_emitter() -> Emitter {
    //
    let emitter = Emitter {
        emitter_position: cgmath::Vector3::new(0., 0., 0.),
        particles_per_emission: 1,
        delay_between_emission_ms: 400,
        particle_color: cgmath::Vector4::new(0., 1., 0., 1.),
        particle_speed: 0.01,
        particle_size: 0.4,
        iteration: 0,
    };

    emitter
}
