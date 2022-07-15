use super::emitter_animation::EmitterAnimate;
use super::emitter_animation::EmitterAnimationData;
use crate::life_cycle::LifeCycle;

pub struct SwayAnimation {
    pub from_ms: u128,
    pub until_ms: u128,
    pub start_elevation_degrees: f32,
    pub end_elevation_degrees: f32,
    pub start_bearing_degrees: f32,
    pub end_bearing_degrees: f32,
}

impl EmitterAnimate for SwayAnimation {
    fn animate(&mut self, data: &mut EmitterAnimationData, life_cycle: &LifeCycle) {
        if life_cycle.cycle_ms < self.from_ms || self.until_ms <= life_cycle.cycle_ms {
            return;
        }

        let emit_angles = &mut data.angle_degrees;
        let delta_current = life_cycle.cycle_ms - self.from_ms;
        let delta_max = self.until_ms - self.from_ms;

        // calculate percent
        let fraction = delta_current as f32 / delta_max as f32;
        emit_angles.elevation = self.start_elevation_degrees
            + fraction * (self.end_elevation_degrees - self.start_elevation_degrees);
        emit_angles.bearing = self.start_bearing_degrees
            + fraction * (self.end_bearing_degrees - self.start_bearing_degrees);
    }
}
