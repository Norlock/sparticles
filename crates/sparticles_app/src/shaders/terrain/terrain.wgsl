struct Terrain {
    noise: f32,
    group_size: f32,
}

@group(0) @binding(0) var cube_write: texture_storage_2d_array<rgba8unorm, write>;
@group(0) @binding(1) var cube_read: texture_cube<f32>;
@group(1) @binding(0) var<uniform> terrain: Terrain;

@compute
@workgroup_size(16, 16)
fn generate_terrain(@builtin(global_invocation_id) position: vec3<u32>) {
    let group_size = u32(terrain.group_size);

    if any(vec2(group_size) <= position.xy) || any(vec2(2048u) <= position.xy) {
        return;
    }

    let color = vec4(stars(vec3<f32>(position)), 1.0);

    textureStore(cube_write, position.xy, position.z, color);
}

// What needs to happen is depending on xyz and camera angle needs to be updated

// Create a small texture width x height = 128 x 128
// Generate random noise with values between 0 and 100
// If exceeds a treshold it is a star
// If it doesn't it will become black
// upscale texture make some random function that will either scale the star up or won't


// Generate stars based on xyz position of the camera
// Mimic a texture cube by using io 0-5
// in particle.wgsl + light_particle.wgsl use the camera to convert the cube back to 2d view
// render 

// |---------|---------|
// |         |         |
// |160, 0   |160, 160 |
// |         |         |
// |---------|---------|

fn stars(pos_in: vec3<f32>) -> vec3<f32> {
    //let group_x = pos_in.x / 128.;
    //let group_y = pos_in.y / 128.;


    var pos = pos_in.xy;
    let star_size = 4.;
    let depth = 10.;
    let star = star_size; // TODO make a minimal change on size

    let empty = 50.;
    let space = star + empty;

    //var cam_xy = camera.position.xy;
    //cam_xy.y *= -1.;

    let offset = 1.0;

    //let offset = random_v2(pos);

    let spot = (pos + offset) % space;

    let half = space / 2.;
    let remainder = half - spot;

    if length(remainder) <= star {
        return vec3(1.0);
    } else {
        return vec3(0.0);
    }
}
