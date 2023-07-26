// Includes declarations

struct StrayAnimation {
    stray_factor: f32,
    from_sec: f32,
    until_sec: f32,
}

@group(0) @binding(0) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(2) var<uniform> em: Emitter; 
@group(1) @binding(0) var<uniform> anim: StrayAnimation; 

fn create_stray(input_random: f32, vel: vec3<f32>) -> vec3<f32> {
    // Velocity in lets say x 10, y 0, z 0
    // invert pow (x * abs(x), y * abs(y), z * abs(z))
    // total speed get abs values abs(x) + abs(y) + abs(z)
    // 0.1 stray factor 100 * 0.1 == 10 (somewhere between -10 and 10)
    // 100 -> add all values to x,y,z (6.44, -3,22, 1.67)
    // (106.44, -3.22, 1.67) -> normalize to 100 again. 
    // before -> 100, after (abs(newx) + abs(newy) + abs(newz) -> 111.33  
    // 100 / 111.33
    // 0.8982
    // 106.44 * 0.8982 == 95.6044
    // -3.22 * 0.8982 == -2.8982
    // 1.67 * 0.8982 == 1.4999
    // v95.6 => 9.777
    // v2.8982 => 1.702
    // v1.49999 => 1.224704

    let spow_x = vel.x * abs(vel.x);
    let spow_y = vel.y * abs(vel.y);
    let spow_z = vel.z * abs(vel.z);

    let before_total = abs(spow_x) + abs(spow_y) + abs(spow_z);

    let randomize = anim.stray_factor * before_total;

    let stray_x = gen_dyn_range(input_random * 0.11, randomize, em.elapsed_sec);
    let stray_y = gen_dyn_range(input_random * 0.44, randomize, em.elapsed_sec);
    let stray_z = gen_dyn_range(input_random * 0.33, randomize, em.elapsed_sec);

    let new_spow_x = spow_x + stray_x;
    let new_spow_y = spow_y + stray_y;
    let new_spow_z = spow_z + stray_z;

    let after_total = abs(new_spow_x) + abs(new_spow_y) + abs(new_spow_z);

    let scale_factor = before_total / after_total;

    let vx = sqrt(abs(new_spow_x) * scale_factor) * sign(new_spow_x);
    let vy = sqrt(abs(new_spow_y) * scale_factor) * sign(new_spow_y);
    let vz = sqrt(abs(new_spow_z) * scale_factor) * sign(new_spow_z);

    return vec3<f32>(vx, vy, vz); 
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

    if (particle.lifetime == 0.) {
        return;
    }

    let age = em.elapsed_sec - particle.spawned_at;

    if (age < anim.from_sec || anim.until_sec <= age) {
        return;
    }
    
    let input_random = em.elapsed_sec + f32(index);

    particle.velocity = create_stray(input_random, particle.velocity);

    particles[index] = particle;
}

