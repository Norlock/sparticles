use super::emitter_animation::EmitterAnimate;
use super::emitter_animation::EmitterAnimationData;
use crate::life_cycle::LifeCycle;

pub struct DiffusionAnimation {
    pub from_ms: u128,
    pub until_ms: u128,
    pub start_elevation_degrees: f32,
    pub end_elevation_degrees: f32,
    pub start_bearing_degrees: f32,
    pub end_bearing_degrees: f32,
}

impl EmitterAnimate for DiffusionAnimation {
    fn animate(&mut self, data: &mut EmitterAnimationData, life_cycle: &LifeCycle) {
        if life_cycle.cycle_ms < self.from_ms || self.until_ms <= life_cycle.cycle_ms {
            return;
        }

        let diffusion = &mut data.diffusion_degrees;
        let delta_current = life_cycle.cycle_ms - self.from_ms;
        let delta_max = self.until_ms - self.from_ms;

        // calculate percent
        let fraction = delta_current as f32 / delta_max as f32;
        diffusion.elevation = self.start_elevation_degrees
            + fraction * (self.end_elevation_degrees - self.start_elevation_degrees);
        diffusion.bearing = self.start_bearing_degrees
            + fraction * (self.end_bearing_degrees - self.start_bearing_degrees);
    }
}
