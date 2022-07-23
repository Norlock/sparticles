use super::animation::Animate;
use crate::{
    instance::{color::Color, particle::Particle},
    life_cycle::LifeCycle,
};

pub struct DuoColorAnimation {
    pub color_from: Color,
    pub color_to: Color,
    pub from_ms: u128,
    pub until_ms: u128,
}

impl Animate for DuoColorAnimation {
    fn animate(&self, particle: &mut Particle, life_cycle: &LifeCycle) {
        if life_cycle.cycle_ms < self.from_ms || self.until_ms <= life_cycle.cycle_ms {
            return;
        }

        let delta_current = life_cycle.cycle_ms - self.from_ms;
        let delta_max = self.until_ms - self.from_ms;

        // calculate percent from 0..1
        let fraction = delta_current as f32 / delta_max as f32;
        particle.color.r = self.color_from.r + fraction * (self.color_to.r - self.color_from.r);
        particle.color.g = self.color_from.g + fraction * (self.color_to.g - self.color_from.g);
        particle.color.b = self.color_from.b + fraction * (self.color_to.b - self.color_from.b);
        particle.color.a = self.color_from.a + fraction * (self.color_to.a - self.color_from.a);

        if 1. < particle.color.r {
            println!("r: {} ", particle.color.r);
        } else if 1. < particle.color.g {
            println!("g: {} ", particle.color.g);
        } else if 1. < particle.color.b {
            println!("b: {} ", particle.color.b);
        } else if 1. < particle.color.a {
            println!("a: {} ", particle.color.a);
        }
    }
}
