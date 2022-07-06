use super::force::Force;
use crate::instance::particle::Particle;

/**
 * Builds up applying force form 0 to nx/ny over time.
 * max_(vx/vy) will determin the max (positive or negative) speed a particle in similar direction needs to have the force applied.
 */
pub struct AcceleratingForce {
    pub nx: f32,
    pub ny: f32,
    pub nz: f32,
    pub max_vx: f32,
    pub max_vy: f32,
    pub max_vz: f32,
    pub from_ms: u128,
    pub until_ms: u128,
}

const MS_PER_SEC: f32 = 1000.;

impl Force for AcceleratingForce {
    fn apply(&self, particle: &mut Particle, cycle_ms: u128) {
        if cycle_ms < self.from_ms || self.until_ms <= cycle_ms {
            return;
        }

        let acceleration = ((cycle_ms - self.from_ms) as f32 / MS_PER_SEC).powf(2.);
        let vx = self.nx * acceleration / particle.mass;
        let vy = self.ny * acceleration / particle.mass;
        let vz = self.nz * acceleration / particle.mass;

        let velocity = &mut particle.velocity;
        let new_vx = velocity.x + vx;
        let new_vy = velocity.y + vy;
        let new_vz = velocity.z + vz;

        if 0. < vx && 0. <= velocity.x {
            if new_vx <= self.max_vx {
                velocity.x += vx;
            }
        } else if vx < 0. && velocity.x <= 0. {
            if self.max_vx <= new_vx {
                velocity.x += vx;
            }
        } else {
            velocity.x += vx;
        }

        if 0. < vy && 0. <= velocity.y {
            if new_vy <= self.max_vy {
                velocity.y += vy;
            }
        } else if vy < 0. && velocity.y <= 0. {
            if self.max_vy <= new_vy {
                velocity.y += vy;
            }
        } else {
            velocity.y += vy;
        }

        if 0. < vz && 0. <= velocity.z {
            if new_vz <= self.max_vz {
                velocity.z += vz;
            }
        } else if vz < 0. && velocity.z <= 0. {
            if self.max_vz <= new_vz {
                velocity.z += vz;
            }
        } else {
            velocity.z += vz;
        }
    }
}
