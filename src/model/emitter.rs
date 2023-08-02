use crate::traits::FromRGB;
use glam::{Vec3, Vec4};

use super::Clock;

const PARTICLE_BUFFER_SIZE: u64 = 16 * 4;

pub struct Emitter {
    spawn_from: u32,
    spawn_until: u32,
    pub spawn_count: u32,
    pub spawn_delay_sec: f32,
    pub spawn_batches_count: u32,

    pub box_pos: Vec3,

    /// width, height, depth
    pub box_dimensions: Vec3,
    /// yaw, pitch, roll
    pub box_rotation: Vec3,

    /// Diffusion emission in radians
    pub diffusion_width_rad: f32,
    /// Diffusion emission in radians
    pub diffusion_depth_rad: f32,

    pub particle_color: Vec4,
    pub particle_friction_coefficient: f32,
    pub particle_speed: f32,
    pub particle_size: f32,
    pub particle_mass: f32,
    pub particle_lifetime_sec: f32,

    elapsed_sec: f32,
    delta_sec: f32,
    iteration: u32,
}

impl Emitter {
    pub fn new() -> Self {
        let spawn_count: u32 = 6;
        let particle_lifetime_sec: f32 = 6.;
        let spawn_delay_sec: f32 = 0.5;

        let spawn_batches_count = (particle_lifetime_sec / spawn_delay_sec).ceil() as u32;

        let box_pos = Vec3::ZERO;
        let box_dimensions = Vec3::new(1., 0.5, 1.);
        let box_rotation = Vec3::ZERO;

        let diffusion_width_rad = 15f32.to_radians();
        let diffusion_depth_rad = 15f32.to_radians();

        Self {
            spawn_delay_sec: 0.5,
            spawn_from: 0,
            spawn_until: 0,
            spawn_count,
            spawn_batches_count,

            box_pos,
            box_dimensions,
            box_rotation,

            diffusion_width_rad,
            diffusion_depth_rad,

            particle_mass: 1.,
            particle_speed: 15.,
            particle_lifetime_sec,
            particle_size: 0.1,
            particle_friction_coefficient: 0.95,
            particle_color: Vec4::from_rgb(0, 255, 0),

            iteration: 1000,
            elapsed_sec: 0.,
            delta_sec: 0.0,
        }
    }

    pub fn update(&mut self, clock: &Clock) {
        self.delta_sec = clock.delta_sec();
        self.elapsed_sec = clock.elapsed_sec();

        let new_iteration = (self.elapsed_sec / self.spawn_delay_sec) as u32;
        let current_batch = new_iteration % self.spawn_batches_count as u32;

        if new_iteration != self.iteration {
            self.spawn_from = current_batch * self.spawn_count;
            self.spawn_until = self.spawn_from + self.spawn_count;
            self.iteration = new_iteration;
        } else {
            // disables spawning in compute shader
            self.spawn_from = 0;
            self.spawn_until = 0;
        }
    }

    pub fn particle_count(&self) -> u64 {
        self.spawn_count as u64 * self.spawn_batches_count as u64
    }

    pub fn particle_buffer_size(&self) -> u64 {
        self.particle_count() * PARTICLE_BUFFER_SIZE
    }

    pub fn create_buffer_content(&self) -> Vec<f32> {
        vec![
            self.delta_sec,
            self.elapsed_sec,
            self.spawn_from as f32,
            self.spawn_until as f32,
            self.box_pos.x,
            self.box_pos.y,
            self.box_pos.z,
            self.box_dimensions.x,
            self.box_dimensions.y,
            self.box_dimensions.z,
            self.box_rotation.x,
            self.box_rotation.y,
            self.box_rotation.z,
            self.diffusion_width_rad,
            self.diffusion_depth_rad,
            self.particle_color.x,
            self.particle_color.y,
            self.particle_color.z,
            self.particle_color.w,
            self.particle_speed,
            self.particle_friction_coefficient,
            self.particle_size,
            self.particle_mass,
            self.particle_lifetime_sec,
        ]
    }
}
