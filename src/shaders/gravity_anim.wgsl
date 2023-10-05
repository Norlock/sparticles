// Includes declarations 
struct GravitationalForce {
    gravitational_force: f32,
    dead_zone: f32,
    mass: f32,
    current_pos_x: f32,
    current_pos_y: f32,
    current_pos_z: f32,
}

@group(0) @binding(1) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(2) var<uniform> em: Emitter; 
@group(1) @binding(0) var<uniform> force: GravitationalForce; 

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

    let position = particle.pos_size.xyz;
    let size = particle.pos_size.w;
    let mass = particle.vel_mass.w;
    let particle_radius = size / 2.;

    let particle_center = position + particle_radius;
    let current_pos = vec3<f32>(force.current_pos_x, force.current_pos_y, force.current_pos_z);
    let distance = current_pos - particle_center;

    if all(abs(distance) < force.dead_zone) {
        return;
    }

    let distance_pow_x = distance.x * distance.x;
    let distance_pow_y = distance.y * distance.y;
    let distance_pow_z = distance.z * distance.z;
    let len_pow = distance_pow_x + distance_pow_y + distance_pow_z;
    let top_formula = force.gravitational_force * force.mass * mass;
    let force = top_formula / len_pow;

    let percentage_x = distance_pow_x / len_pow;
    let percentage_y = distance_pow_y / len_pow;
    let percentage_z = distance_pow_z / len_pow;

    let vx = force * percentage_x;
    particle.vel_mass.x += vx * sign(distance.x) * em.delta_sec;

    let vy = force * percentage_y;
    particle.vel_mass.y += vy * sign(distance.y) * em.delta_sec;

    let vz = force * percentage_z;
    particle.vel_mass.z += vz * sign(distance.z) * em.delta_sec;

    particles[index] = particle;
}

