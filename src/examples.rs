use crate::instance::angles::Angles;
use crate::instance::emitter::{Emitter, EmitterSize};
use std::time::Duration;

pub fn simple_emitter() -> Emitter {
    let emitter = Emitter {
        emitter_position: cgmath::Vector3::new(0., 0., 0.),
        particles_per_emission: 1,
        delay_between_emission_ms: 400,
        particle_color: cgmath::Vector4::new(0., 1., 0., 1.),
        particle_speed: 10.,
        particle_size: 0.4,
        iteration: 0,
        emitter_size: EmitterSize {
            length: 8.,
            depth: 4.,
        },
        particle_friction_coefficient: 0.997,
        emitter_duration: Duration::from_secs(30),
        angle_radians: Angles::new(45., 0.),
        diffusion_radians: Angles::new(45., 45.),
        emission_distortion: 0.,
        particle_lifetime: Duration::from_secs(5),
        particle_radius: 0.1,
        particle_mass: 1.,
        bounds: None,
    };

    emitter
}
