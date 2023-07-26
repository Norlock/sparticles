// Includes declarations

@group(0) @binding(0) var<storage, read_write> particles_dst : array<Particle>;
@group(0) @binding(1) var<storage, read> particles_src : array<Particle>;
@group(0) @binding(2) var<uniform> em: Emitter; 

// See stray anim for explaination
fn create_spread_z(input_random: f32, vel: vec3<f32>) -> vec3<f32> {
    let spow_x = em.particle_speed_x * abs(vel.x);
    let spow_y = em.particle_speed_y * abs(vel.y);
    let spow_z = em.particle_speed_z * abs(vel.z);

    let pow_x = abs(spow_x);
    let pow_y = abs(spow_y);

    let before_total = pow_x + pow_y + abs(spow_z);

    let randomize = em.spread_z * before_total;
    let stray_z = gen_dyn_range(input_random * 1.13, randomize, em.elapsed_sec);

    let new_spow_z = spow_z + stray_z;

    let after_total = pow_x + pow_y + abs(new_spow_z);

    let scale_factor = before_total / after_total;

    let vx = sqrt(pow_x * scale_factor) * sign(spow_x);
    let vy = sqrt(pow_y * scale_factor) * sign(spow_y);
    let vz = sqrt(abs(new_spow_z) * scale_factor) * sign(new_spow_z);

    return vec3<f32>(vx, vy, vz); 
}

// See stray anim for explaination
fn create_spread_xy(input_random: f32) -> vec3<f32> {
    let spow_x = em.particle_speed_x * abs(em.particle_speed_x);
    let spow_y = em.particle_speed_y * abs(em.particle_speed_y);

    let before_total = abs(spow_x) + abs(spow_y);

    let randomize = em.spread_xy * before_total;

    let stray_x = gen_dyn_range(input_random * 1.13, randomize, em.elapsed_sec);
    let stray_y = gen_dyn_range(input_random * 1.313, randomize, em.elapsed_sec);

    let new_spow_x = spow_x + stray_x;
    let new_spow_y = spow_y + stray_y;

    let after_total = abs(new_spow_x) + abs(new_spow_y);

    let scale_factor = before_total / after_total;

    let vx = sqrt(abs(new_spow_x) * scale_factor) * sign(new_spow_x);
    let vy = sqrt(abs(new_spow_y) * scale_factor) * sign(new_spow_y);

    return vec3<f32>(vx, vy, em.particle_speed_z); 
}

fn pitch_matrix() -> mat3x3<f32> {
    let s = sin(em.pitch);
    let c = cos(em.pitch);

    return mat3x3<f32>(
        vec3<f32>(c, s, 0.),
        vec3<f32>(-s, c, 0.),
        vec3<f32>(0., 0., 1.),
    );
}

fn roll_matrix() -> mat3x3<f32> {
    let s = sin(em.roll);
    let c = cos(em.roll);

    return mat3x3<f32>(
        vec3<f32>(1., 0., 0.),
        vec3<f32>(0., c, s),
        vec3<f32>(0., -s, c),
    );
}

fn yaw_matrix() -> mat3x3<f32> {
    let s = sin(em.yaw);
    let c = cos(em.yaw);

    return mat3x3<f32>(
        vec3<f32>(c, 0., -s),
        vec3<f32>(0., 1., 0.),
        vec3<f32>(s, 0., c),
    );
}


fn create_particle_position(input_random: f32) -> vec3<f32> {
    let diameter_length = em.box_length * 2.0;
    let diameter_height = em.box_height * 2.0;
    let diameter_depth = em.box_depth * 2.0;

    let random_length = random(input_random * 1.6, em.elapsed_sec);
    let random_height = random(input_random * 0.42, em.elapsed_sec);
    let random_depth = random(input_random / 0.11, em.elapsed_sec);

    let unrotated_x = random_length * diameter_length - em.box_length;
    let unrotated_y = random_height * diameter_height - em.box_height;
    let unrotated_z = random_depth * diameter_depth - em.box_depth;

    let unrotated = vec3<f32>(unrotated_x, unrotated_y, unrotated_z);
    let local = unrotated * roll_matrix() * yaw_matrix() * pitch_matrix();
    
    return vec3<f32>(em.box_x, em.box_y, em.box_z) + local;
}

fn spawn_particle(index: u32) {
    var particle = particles_src[index];

    let input_random = f32(index);
    let particle_position = create_particle_position(input_random);

    let particle_color = vec4<f32>(
        em.particle_color_r,
        em.particle_color_g,
        em.particle_color_b,
        em.particle_color_a,
    );

    var velocity = create_spread_xy(input_random);
    velocity = create_spread_z(input_random, velocity);

    particle.position = particle_position;
    particle.velocity = velocity;
    particle.color = particle_color;
    particle.size = em.particle_size;
    particle.spawned_at = em.elapsed_sec;
    particle.lifetime = em.particle_lifetime;
    particle.scale = em.particle_scale;
    particle.mass = em.particle_mass;

    particles_dst[index] = particle;
}

fn is_decayed(particle: Particle) -> bool {
    return particle.lifetime == 0. || particle.lifetime < em.elapsed_sec - particle.spawned_at;
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

    if (is_decayed(particle)) {
        particle.lifetime = 0.;
        particles_dst[index] = particle;
        return;
    }

    let x_force = particle.velocity.x * particle.mass;
    let y_force = particle.velocity.y * particle.mass;
    let z_force = particle.velocity.z * particle.mass;
    
    let friction_coefficient = em.particle_friction_coefficient;

    particle.velocity.x = x_force * friction_coefficient / particle.mass;
    particle.velocity.y = y_force * friction_coefficient / particle.mass;
    particle.velocity.z = z_force * friction_coefficient / particle.mass;

    particle.position.x += particle.velocity.x * em.delta_sec;
    particle.position.y += particle.velocity.y * em.delta_sec;
    particle.position.z += particle.velocity.z * em.delta_sec;

    particles_dst[index] = particle;
}

