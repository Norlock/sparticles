use crate::clock::Clock;

use super::{
    compute::Compute,
    particle::{Instance, Particle},
};

pub struct Emitter {
    pub emitter_position: cgmath::Vector3<f32>,
    pub particles_per_emission: u32,
    pub delay_between_emission_ms: u32,
    pub particle_color: cgmath::Vector4<f32>,
    pub particle_velocity: cgmath::Vector3<f32>,
    pub particle_size: f32,
    pub iteration: u32,
}

pub struct SpawnData {
    clock: Clock,
    instances: Vec<Instance>,
}

impl Emitter {
    pub fn spawn(&mut self, data: &SpawnData) {
        let elapsed_ms = data.clock.lifetime_ms() as u32;
        let new_iteration = elapsed_ms / self.delay_between_emission_ms;

        if self.iteration != new_iteration {
            self.iteration = new_iteration;

            for _ in 0..self.particles_per_emission {
                let particle = Particle {
                    position: self.emitter_position,
                    color: self.particle_color,
                    velocity: self.particle_velocity,
                    size: self.particle_size,
                };

                particle.map_instance(&mut data.instances);
            }
        }
    }
}
