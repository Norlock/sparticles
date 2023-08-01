struct Particle {
    position: vec3<f32>,
    size: f32,
    color: vec4<f32>,
    velocity: vec3<f32>, 
    lifetime: f32, // lifetime == -1. is decayed
    mass: f32,
};

struct Emitter {
    delta_sec: f32,
    elapsed_sec: f32,
    spawn_from: f32,
    spawn_until: f32,
    pos_x: f32,
    pos_y: f32,
    pos_z: f32,
    particle_color_r: f32,
    particle_color_g: f32,
    particle_color_b: f32,
    particle_color_a: f32,
    particle_velocity_x: f32,
    particle_velocity_y: f32,
    particle_velocity_z: f32,
    particle_friction_coefficient: f32,
    particle_size: f32,
    particle_mass: f32,
    particle_lifetime: f32,
};

fn random(input: f32, elapsed_sec: f32) -> f32 {
    let value = vec2<f32>(input, elapsed_sec);
    return fract(sin(dot(value, vec2<f32>(12.9898,78.233))) * 43758.5453);
}

fn gen_abs_range(unique: f32, value: f32, elapsed_sec: f32) -> f32 {
    return abs(random(unique, elapsed_sec)) * value;
}

fn gen_dyn_range(unique: f32, value: f32, elapsed_sec: f32) -> f32 {
    return sin(random(unique, elapsed_sec) * 60.) * value;
}
