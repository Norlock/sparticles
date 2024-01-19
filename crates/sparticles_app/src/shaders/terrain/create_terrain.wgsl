struct Terrain {
    noise: f32,
    tex_size: u32,
}

@group(0) @binding(0) var cube_write: texture_storage_2d_array<rgba16float, write>;
@group(0) @binding(1) var cube_read: texture_cube<f32>;
@group(0) @binding(2) var cube_s: sampler;
@group(1) @binding(0) var<uniform> terrain: Terrain;
@group(2) @binding(0) var<uniform> camera: Camera;

const TEXTURE_SIZE: u32 = 2048u;

@compute
@workgroup_size(16, 16)
fn generate_terrain(@builtin(global_invocation_id) position: vec3<u32>) {
    if any(vec2(terrain.tex_size) <= position.xy) {
        return;
    }

    let color = vec4(stars(position), 1.0);

    textureStore(cube_write, position.xy, position.z, color);
}

// What needs to happen is depending on xyz and camera angle needs to be updated

// Create a small texture width x height = 128 x 128
// Generate random noise with values between 0 and 100
// If exceeds a treshold it is a star
// If it doesn't it will become black
// upscale texture make some random function that will either scale the star up or won't

fn stars(pos_in: vec3<u32>) -> vec3<f32> {
    //let layer = pos_in.z;

    var pos = vec2<f32>(pos_in.xy);
    let star_size = 4.;
    let depth = 10.;
    let star = star_size; // TODO make a minimal change on size

    let empty = 50.;
    let space = star + empty;

    //var cam_xy = camera.position.xy;
    //cam_xy.y *= -1.;

    let offset = 1.0;

    //let offset = random_v2(pos);

    let spot = pos % space;

    let half = space / 2.;
    let remainder = half - spot;

    if length(remainder) <= star {
        return vec3(200.0);
    } else {
        return vec3(0.0);
    }
}
