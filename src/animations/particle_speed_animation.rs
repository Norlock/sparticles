use crate::life_cycle::LifeCycle;

use super::emitter_animation::EmitterAnimate;
use super::emitter_animation::EmitterAnimationData;

pub struct ParticleSpeedAnimation {
    pub from_ms: u128,
    pub until_ms: u128,
    pub from_speed: f32,
    pub to_speed: f32,
}

impl EmitterAnimate for ParticleSpeedAnimation {
    fn animate(&mut self, data: &mut EmitterAnimationData, life_cycle: &LifeCycle) {
        if life_cycle.cycle_ms < self.from_ms || self.until_ms <= life_cycle.cycle_ms {
            return;
        }

        let delta_current = life_cycle.cycle_ms - self.from_ms;
        let delta_max = self.until_ms - self.from_ms;

        // calculate percent from 0..1
        let fraction = delta_current as f32 / delta_max as f32;
        data.particle_speed = self.from_speed + fraction * (self.to_speed - self.from_speed);
    }
}
