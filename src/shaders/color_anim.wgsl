// Includes declarations

struct ColorAnimation {
    from_col_r: f32,
    from_col_g: f32,
    from_col_b: f32,
    from_col_a: f32,
    to_col_r: f32,
    to_col_g: f32,
    to_col_b: f32,
    to_col_a: f32,
    beginning_sec: f32,
    ending_sec: f32,
    // animation_style
}

@group(0) @binding(0) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(2) var<uniform> emitter: Emitter; 

@group(1) @binding(0) var<uniform> anim: ColorAnimation; 

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

    let age = emitter.elapsed_sec - particle.spawned_at;

    if (age < anim.beginning_sec || anim.ending_sec <= age) {
        return;
    }

    let delta_sec = age - anim.beginning_sec;
    let delta_end = anim.ending_sec - anim.beginning_sec;
    let fraction = delta_sec / delta_end;

    particle.color.r = anim.from_col_r + fraction * (anim.to_col_r - anim.from_col_r);
    particle.color.g = anim.from_col_g + fraction * (anim.to_col_g - anim.from_col_g);
    particle.color.b = anim.from_col_b + fraction * (anim.to_col_b - anim.from_col_b);

    particles[index] = particle;
}

