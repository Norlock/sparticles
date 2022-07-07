use crate::{
    animations::animation::Animate, instance::particle::Particle, life_cycle::LifeCycle,
    random::gen_dyn_range,
};
use rand::thread_rng;

pub struct StrayAnimation {
    from_ms: u128,
    until_ms: u128,
    stray_radians: f32,
}

impl StrayAnimation {
    /// Between 1 and 50 for strayness_number is advised
    pub fn new(from_ms: u128, until_ms: u128, strayness_number: f32) -> Self {
        Self {
            from_ms,
            until_ms,
            stray_radians: strayness_number.to_radians(),
        }
    }
}

impl Animate for StrayAnimation {
    fn animate(&self, data: &mut Particle, life_cycle: &LifeCycle) {
        if life_cycle.cycle_ms < self.from_ms || self.until_ms <= life_cycle.cycle_ms {
            return;
        }

        let velocity = &mut data.velocity;

        let speed_squared = velocity.x.powi(2) + velocity.y.powi(2) + velocity.z.powi(2);
        if speed_squared == 0. {
            return;
        }

        let speed = speed_squared.sqrt();

        let mut rng = thread_rng();
        let mut stray_factor = || gen_dyn_range(&mut rng, self.stray_radians) / 2.;

        let cos_x = velocity.x / speed + stray_factor();
        let cos_y = velocity.y / speed + stray_factor();
        let cos_z = velocity.z / speed + stray_factor();

        let new_vx = speed * cos_x;
        let new_vy = speed * cos_y;
        let new_vz = speed * cos_z;

        let mut new_velocity = cgmath::Vector3::new(new_vx, new_vy, new_vz);

        equalize_total_speed(speed_squared, &mut new_velocity);

        velocity.x = new_velocity.x;
        velocity.y = new_velocity.y;
        velocity.z = new_velocity.z;
    }
}

fn equalize_total_speed(speed_squared: f32, new: &mut cgmath::Vector3<f32>) {
    let new_vx_squared = new.x.powi(2);
    let new_vy_squared = new.y.powi(2);
    let new_vz_squared = new.z.powi(2);
    let new_speed_squared = new_vx_squared + new_vy_squared + new_vz_squared;

    let scale_factor = speed_squared / new_speed_squared;
    new.x = (new_vx_squared * scale_factor).sqrt() * new.x.signum();
    new.y = (new_vy_squared * scale_factor).sqrt() * new.y.signum();
    new.z = (new_vz_squared * scale_factor).sqrt() * new.z.signum();
}
