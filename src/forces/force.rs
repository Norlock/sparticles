use crate::clock::Clock;
use crate::instance::particle::Particle;
use crate::life_cycle::LifeCycle;

pub trait Force {
    fn apply(&self, particle: &mut Particle, time: &LifeCycle);
}

pub struct ForceHandler {
    duration_ms: u128,
    forces: Vec<Box<dyn Force>>,
}

impl ForceHandler {
    pub fn new(duration_ms: u128) -> Self {
        Self {
            duration_ms,
            forces: Vec::new(),
        }
    }

    pub fn add(&mut self, force: Box<dyn Force>) {
        self.forces.push(force);
    }

    pub fn apply(&self, particle: &mut Particle, clock: &Clock) {
        let cycle_ms = clock.elapsed_ms() % self.duration_ms;

        let time = LifeCycle {
            cycle_ms,
            delta_sec: clock.delta_sec(),
        };

        for force in self.forces.iter() {
            force.apply(particle, &time);
        }
    }
}
