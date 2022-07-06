use crate::forces::accelerating_force::AcceleratingForce;
use crate::forces::force::ForceHandler;
use crate::instance::emitter::Emitter;
use std::time::Duration;

pub fn simple_emitter() -> Emitter {
    let mut emitter = Emitter::default();

    let forces_length = Duration::from_secs(6).as_millis();
    let mut force_handler = ForceHandler::new(forces_length);

    force_handler.add(Box::new(AcceleratingForce {
        from_ms: 0,
        until_ms: 1000,
        nx: 2.,
        ny: 2.,
        nz: 0.,
        max_vx: 20.,
        max_vy: 20.,
        max_vz: 0.,
    }));

    emitter.force_handler = Some(force_handler);

    emitter
}
