use super::particle::Particle;
use super::{angles::Angles, color::Color};
use crate::animations::emitter_animation::{EmitterAnimationData, EmitterAnimationHandler};
use crate::random::{gen_abs_range, gen_dyn_range};
use crate::{animations::animation::AnimationHandler, clock::Clock, forces::force::ForceHandler};
use cgmath::Zero;
use rand::prelude::thread_rng;
use std::time::Duration;

const EMIT_RADIANS: f32 = 90_f32 * (std::f32::consts::PI / 180_f32); // 0 deg will be emitting above
#[derive(Clone, Copy)]
pub struct EmitterSize {
    pub length: f32,
    pub depth: f32,
}

pub struct Emitter {
    pub emitter_position: cgmath::Vector3<f32>,
    pub emitter_size: EmitterSize,

    pub particles_per_emission: u32,
    pub delay_between_emission_ms: u32,
    pub particle_color: Color,

    /// Newton force
    pub particle_speed: f32,
    /// number between 0 and 1, e.g. 0.001
    pub particle_friction_coefficient: f32,

    pub particle_size: f32,
    pub iteration: u32,

    pub emitter_duration: Duration,
    pub angle_radians: Angles,

    /// Initial spread factor x,y / z
    pub diffusion_radians: Angles,

    pub emission_offset: f32,
    //pub particle_texture: Option<Texture2D>,
    pub particle_lifetime: Duration,

    pub particle_mass: f32,

    pub bounds: Option<Bounds>,
    pub animation_handler: Option<AnimationHandler>,
    pub emitter_animation_handler: Option<EmitterAnimationHandler>,
    pub force_handler: Option<ForceHandler>,
    pub particles: Vec<Particle>,
}

impl Default for Emitter {
    fn default() -> Self {
        Self {
            emitter_position: cgmath::Vector3::zero(),
            emitter_size: EmitterSize {
                length: 4.,
                depth: 4.,
            },
            delay_between_emission_ms: 400,
            iteration: 0,
            bounds: None,
            particle_mass: 1.,
            particle_speed: 10.0,
            particle_lifetime: Duration::from_secs(5),
            particles_per_emission: 100,
            emission_offset: 0.,
            diffusion_radians: Angles::new(45_f32.to_radians(), 45_f32.to_radians()),
            angle_radians: Angles::new(45_f32.to_radians(), 0_f32.to_radians()),
            emitter_duration: Duration::from_secs(30),
            particle_size: 0.1,
            particle_friction_coefficient: 0.997,
            particle_color: Color::rgb(0, 255, 0),
            force_handler: None,
            animation_handler: None,
            emitter_animation_handler: None,
            particles: Vec::new(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Bounds {
    pub start_x: Option<f32>,
    pub start_y: Option<f32>,
    pub start_z: Option<f32>,
    pub end_x: Option<f32>,
    pub end_y: Option<f32>,
    pub end_z: Option<f32>,
}

impl Emitter {
    pub fn spawn(&mut self, clock: &Clock) {
        let elapsed_ms = clock.elapsed_ms();
        let new_iteration = elapsed_ms as u32 / self.delay_between_emission_ms;

        if self.iteration == new_iteration {
            return;
        }

        self.iteration = new_iteration;

        let mut rng = thread_rng();

        for _ in 0..self.particles_per_emission {
            let emitter_length = gen_abs_range(&mut rng, self.emitter_size.length);
            let emitter_depth = gen_abs_range(&mut rng, self.emitter_size.depth);
            let distortion = gen_dyn_range(&mut rng, self.emission_offset);

            let Angles { elevation, bearing } = self.angle_radians;

            // Used to emit perpendicular of emitter.
            let perpendicular = elevation.cos() * -1.;
            let x = distortion + emitter_length * perpendicular * bearing.cos();
            let y = distortion + emitter_length * elevation.sin() * bearing.cos();
            let z = (distortion + emitter_depth) + emitter_length * bearing.sin();
            let particle_position = cgmath::Vector3::new(x, y, z);

            let diffusion_elevation_delta =
                gen_dyn_range(&mut rng, self.diffusion_radians.elevation);
            let bearing_radians = gen_dyn_range(&mut rng, self.diffusion_radians.bearing);
            let elevation_radians = self.angle_emission_radians() + diffusion_elevation_delta;

            // Used to emit perpendicular of emitter.
            let perpendicular = elevation_radians.cos() * -1.;
            let vx = self.particle_speed * perpendicular * bearing_radians.cos();
            let vy = self.particle_speed * elevation_radians.sin() * bearing_radians.cos();
            let vz = self.particle_speed * bearing_radians.sin();

            let velocity = cgmath::Vector3::new(vx, vy, vz);

            self.particles.push(Particle {
                position: particle_position,
                color: self.particle_color,
                velocity,
                size: self.particle_size,
                spawned_at: elapsed_ms,
                lifetime_ms: self.particle_lifetime.as_millis(),
                friction_coefficient: self.particle_friction_coefficient,
                mass: self.particle_mass,
            });
        }
    }

    pub fn handle_particles(&mut self, mut instances: &mut Vec<f32>, clock: &Clock) {
        let elapsed_ms = clock.elapsed_ms();

        self.particles.retain_mut(|particle| {
            let is_alive = elapsed_ms - particle.spawned_at < particle.lifetime_ms;

            particle.update(clock.delta_sec());

            if let Some(force_handler) = &self.force_handler {
                force_handler.apply(particle, &clock);
            }

            if let Some(animation_handler) = &self.animation_handler {
                animation_handler.apply(particle, &clock);
            }

            Particle::map_instance(particle, &mut instances);

            return is_alive;
        });
    }

    pub fn animate_emitter(&mut self, clock: &Clock) {
        if let Some(emitter_animation_handler) = &mut self.emitter_animation_handler {
            let mut data = EmitterAnimationData {
                angle_degrees: self.angle_radians.to_degrees(),
                diffusion_degrees: self.diffusion_radians.to_degrees(),
                emitter_position: self.emitter_position,
                emitter_size: self.emitter_size,
                emission_offset: self.emission_offset,
                particles_per_emission: self.particles_per_emission,
                delay_between_emission_ms: self.delay_between_emission_ms,
                bounds: self.bounds,
            };

            emitter_animation_handler.animate(&mut data, clock);

            self.angle_radians = data.angle_degrees.to_radians();
            self.diffusion_radians = data.diffusion_degrees.to_radians();
            self.emitter_position = data.emitter_position;
            self.emitter_size = data.emitter_size;
            self.emission_offset = data.emission_offset;
            self.particles_per_emission = data.particles_per_emission;
            self.delay_between_emission_ms = data.delay_between_emission_ms;
        }
    }

    pub fn angle_emission_radians(&self) -> f32 {
        self.angle_radians.elevation + EMIT_RADIANS
    }
}
