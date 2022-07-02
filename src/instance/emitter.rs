use super::angles::Angles;
use super::particle::Particle;
use crate::clock::Clock;
use cgmath::Zero;
use rand::{
    prelude::{thread_rng, ThreadRng},
    Rng,
};
use std::time::Duration;

const EMIT_RADIANS: f32 = 90_f32 * (std::f32::consts::PI / 180_f32); // 0 deg will be emitting above
pub struct EmitterSize {
    pub length: f32,
    pub depth: f32,
}

pub struct Emitter {
    pub emitter_position: cgmath::Vector3<f32>,
    pub emitter_size: EmitterSize,

    pub particles_per_emission: u32,
    pub delay_between_emission_ms: u32,
    pub particle_color: cgmath::Vector4<f32>,

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

    pub emission_distortion: f32,
    //pub particle_texture: Option<Texture2D>,
    pub particle_lifetime: Duration,

    pub particle_radius: f32,
    pub particle_mass: f32,

    pub bounds: Option<Bounds>,
    //pub particle_animation_options: Option<AnimationOptions>,
    //pub emitter_animation_handler: Option<EmitterAnimationHandler>,
    //pub force_handler: Option<ForceHandler>,
}

impl Default for Emitter {
    fn default() -> Self {
        Self {
            emitter_position: cgmath::Vector3::zero(),
            emitter_size: EmitterSize {
                length: 8.,
                depth: 4.,
            },
            delay_between_emission_ms: 400,
            iteration: 0,
            bounds: None,
            particle_mass: 1.,
            particle_speed: 0.01,
            particle_radius: 0.1,
            particle_lifetime: Duration::from_secs(5),
            particles_per_emission: 10,
            emission_distortion: 0.,
            diffusion_radians: Angles::new(45_f32.to_radians(), 45_f32.to_radians()),
            angle_radians: Angles::new(45_f32.to_radians(), 0_f32.to_radians()),
            emitter_duration: Duration::from_secs(30),
            particle_size: 0.4,
            particle_friction_coefficient: 0.99,
            particle_color: cgmath::Vector4::new(0., 1., 0., 1.),
        }
    }
}

pub struct Bounds {
    pub start_x: Option<f32>,
    pub start_y: Option<f32>,
    pub start_z: Option<f32>,
    pub end_x: Option<f32>,
    pub end_y: Option<f32>,
    pub end_z: Option<f32>,
}

pub struct SpawnData<'a> {
    pub elapsed_ms: u128,
    pub particles: &'a mut Vec<Particle>,
}

impl Emitter {
    pub fn spawn(&mut self, data: &mut SpawnData) {
        let elapsed_ms = data.elapsed_ms;
        let new_iteration = elapsed_ms as u32 / self.delay_between_emission_ms;

        if self.iteration == new_iteration {
            return;
        }

        self.iteration = new_iteration;

        //let velocity = cgmath::Vector3::new(self.particle_speed, self.particle_speed, 0.);

        let mut rng = thread_rng();

        //println!("random {}", rng.gen_range());

        for _ in 0..self.particles_per_emission {
            let emitter_length = gen_abs_range(&mut rng, self.emitter_size.length);
            let emitter_depth = gen_abs_range(&mut rng, self.emitter_size.depth);
            let distortion = gen_dyn_range(&mut rng, self.emission_distortion);

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

            data.particles.push(Particle {
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

    pub fn angle_emission_radians(&self) -> f32 {
        self.angle_radians.elevation + EMIT_RADIANS
    }
}

pub fn gen_dyn_range(rng: &mut ThreadRng, val: f32) -> f32 {
    if 0. < val {
        rng.gen_range(-val..val)
    } else {
        0.
    }
}

pub fn gen_abs_range(rng: &mut ThreadRng, val: f32) -> f32 {
    if 0. < val {
        rng.gen_range(0_f32..val)
    } else {
        0.
    }
}
