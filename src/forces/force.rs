use crate::instance::particle::Particle;

pub trait Force {
    fn apply(&self, particle: &mut Particle, force_cycle_ms: u128);
}

pub struct ForceHandler {
    duration_ms: u128,
    forces: Vec<Box<dyn Force>>,
}

pub struct ForceData {
    cycle_ms: u128,
    delta_sec: f32,
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

    pub fn apply(&self, data: &mut Particle, elapsed_ms: u128) {
        let forces_cycle_ms = elapsed_ms % self.duration_ms;

        for force in self.forces.iter() {
            force.apply(data, forces_cycle_ms);
        }
    }
}
