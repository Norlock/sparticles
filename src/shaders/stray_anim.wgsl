// Includes declarations

struct StrayAnimation {
    stray_radians: f32,
    from_sec: f32,
    until_sec: f32,
}

@group(0) @binding(1) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(2) var<uniform> em: Emitter; 
@group(1) @binding(0) var<uniform> anim: StrayAnimation; 

fn create_stray(input_random: f32, vel: vec3<f32>) -> vec3<f32> {
    let stray = anim.stray_radians;
    
    let pitch_stray = gen_dyn_range(input_random * 0.11, stray, em.elapsed_sec);
    let yaw_stray = gen_dyn_range(input_random * 0.44, stray, em.elapsed_sec);
    let roll_stray = gen_dyn_range(input_random * 0.342, stray, em.elapsed_sec);

    return vel 
        * yaw_matrix(pitch_stray) 
        * pitch_matrix(yaw_stray) 
        * roll_matrix(roll_stray);
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

    if (particle.lifetime < anim.from_sec || anim.until_sec <= particle.lifetime) {
        return;
    }
    
    let input_random = em.elapsed_sec + f32(index);

    let vel = create_stray(input_random, particle.vel_mass.xyz);
    particle.vel_mass.x = vel.x;
    particle.vel_mass.y = vel.y;
    particle.vel_mass.z = vel.z;

    particles[index] = particle;
}

