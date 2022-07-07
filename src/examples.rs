use crate::forces::accelerating_force::AcceleratingForce;
use crate::forces::force::ForceHandler;
use crate::forces::gravitational_force::GravitationalForce;
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
        nx: 50.,
        ny: 50.,
        nz: 0.,
        max_vx: 300.,
        max_vy: 300.,
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

    force_handler.add(Box::new(GravitationalForce {
        from_ms: 0,
        until_ms: 5000,
        gravitational_force: 0.015,
        dead_zone: 10.,
        mass: 1000_000.,
        start_pos: cgmath::Vector3::new(-1., 1., 0.),
        end_pos: cgmath::Vector3::new(1., -1., 0.),
    }));

    //force_handler.add(Box::new(GravitationalForce {
    //from_ms: 5000,
    //until_ms: 10000,
    //gravitation_force: -0.4,
    //dead_zone: 20.,
    //mass: 1000.,
    //start: Point(400., 800.),
    //end: Point(400., 400.),
    //}));

    emitter.force_handler = Some(force_handler);

    emitter
}
