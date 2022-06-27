use super::particle::Particle;
use crate::clock::Clock;

pub struct Emitter {
    pub emitter_position: cgmath::Vector3<f32>,
    pub particles_per_emission: u32,
    pub delay_between_emission_ms: u32,
    pub particle_color: cgmath::Vector4<f32>,
    pub particle_speed: f32,
    pub particle_size: f32,
    pub iteration: u32,
}

pub struct SpawnData<'a> {
    pub clock: &'a Clock,
    pub instances: &'a mut Vec<f32>,
    pub num_spawned_particles: u32,
}

impl Emitter {
    pub fn spawn(&mut self, data: &mut SpawnData) {
        let elapsed_ms = data.clock.lifetime_ms() as u32;
        let new_iteration = elapsed_ms / self.delay_between_emission_ms;

        if self.iteration != new_iteration {
            self.iteration = new_iteration;

            let velocity = cgmath::Vector3::new(self.particle_speed, self.particle_speed, 0.);

            data.num_spawned_particles += self.particles_per_emission;

            for _ in 0..self.particles_per_emission {
                let particle = Particle {
                    position: self.emitter_position,
                    color: self.particle_color,
                    velocity,
                    size: self.particle_size,
                };

                particle.map_instance(&mut data.instances);
            }
        }
    }
}
