use crate::traits::{CalculateBufferSize, HandleAngles};
use egui_wgpu::wgpu;
use glam::{Vec2, Vec3};
use std::num::NonZeroU64;

impl HandleAngles for Vec3 {
    fn to_degrees(&self) -> Self {
        let x = self.x.to_degrees();
        let y = self.y.to_degrees();
        let z = self.z.to_degrees();

        Self { x, y, z }
    }

    fn to_radians(&self) -> Self {
        let x = self.x.to_radians();
        let y = self.y.to_radians();
        let z = self.z.to_radians();

        Self { x, y, z }
    }
}

impl HandleAngles for Vec2 {
    fn to_degrees(&self) -> Self {
        let x = self.x.to_degrees();
        let y = self.y.to_degrees();

        Self { x, y }
    }

    fn to_radians(&self) -> Self {
        let x = self.x.to_radians();
        let y = self.y.to_radians();

        Self { x, y }
    }
}

impl CalculateBufferSize for Vec<f32> {
    fn cal_buffer_size(&self) -> Option<NonZeroU64> {
        wgpu::BufferSize::new(self.len() as u64 * 4)
    }
}

impl CalculateBufferSize for [f32] {
    fn cal_buffer_size(&self) -> Option<NonZeroU64> {
        wgpu::BufferSize::new(self.len() as u64 * 4)
    }
}
