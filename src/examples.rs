use crate::forces::accelerating_force::AcceleratingForce;
use crate::forces::force::ForceHandler;
use crate::forces::lerp_force::LerpForce;
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

    force_handler.add(Box::new(LerpForce {
        from_ms: 2_000,
        until_ms: 3_000,
        min_nx: 0.,
        min_ny: -10.,
        min_nz: 0.,
        max_nx: 0.,
        max_ny: -40.,
        max_nz: 0.,
    }));

    emitter.force_handler = Some(force_handler);

    emitter
}
