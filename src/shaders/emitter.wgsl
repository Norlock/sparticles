@group(0) @binding(0) var<storage, read> particles_src : array<Particle>;
@group(0) @binding(1) var<storage, read_write> particles_dst : array<Particle>;
@group(0) @binding(2) var<uniform> em: Emitter; 

fn is_decayed(par: Particle) -> bool {
    return em.particle_lifetime < par.lifetime;
}

fn spawn_particle(index: u32) {
    var particle = particles_src[index];

    let particle_position = vec3<f32>(
        em.pos_x,
        em.pos_y,
        em.pos_z,
    );

    let particle_color = vec4<f32>(
        em.particle_color_r,
        em.particle_color_g,
        em.particle_color_b,
        em.particle_color_a,
    );

    let velocity = vec3<f32>(
        em.particle_velocity_x,
        em.particle_velocity_y,
        em.particle_velocity_z,
    );

    particle.position = particle_position;
    particle.velocity = velocity;
    particle.color = particle_color;
    particle.size = em.particle_size;
    particle.lifetime = 0.;
    particle.mass = em.particle_mass;

    particles_dst[index] = particle;
}

@compute
@workgroup_size(128)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let particle_len = arrayLength(&particles_src);
    let index = global_invocation_id.x;

    if (particle_len <= index) {
        return;
    }

    if (u32(em.spawn_from) <= index && index < u32(em.spawn_until)) {
        spawn_particle(index);
        return;
    } 

    var particle = particles_src[index];
    particle.lifetime += em.delta_sec;

    if (is_decayed(particle)) {
        if (particle.lifetime != -1.) {
            particle.lifetime = -1.;
            particles_dst[index] = particle;
        }
        return;
    }

    let friction_coefficient = em.particle_friction_coefficient;
    particle.velocity *= friction_coefficient;
    particle.position += particle.velocity * em.delta_sec;

    particles_dst[index] = particle;
}

