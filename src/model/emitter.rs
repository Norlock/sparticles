use super::{Clock, EmitterGuiState};
use crate::{
    math::{SparVec3, SparVec4},
    traits::{FromRGB, HandleAngles},
};
use glam::{Vec3, Vec4};
use serde::{Deserialize, Serialize};

const PARTICLE_BUFFER_SIZE: u64 = 14 * 4;

pub struct EmitSpawnOptions {
    pub spawn_count: u32,
    pub spawn_delay_sec: f32,
    pub particle_lifetime_sec: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Range(f32, f32);

impl Range {
    pub fn new(min: f32, max: f32) -> Self {
        assert!(min <= max, "Min must be smaller than max");
        Range(min, max)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmitterUniform {
    pub id: String,
    spawn_from: u32,
    spawn_until: u32,
    spawn_batches_count: u32,
    elapsed_sec: f32,
    delta_sec: f32,
    iteration: u32,

    pub spawn_count: u32,
    pub spawn_delay_sec: f32,

    pub box_pos: SparVec3,
    /// width, height, depth
    pub box_dimensions: SparVec3,
    /// yaw, pitch, roll
    pub box_rotation: SparVec3,

    /// Diffusion emission in radians
    pub diff_width: f32,
    /// Diffusion emission in radians
    pub diff_depth: f32,

    pub particle_color: SparVec4,
    pub particle_friction_coefficient: f32,
    pub particle_speed: Range,
    pub particle_size: Range,
    /// Mass per size 1
    pub particle_material_mass: f32,
    pub particle_lifetime_sec: f32,
    pub texture_image: String,
}

impl EmitterUniform {
    pub fn new(id: String) -> Self {
        let spawn_count: u32 = 6;
        let particle_lifetime_sec: f32 = 6.;
        let spawn_delay_sec: f32 = 0.5;

        let spawn_batches_count = (particle_lifetime_sec / spawn_delay_sec).ceil() as u32;

        let box_pos = Vec3::ZERO.into();
        let box_dimensions = [1., 0.5, 1.].into();
        let box_rotation = [45f32.to_radians(), 0., 0.].into();

        let diffusion_width_rad = 15f32.to_radians();
        let diffusion_depth_rad = 15f32.to_radians();

        Self {
            id,
            spawn_delay_sec: 0.5,
            spawn_from: 0,
            spawn_until: 0,
            spawn_count,
            spawn_batches_count,

            box_pos,
            box_dimensions,
            box_rotation,

            diff_width: diffusion_width_rad,
            diff_depth: diffusion_depth_rad,

            particle_material_mass: 5.,
            particle_lifetime_sec,
            particle_speed: Range(10., 15.),
            particle_size: Range(0.1, 0.15),
            particle_friction_coefficient: 0.99,
            particle_color: Vec4::from_rgb(0, 255, 0).into(),

            iteration: 1000,
            elapsed_sec: 0.,
            delta_sec: 0.0,
            texture_image: "1x1.png".to_string(),
        }
    }

    pub fn process_gui(&mut self, gui: &EmitterGuiState) {
        *self.box_rotation = gui.box_rotation_deg.to_radians();
        *self.box_dimensions = gui.box_dimensions;

        self.diff_width = gui.diff_width_deg.to_radians();
        self.diff_depth = gui.diff_depth_deg.to_radians();

        self.particle_speed.0 = gui.particle_speed_min;
        self.particle_speed.1 = gui.particle_speed_max;

        self.particle_size.0 = gui.particle_size_min;
        self.particle_size.1 = gui.particle_size_max;

        if gui.recreate {
            self.spawn_count = gui.spawn_count;
            self.spawn_delay_sec = gui.spawn_delay_sec;
            self.particle_lifetime_sec = gui.particle_lifetime_sec;
        }
    }

    pub fn create_gui(&self) -> EmitterGuiState {
        EmitterGuiState {
            spawn_count: self.spawn_count,
            spawn_delay_sec: self.spawn_delay_sec,
            particle_lifetime_sec: self.particle_lifetime_sec,
            recreate: false,
            box_position: *self.box_rotation,
            box_dimensions: *self.box_dimensions,
            box_rotation_deg: self.box_rotation.to_degrees(),
            diff_width_deg: self.diff_width.to_degrees(),
            diff_depth_deg: self.diff_depth.to_degrees(),
            particle_speed_min: self.particle_speed.0,
            particle_speed_max: self.particle_speed.1,
            particle_size_min: self.particle_size.0,
            particle_size_max: self.particle_size.1,
        }
    }

    pub fn update(&mut self, clock: &Clock) {
        self.delta_sec = clock.delta_sec();
        self.elapsed_sec = clock.elapsed_sec();

        let new_iteration = (self.elapsed_sec / self.spawn_delay_sec) as u32;
        let current_batch = new_iteration % self.spawn_batches_count;

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

    pub fn get_spawn_options(&self) -> EmitSpawnOptions {
        EmitSpawnOptions {
            spawn_count: self.spawn_count,
            spawn_delay_sec: self.spawn_delay_sec,
            particle_lifetime_sec: self.particle_lifetime_sec,
        }
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
            self.diff_width,
            self.diff_depth,
            self.particle_color.x,
            self.particle_color.y,
            self.particle_color.z,
            self.particle_color.w,
            self.particle_speed.0,
            self.particle_speed.1,
            self.particle_size.0,
            self.particle_size.1,
            self.particle_friction_coefficient,
            self.particle_material_mass,
            self.particle_lifetime_sec,
        ]
    }
}
