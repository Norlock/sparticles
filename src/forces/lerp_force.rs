use super::force::Force;
use crate::{instance::particle::Particle, life_cycle::LifeCycle};

pub struct LerpForce {
    pub min_nx: f32,
    pub min_ny: f32,
    pub min_nz: f32,
    pub max_nx: f32,
    pub max_ny: f32,
    pub max_nz: f32,
    pub from_ms: u128,
    pub until_ms: u128,
}

impl Force for LerpForce {
    fn apply(&self, particle: &mut Particle, time: &LifeCycle) {
        if time.cycle_ms < self.from_ms || self.until_ms <= time.cycle_ms {
            return;
        }

        let delta_current = time.cycle_ms - self.from_ms;
        let delta_max = self.until_ms - self.from_ms;

        let fraction = delta_current as f32 / delta_max as f32;
        let velocity = &mut particle.velocity;

        velocity.x +=
            (self.min_nx + fraction * (self.max_nx - self.min_nx)) / particle.mass * time.delta_sec;
        velocity.y +=
            (self.min_ny + fraction * (self.max_ny - self.min_ny)) / particle.mass * time.delta_sec;
        velocity.z +=
            (self.min_nz + fraction * (self.max_nz - self.min_nz)) / particle.mass * time.delta_sec;
    }
}
