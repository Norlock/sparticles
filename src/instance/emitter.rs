use super::particle::Particle;
use super::{angles::Angles, color::Color};
use crate::animations::emitter_animation::{EmitterAnimationData, EmitterAnimationHandler};
use crate::random::{gen_abs_range, gen_dyn_range};
use crate::{animations::animation::AnimationHandler, clock::Clock, forces::force::ForceHandler};
use cgmath::Zero;
use rand::prelude::thread_rng;
use std::collections::VecDeque;
use std::time::Duration;

const EMIT_RADIANS: f32 = 90_f32 * (std::f32::consts::PI / 180_f32); // 0 deg will be emitting above
#[derive(Clone, Copy)]
pub struct EmitterSize {
    pub length: f32,
    pub depth: f32,
}

pub struct Emitter {
    emitter_position: cgmath::Vector3<f32>,
    emitter_size: EmitterSize,
    emitter_duration: Duration,

    particles_per_emission: u32,
    particle_color: Color,
    particle_speed: f32,
    particle_friction_coefficient: f32,
    particle_size: f32,
    particle_lifetime: Duration,
    particle_mass: f32,

    iteration: u32,
    delay_between_emission_ms: u32,

    /// Angle of emitter x,y,z
    angle_radians: Angles,

    /// Spread factor x,y,z
    diffusion_radians: Angles,

    emission_offset: f32,
    //particle_texture: Option<Texture2D>,
    bounds: Vec<Bounds>,
    animation_handler: Option<AnimationHandler>,
    emitter_animation_handler: Option<EmitterAnimationHandler>,
    force_handler: Option<ForceHandler>,
    particles: Vec<Particle>,
    particle_shader_count: usize,
    trail_length: u32,
}

pub struct EmitterBuilder {
    em: Emitter,
}

impl EmitterBuilder {
    pub fn default() -> Self {
        let em = Emitter {
            emitter_position: cgmath::Vector3::zero(),
            emitter_size: EmitterSize {
                length: 4.,
                depth: 4.,
            },
            delay_between_emission_ms: 400,
            iteration: 0,
            bounds: Vec::new(),
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
            particle_shader_count: 0,
            trail_length: 0,
        };

        Self { em }
    }

    #[allow(dead_code)]
    pub fn emitter_position(mut self, emitter_position: cgmath::Vector3<f32>) -> Self {
        self.em.emitter_position = emitter_position;
        self
    }

    #[allow(dead_code)]
    pub fn emitter_size(mut self, emitter_size: EmitterSize) -> Self {
        self.em.emitter_size = emitter_size;
        self
    }

    #[allow(dead_code)]
    pub fn particles_per_emission(mut self, particles_per_emission: u32) -> Self {
        self.em.particles_per_emission = particles_per_emission;
        self
    }

    #[allow(dead_code)]
    pub fn delay_between_emission_ms(mut self, delay_between_emission_ms: u32) -> Self {
        self.em.delay_between_emission_ms = delay_between_emission_ms;
        self
    }

    #[allow(dead_code)]
    pub fn particle_color(mut self, particle_color: Color) -> Self {
        self.em.particle_color = particle_color;
        self
    }

    #[allow(dead_code)]
    pub fn particle_speed(mut self, particle_speed: f32) -> Self {
        self.em.particle_speed = particle_speed;
        self
    }

    #[allow(dead_code)]
    pub fn particle_friction_coefficient(mut self, particle_friction_coefficient: f32) -> Self {
        self.em.particle_friction_coefficient = particle_friction_coefficient;
        self
    }

    #[allow(dead_code)]
    pub fn particle_size(mut self, particle_size: f32) -> Self {
        self.em.particle_size = particle_size;
        self
    }

    #[allow(dead_code)]
    pub fn emitter_duation(mut self, emitter_duration: Duration) -> Self {
        self.em.emitter_duration = emitter_duration;
        self
    }

    #[allow(dead_code)]
    pub fn angle_degrees(mut self, angle_degrees: Angles) -> Self {
        self.em.angle_radians = angle_degrees.to_radians();
        self
    }

    /// Initial spread factor x,y / z
    #[allow(dead_code)]
    pub fn diffusion_degrees(mut self, diffusion_degrees: Angles) -> Self {
        self.em.diffusion_radians = diffusion_degrees.to_radians();
        self
    }

    #[allow(dead_code)]
    pub fn emission_offset(mut self, emission_offset: f32) -> Self {
        self.em.emission_offset = emission_offset;
        self
    }

    #[allow(dead_code)]
    pub fn particle_lifetime(mut self, particle_lifetime: Duration) -> Self {
        self.em.particle_lifetime = particle_lifetime;
        self
    }

    #[allow(dead_code)]
    pub fn particle_mass(mut self, particle_mass: f32) -> Self {
        self.em.particle_mass = particle_mass;
        self
    }

    #[allow(dead_code)]
    pub fn bounds(mut self, bounds: Vec<Bounds>) -> Self {
        self.em.bounds = bounds;
        self
    }

    #[allow(dead_code)]
    pub fn animation_handler(mut self, animation_handler: AnimationHandler) -> Self {
        self.em.animation_handler = Some(animation_handler);
        self
    }

    #[allow(dead_code)]
    pub fn emitter_animation_handler(
        mut self,
        emitter_animation_handler: EmitterAnimationHandler,
    ) -> Self {
        self.em.emitter_animation_handler = Some(emitter_animation_handler);
        self
    }

    #[allow(dead_code)]
    pub fn force_handler(mut self, force_handler: ForceHandler) -> Self {
        self.em.force_handler = Some(force_handler);
        self
    }

    #[allow(dead_code)]
    pub fn trail_length(mut self, trail_length: u32) -> Self {
        self.em.trail_length = trail_length;
        self
    }

    pub fn build(self) -> Emitter {
        self.em
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
    pub fn update(&mut self, clock: &Clock) {
        let elapsed_ms = clock.elapsed_ms();
        let new_iteration = elapsed_ms as u32 / self.delay_between_emission_ms;
        let update_trail = 0 < self.trail_length;
        self.particle_shader_count = 0;

        // Update particles
        self.particles.retain_mut(|particle| {
            if particle.lifetime_ms < elapsed_ms - particle.spawned_at {
                return false;
            }

            self.particle_shader_count += particle.history.len() + 1;
            particle.update(clock.delta_sec());

            if let Some(force_handler) = &self.force_handler {
                force_handler.apply(particle, &clock);
            }

            if let Some(animation_handler) = &self.animation_handler {
                animation_handler.apply(particle, &clock);
            }

            if update_trail {
                particle.history.push_back(particle.position);

                if self.trail_length < particle.history.len() as u32 {
                    particle.history.pop_front();
                }
            }

            return true;
        });

        if self.iteration == new_iteration {
            return;
        }

        self.iteration = new_iteration;
        self.particle_shader_count += self.particles_per_emission as usize;

        let mut rng = thread_rng();

        // Spawn particles
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
                history: VecDeque::new(),
            });
        }
    }

    pub fn map_particles(&mut self, instances: &mut Vec<f32>) {
        self.particles
            .iter()
            .for_each(|p| p.map_instance(instances));
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
                bounds: &self.bounds,
                particle_speed: self.particle_speed,
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

    pub fn particle_count(&self) -> usize {
        self.particle_shader_count
    }
}
