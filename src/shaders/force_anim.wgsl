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
        if 0. < force_vel {
            let delta_vel = max(force_vel - particle_vel, 0.);

            if 0. < delta_vel {
                let delta_force = delta_vel * force_mass * em.delta_sec;
                let possible_force = particle_force + delta_force;
                let possible_speed = possible_force / particle_mass;

                return min(possible_speed, force_vel);
            }
        } else if force_vel < 0. {
            let delta_vel = min(force_vel - particle_vel, 0.);

            if delta_vel < 0. {
                let delta_force = delta_vel * force_mass * em.delta_sec;
                let possible_force = particle_force + delta_force;
                let possible_speed = possible_force / particle_mass;

                return max(possible_speed, force_vel);
            }
        } 
    } else {
        let delta_force = force_vel * force_mass * em.delta_sec;
        let possible_force = particle_force + delta_force;
        let possible_speed = possible_force / particle_mass;

        if 0. < force_vel {
            return min(possible_speed, force_vel);
        } else if force_vel < 0. {
            return max(possible_speed, force_vel);
        }
    }

    return particle_vel;
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

    var particle_vel = particle.velocity;

    let force_vel = vec3<f32>(force.vel_x, force.vel_y, force.vel_z);
    let surface_particle = pi() * pow(particle.size, 2.0);
    let surface_sample = pi();
    let surface_scale = surface_particle / surface_sample;

    let applied_mass = force.mass * surface_scale;
    
    particle_vel.x = get_velocity(particle_vel.x, particle.mass, force.vel_x, applied_mass);
    particle_vel.y = get_velocity(particle_vel.y, particle.mass, force.vel_y, applied_mass);
    particle_vel.z = get_velocity(particle_vel.z, particle.mass, force.vel_z, applied_mass);

    particle.velocity = particle_vel;

    particles[index] = particle;
}
