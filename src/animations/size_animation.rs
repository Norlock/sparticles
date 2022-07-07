use crate::{animations::animation::Animate, instance::particle::Particle, life_cycle::LifeCycle};

pub struct SizeAnimation {
    pub start_size: f32,
    pub end_size: f32,
    pub from_ms: u128,
    pub until_ms: u128,
}

impl Animate for SizeAnimation {
    fn animate(&self, particle: &mut Particle, life_cycle: &LifeCycle) {
        if life_cycle.cycle_ms < self.from_ms || self.until_ms <= life_cycle.cycle_ms {
            return;
        }

        let delta_current = life_cycle.cycle_ms - self.from_ms;
        let delta_max = self.until_ms - self.from_ms;

        // calculate percent
        let fraction = delta_current as f32 / delta_max as f32;
        particle.size = self.start_size + fraction * (self.end_size - self.start_size);
    }
}
