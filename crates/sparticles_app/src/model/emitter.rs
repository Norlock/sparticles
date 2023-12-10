use super::Clock;
use crate::loader::{Model, BUILTIN_ID, CIRCLE_MAT_ID, CIRCLE_MESH_ID};
use crate::model::state::FastFetch;
use crate::traits::{FromRGB, HandleAngles};
use crate::util::ID;
use async_std::sync::{Mutex, RwLock};
use glam::{f32::Vec3, f32::Vec4};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

const PARTICLE_BUFFER_SIZE: u64 = 26 * 4;

pub struct EmitSpawnOptions {
    pub spawn_count: u32,
    pub spawn_delay_sec: f32,
    pub particle_lifetime_sec: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Boundry(f32, f32);

impl Boundry {
    pub fn new(min: f32, max: f32) -> Self {
        assert!(min <= max, "Min must be smaller than max");
        Boundry(min, max)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshRef {
    pub collection_id: ID,
    pub mesh_id: ID,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialRef {
    pub collection_id: ID,
    pub material_id: ID,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmitterUniform {
    pub id: ID,
    spawn_from: u32,
    spawn_until: u32,
    spawn_batches_count: u32,
    elapsed_sec: f32,
    delta_sec: f32,
    iteration: u32,

    pub spawn_count: u32,
    pub spawn_delay_sec: f32,

    pub box_position: Vec3,
    /// width, height, depth
    pub box_dimensions: Vec3,
    /// yaw, pitch, roll
    pub box_rotation: Vec3,

    /// Diffusion emission in radians
    pub diff_width: f32,
    /// Diffusion emission in radians
    pub diff_depth: f32,

    pub hdr_mul: f32,
    pub particle_color: Vec4,
    pub particle_friction_coefficient: f32,
    pub particle_speed: Boundry,
    pub particle_size: Boundry,
    /// Mass per size 1
    pub particle_material_mass: f32,
    pub particle_lifetime_sec: f32,
    pub mesh: MeshRef,
    pub material: MaterialRef,
}

pub struct EmitterSettings {
    pub id: String,
    pub spawn_count: u32,
    pub spawn_delay_sec: f32,
    pub particle_lifetime_sec: f32,
    pub recreate: bool,

    pub box_position: Vec3,
    pub box_dimensions: Vec3,
    pub box_rotation_deg: Vec3,

    pub diff_width_deg: f32,
    pub diff_depth_deg: f32,

    pub particle_speed_min: f32,
    pub particle_speed_max: f32,
    pub particle_size_min: f32,
    pub particle_size_max: f32,

    pub particle_color: Vec4,
    pub hdr_mul: f32,
}

impl EmitterUniform {
    pub fn new(id: ID) -> Self {
        let spawn_count: u32 = 6;
        let particle_lifetime_sec: f32 = 6.;
        let spawn_delay_sec: f32 = 0.5;

        let spawn_batches_count = (particle_lifetime_sec / spawn_delay_sec).ceil() as u32;

        let box_pos = Vec3::ZERO;
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

            box_position: box_pos,
            box_dimensions,
            box_rotation,

            hdr_mul: 1.0,

            diff_width: diffusion_width_rad,
            diff_depth: diffusion_depth_rad,

            particle_material_mass: 5.,
            particle_lifetime_sec,
            particle_speed: Boundry(10., 15.),
            particle_size: Boundry(0.1, 0.15),
            particle_friction_coefficient: 0.99,
            particle_color: Vec4::from_rgb(0, 255, 0),

            iteration: 1000,
            elapsed_sec: 0.,
            delta_sec: 0.0,

            material: MaterialRef {
                collection_id: BUILTIN_ID.to_string(),
                material_id: CIRCLE_MAT_ID.to_string(),
            },
            mesh: MeshRef {
                collection_id: BUILTIN_ID.to_string(),
                mesh_id: CIRCLE_MESH_ID.to_string(),
            },
        }
    }

    pub fn update_settings(&mut self, settings: &EmitterSettings) {
        self.box_rotation = settings.box_rotation_deg.to_radians();
        self.box_dimensions = settings.box_dimensions;
        self.box_position = settings.box_position;

        self.diff_width = settings.diff_width_deg.to_radians();
        self.diff_depth = settings.diff_depth_deg.to_radians();

        self.particle_speed.0 = settings.particle_speed_min;
        self.particle_speed.1 = settings.particle_speed_max;

        self.particle_size.0 = settings.particle_size_min;
        self.particle_size.1 = settings.particle_size_max;

        self.particle_color = settings.particle_color;
        self.hdr_mul = settings.hdr_mul;

        if settings.recreate {
            self.spawn_count = settings.spawn_count;
            self.spawn_delay_sec = settings.spawn_delay_sec;
            self.particle_lifetime_sec = settings.particle_lifetime_sec;
        }
    }

    pub fn create_settings(&self) -> EmitterSettings {
        EmitterSettings {
            id: self.id.to_string(),
            spawn_count: self.spawn_count,
            spawn_delay_sec: self.spawn_delay_sec,
            box_position: self.box_position,
            box_dimensions: self.box_dimensions,
            box_rotation_deg: self.box_rotation.to_degrees(),
            diff_width_deg: self.diff_width.to_degrees(),
            diff_depth_deg: self.diff_depth.to_degrees(),
            particle_lifetime_sec: self.particle_lifetime_sec,
            particle_color: self.particle_color,
            hdr_mul: self.hdr_mul,
            particle_speed_min: self.particle_speed.0,
            particle_speed_max: self.particle_speed.1,
            particle_size_min: self.particle_size.0,
            particle_size_max: self.particle_size.1,

            recreate: false,
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

    pub async fn create_buffer_content(
        &self,
        collection: &Arc<RwLock<HashMap<String, Model>>>,
    ) -> Vec<f32> {
        let collection = &collection.read().await;
        let mesh = collection.get_mesh(&self.mesh);
        let particle_model = mesh.model.to_cols_array();

        [
            &[
                self.delta_sec,
                self.elapsed_sec,
                self.spawn_from as f32,
                self.spawn_until as f32,
                self.box_position.x,
                self.box_position.y,
                self.box_position.z,
                self.box_dimensions.x,
                self.box_dimensions.y,
                self.box_dimensions.z,
                self.box_rotation.x,
                self.box_rotation.y,
                self.box_rotation.z,
                self.diff_width,
                self.diff_depth,
                0., // padding
            ],
            particle_model.as_slice(),
            &[
                self.particle_color.x * self.hdr_mul,
                self.particle_color.y * self.hdr_mul,
                self.particle_color.z * self.hdr_mul,
                self.particle_color.w,
                self.particle_speed.0,
                self.particle_speed.1,
                self.particle_size.0,
                self.particle_size.1,
                self.particle_friction_coefficient,
                self.particle_material_mass,
                self.particle_lifetime_sec,
                0., // padding
            ],
        ]
        .concat()
    }
}
