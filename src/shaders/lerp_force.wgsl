// Includes declarations

struct LerpForce {
    force_x: f32,
    force_y: f32,
    force_z: f32,
    padding: f32,
}

@group(0) @binding(0) var<uniform> force: LerpForce; 
@group(1) @binding(0) var<storage, read_write> particles: array<Particle>;
@group(1) @binding(2) var<uniform> em: Emitter; 

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

    particle.velocity.x += force.force_x / particle.mass;
    particle.velocity.y += force.force_y / particle.mass;
    particle.velocity.z += force.force_z / particle.mass;

    particles[index] = particle;
}

