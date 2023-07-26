// Includes declarations 

struct GravitationalForce {
    gravitational_force: f32,
    dead_zone: f32,
    mass: f32,
    current_pos_x: f32,
    current_pos_y: f32,
    current_pos_z: f32,
}


@group(0) @binding(0) var<storage, read_write> particles: array<Particle>;
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

    if (particle.lifetime == 0.) {
        return;
    }

    let position = particle.position;
    let particle_radius = particle.size * particle.scale / 2.;

    let particle_center_x = position.x + particle_radius;
    let particle_center_y = position.y + particle_radius;
    let particle_center_z = position.z + particle_radius;
    let x_distance = force.current_pos_x - particle_center_x;
    let y_distance = force.current_pos_y - particle_center_y;
    let z_distance = force.current_pos_z - particle_center_z;

    if abs(x_distance) < force.dead_zone
        && abs(y_distance) < force.dead_zone
        && abs(z_distance) < force.dead_zone
    {
        return;
    }

    let x_distance_pow = pow(x_distance, 2.);
    let y_distance_pow = pow(y_distance, 2.);
    let z_distance_pow = pow(z_distance, 2.);

    let distance_pow = x_distance_pow + y_distance_pow + z_distance_pow;

    let top_formula = force.gravitational_force * force.mass * particle.mass;
    let force = top_formula / distance_pow;

    let x_percentage = x_distance_pow / distance_pow;
    let y_percentage = y_distance_pow / distance_pow;
    let z_percentage = z_distance_pow / distance_pow;

    let delta = 0.007;

    let vx = force * x_percentage / particle.mass;
    particle.velocity.x += vx * sign(x_distance) * delta;

    let vy = force * y_percentage / particle.mass;
    particle.velocity.y += vy * sign(y_distance) * delta;

    let vz = force * z_percentage / particle.mass;
    particle.velocity.z += vz * sign(z_distance) * delta;

    particles[index] = particle;
}

