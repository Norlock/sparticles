use crate::instance::particle::Particle;
use crate::life_cycle::LifeCycle;
use cgmath::VectorSpace;

use crate::forces::force::Force;

pub struct GravitationalForce {
    /// In newton
    pub gravitational_force: f32,
    /// Use to exclude extreme gravitational pulls, e.g. 20.
    pub dead_zone: f32,
    pub mass: f32,
    pub from_ms: u128,
    pub until_ms: u128,
    pub start_pos: cgmath::Vector3<f32>,
    pub end_pos: cgmath::Vector3<f32>,
}

impl GravitationalForce {
    fn current_point(&self, force_cycle_ms: u128) -> cgmath::Vector3<f32> {
        let delta_current = force_cycle_ms - self.from_ms;
        let delta_end = self.until_ms - self.from_ms;

        let fraction = delta_current as f32 / delta_end as f32;
        self.start_pos.lerp(self.end_pos, fraction)
    }
}

impl Force for GravitationalForce {
    // Based on newton's law of universal gravity.
    // TODO convert to Einstein
    fn apply(&self, particle: &mut Particle, life_cycle: &LifeCycle) {
        if life_cycle.cycle_ms < self.from_ms || self.until_ms <= life_cycle.cycle_ms {
            return;
        }

        let gravitational_point = self.current_point(life_cycle.cycle_ms);

        let position = particle.position;
        let velocity = &mut particle.velocity;

        let particle_radius = particle.size / 2.;

        let particle_center_x = position.x + particle_radius;
        let particle_center_y = position.y + particle_radius;
        let particle_center_z = position.z + particle_radius;
        let x_distance = gravitational_point.x - particle_center_x;
        let y_distance = gravitational_point.y - particle_center_y;
        let z_distance = gravitational_point.z - particle_center_z;

        if x_distance.abs() < self.dead_zone
            && y_distance.abs() < self.dead_zone
            && z_distance.abs() < self.dead_zone
        {
            return;
        }

        let x_distance_pow = x_distance.powi(2);
        let y_distance_pow = y_distance.powi(2);
        let z_distance_pow = z_distance.powi(2);

        let distance_pow = x_distance_pow + y_distance_pow + z_distance_pow;

        let top_formula = self.gravitational_force * self.mass * particle.mass;
        let force = top_formula / distance_pow;

        let x_percentage = x_distance_pow / distance_pow;
        let y_percentage = y_distance_pow / distance_pow;
        let z_percentage = z_distance_pow / distance_pow;

        let vx = force * x_percentage / particle.mass;
        velocity.x += vx * x_distance.signum() * life_cycle.delta_sec;

        let vy = force * y_percentage / particle.mass;
        velocity.y += vy * y_distance.signum() * life_cycle.delta_sec;

        let vz = force * z_percentage / particle.mass;
        velocity.z += vz * z_distance.signum() * life_cycle.delta_sec;
    }
}
