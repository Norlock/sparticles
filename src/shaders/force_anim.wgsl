// Includes declarations 
struct Force {
    vel_x: f32,
    vel_y: f32,
    vel_z: f32,
    mass: f32,
}

@group(0) @binding(1) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(2) var<uniform> em: Emitter; 
@group(1) @binding(0) var<uniform> force: Force; 

fn get_velocity(particle_vel: f32, particle_mass: f32, force_vel: f32, force_mass: f32) -> f32 {
    let particle_force = particle_vel * particle_mass; 
    let same_dir = sign(particle_vel) == sign(force_vel);

    if same_dir {
        if 0. <= force_vel {
            let delta_vel = max(force_vel - particle_vel, 0.);

            if 0. < delta_vel {
                let delta_force = delta_vel * force_mass * em.delta_sec;
                let possible_force = particle_force + delta_force;
                let possible_speed = possible_force / particle_mass;

                return min(possible_speed, force_vel);
            }
            return particle_vel;
        } else {
            let delta_vel = min(force_vel - particle_vel, 0.);

            if delta_vel < 0. {
                let delta_force = delta_vel * force_mass * em.delta_sec;
                let possible_force = particle_force + delta_force;
                let possible_speed = possible_force / particle_mass;

                return max(possible_speed, force_vel);
            }
            return particle_vel;
        }
    } else {
        let delta_force = force_vel * force_mass * em.delta_sec;
        let possible_force = particle_force + delta_force;
        let possible_speed = possible_force / particle_mass;

        if 0. < force_vel {
            return min(possible_speed, force_vel);
        } else {
            return max(possible_speed, force_vel);
        }
    }
}

@compute
@workgroup_size(128)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let particle_len = arrayLength(&particles);

    let index = global_invocation_id.x;

    if (particle_len <= index) {
        return;
    }

    var particle = particles[index];

    if (particle.lifetime == -1.) {
        return;
    }

    // e.g particle vel = 10, 0, 0, mass = 1
    // force vel = -5, 0, 0, mass = 1
    // delta vel = -15, 0, 0
    // applied force = -15, 0, 0
    // particle force = 10, 0, 0
    // delta_force = -5, 0, 0
    // todo even goede documentatie

    var particle_vel = particle.velocity;

    let force_vel = vec3<f32>(force.vel_x, force.vel_y, force.vel_z);
    let force_mass = force.mass * particle.size;
    
    particle_vel.x = get_velocity(particle_vel.x, particle.mass, force.vel_x, force_mass);
    particle_vel.y = get_velocity(particle_vel.y, particle.mass, force.vel_y, force_mass);
    particle_vel.z = get_velocity(particle_vel.z, particle.mass, force.vel_z, force_mass);

    particle.velocity = particle_vel;

    particles[index] = particle;
}

//    if 0. <= force_vel.x {
//        let delta_vel_x = max(force_vel.x - particle_vel.x, 0.);
//
//        if 0. < delta_vel_x {
//            let delta_x_force = delta_vel_x * applied_mass * em.delta_sec;
//            let possible_x_force = particle_force.x + delta_x_force;
//            let possible_x_speed = possible_x_force / particle.mass;
//
//            particle_vel.x = min(possible_x_speed, force_vel.x);
//        }
//    } else {
//        if sign(particle_vel.x) == sign(force_vel.x) {
//
//        }
//        let delta_vel_x = min(force_vel.x - particle_vel.x, 0.);
//
//        if delta_vel_x < 0. {
//            let delta_x_force = delta_vel_x * applied_mass * em.delta_sec;
//            let possible_x_force = particle_force.x + delta_x_force;
//            let possible_x_speed = possible_x_force / particle.mass;
//
//            particle_vel.x = max(possible_x_speed, force_vel.x);
//        }
//    }
//
//    if 0. <= force_vel.y {
//        let delta_vel_y = max(force_vel.y - particle_vel.y, 0.);
//
//        if 0. < delta_vel_y {
//            let delta_y_force = delta_vel_y * applied_mass * em.delta_sec;
//            let possible_y_force = particle_force.y + delta_y_force;
//            let possible_y_speed = possible_y_force / particle.mass;
//
//            particle_vel.y = min(possible_y_speed, force_vel.y);
//        }
//    } else {
//        let delta_vel_y = min(force_vel.y - particle_vel.y, 0.);
//
//        if delta_vel_y < 0. {
//            let delta_y_force = delta_vel_y * applied_mass * em.delta_sec;
//            let possible_y_force = particle_force.y + delta_y_force;
//            let possible_y_speed = possible_y_force / particle.mass;
//
//            particle_vel.y = max(possible_y_speed, force_vel.y);
//        }
//    }
//
//    if 0. <= force_vel.z {
//        let delta_vel_z = max(force_vel.z - particle_vel.z, 0.);
//
//        if 0. < delta_vel_z {
//            let delta_z_force = delta_vel_z * applied_mass * em.delta_sec;
//            let possible_z_force = particle_force.z + delta_z_force;
//            let possible_z_speed = possible_z_force / particle.mass;
//
//            particle_vel.z = min(possible_z_speed, force_vel.z);
//        }
//    } else {
//        let delta_vel_z = min(force_vel.z - particle_vel.z, 0.);
//
//        if delta_vel_z < 0. {
//            let delta_z_force = delta_vel_z * applied_mass * em.delta_sec;
//            let possible_z_force = particle_force.z + delta_z_force;
//            let possible_z_speed = possible_z_force / particle.mass;
//
//            particle_vel.z = max(possible_z_speed, force_vel.z);
//        }
//    }
//
