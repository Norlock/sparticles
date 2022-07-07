use crate::clock::Clock;
use crate::{instance::particle::Particle, life_cycle::LifeCycle};

//pub struct AnimationTime {
//pub cycle_ms: u32,
//pub total_ms: u128,
//pub delta_sec: f32,
//}

pub trait Animate {
    fn animate(&self, data: &mut Particle, life_cycle: &LifeCycle);
}

pub struct AnimationHandler {
    duration_ms: u128,
    animations: Vec<Box<dyn Animate>>,
}

impl AnimationHandler {
    pub fn new(duration_ms: u128) -> Self {
        Self {
            duration_ms,
            animations: Vec::new(),
        }
    }

    pub fn add(&mut self, animation: Box<dyn Animate>) {
        self.animations.push(animation);
    }

    pub fn apply(&self, particle: &mut Particle, clock: &Clock) {
        let cycle_ms = (clock.elapsed_ms() - particle.spawned_at) % self.duration_ms;

        let time = LifeCycle {
            cycle_ms,
            delta_sec: clock.delta_sec(),
        };

        for animationn in self.animations.iter() {
            animationn.animate(particle, &time);
        }
    }
}
