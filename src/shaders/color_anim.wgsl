// Includes declarations

struct ColorAnimation {
    from_col: vec4<f32>,
    to_col: vec4<f32>,
    from_sec: f32,
    until_sec: f32,
    padding_1: f32,
    padding_2: f32,
}

@group(0) @binding(1) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(2) var<uniform> emitter: Emitter; 
@group(1) @binding(0) var<uniform> anim: ColorAnimation; 

@compute
@workgroup_size(128)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let particle_len = arrayLength(&particles);
    let index = global_invocation_id.x;

    if particle_len <= index {
        return;
    }

    var particle = particles[index];

    if particle.lifetime < anim.from_sec || anim.until_sec <= particle.lifetime {
        return;
    }

    let delta_sec = particle.lifetime - anim.from_sec;
    let delta_end = anim.until_sec - anim.from_sec;
    let fraction = delta_sec / delta_end;

    particle.color = anim.from_col + fraction * (anim.to_col - anim.from_col);

    particles[index] = particle;
}

