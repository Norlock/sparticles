use crate::instance::particle::Particle;

pub trait Force {
    fn apply(&self, particle: &mut Particle, cycle_ms: u128);
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

    pub fn apply(&self, particle: &mut Particle, elapsed_ms: u128) {
        let forces_cycle_ms = elapsed_ms % self.duration_ms;

        for force in self.forces.iter() {
            force.apply(particle, forces_cycle_ms);
        }
    }
}
