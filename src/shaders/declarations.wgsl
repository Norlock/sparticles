struct Particle {
    position: vec3<f32>,
    size: f32,
    color: vec4<f32>,
    velocity: vec3<f32>, 
    spawned_at: f32,
    lifetime: f32, // lifetime == 0. is decayed
    scale: f32,
    mass: f32,
};

struct Cell {
    x: f32,
    y: f32,
    z: f32,

    // Total mass
    mass: f32,

    // Stores only particle index
    particles: array<u32>, 
}

struct Camera {
    view_pos: vec4<f32>,
    view_matrix: mat4x4<f32>,
    view_proj: mat4x4<f32>,
    vertex_positions: mat4x2<f32>,
    rotated_vertices: mat4x4<f32>,
};

struct Emitter {
    box_x: f32,
    box_y: f32,
    box_z: f32,
    box_length: f32,
    box_height: f32,
    box_depth: f32,
    pitch: f32,
    roll: f32,
    yaw: f32,

    spread_xy: f32,
    spread_z: f32,

    particle_speed_x: f32,
    particle_speed_y: f32,
    particle_speed_z: f32,

    particle_color_r: f32,
    particle_color_g: f32,
    particle_color_b: f32,
    particle_color_a: f32,
    particle_friction_coefficient: f32,
    particle_size: f32,
    particle_scale: f32,
    particle_mass: f32,

    spawn_from: f32,
    spawn_until: f32,
    particle_lifetime: f32,
    elapsed_sec: f32,
    delta_sec: f32,
    padding: f32,
};

struct TrailPoint {
    x: f32,
    y: f32,
    z: f32,
    alive: f32,
}

struct TrailUniform {
    trail_length: f32,
    update_index: f32,

    col_r: f32,
    col_g: f32,
    col_b: f32,
    col_a: f32,
}

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
