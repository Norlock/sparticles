use crate::clock::Clock;
use crate::instance::angles::Angles;
use crate::instance::emitter::Bounds;
use crate::instance::emitter::EmitterSize;
use crate::life_cycle::LifeCycle;

pub trait EmitterAnimate {
    fn animate(&mut self, data: &mut EmitterAnimationData, life_cycle: &LifeCycle);
}

pub struct EmitterAnimationData<'a> {
    pub emitter_position: cgmath::Vector3<f32>,
    pub emitter_size: EmitterSize,
    pub emission_offset: f32,

    pub particles_per_emission: u32,
    pub delay_between_emission_ms: u32,
    pub angle_degrees: Angles,

    pub particle_speed: f32,

    /// Initial spread factor x,y / z
    pub diffusion_degrees: Angles,

    pub bounds: &'a Vec<Bounds>,
}

pub struct EmitterAnimationHandler {
    animations: Vec<Box<dyn EmitterAnimate>>,
    duration_ms: u128,
}

impl EmitterAnimationHandler {
    pub fn new(animations: Vec<Box<dyn EmitterAnimate>>, duration_ms: u128) -> Self {
        Self {
            animations,
            duration_ms,
        }
    }

    pub fn animate(&mut self, data: &mut EmitterAnimationData, clock: &Clock) {
        let cycle_ms = clock.elapsed_ms() % self.duration_ms;

        let life_cycle = LifeCycle {
            cycle_ms,
            delta_sec: clock.delta_sec(),
        };

        for animation in self.animations.iter_mut() {
            animation.animate(data, &life_cycle);
        }
    }
}
