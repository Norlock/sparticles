struct Terrain {
    noise: f32,
    group_size: f32,
}

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

var<private> positions: array<vec2<f32>, 3> = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -3.0),
    vec2<f32>(-1.0, 1.0),
    vec2<f32>(3.0, 1.0)
);

@group(0) @binding(0) var cube: texture_cube_2d;
@group(1) @binding(0) var<uniform> terrain_globals: Terrain;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    out.pos = vec4(positions[vertex_index], 0., 1.);
    return out;
}

// What needs to happen is depending on xyz and camera angle needs to be updated

fn has_star() -> bool {
    // Get the frustum of the camera
    // generate noise based on the xyz inside the frustum
    return true;
}

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
    let group_x = pos_in.x / 128.;
    let group_y = pos_in.y / 128.;


    var pos = pos_in.xy;
    let star_size = 4.;
    let depth = 10.;
    let star = depth / abs(camera.position.z) * star_size; // TODO make a minimal change on size

    if star <= 1. {
        return vec3(0.);
    }

    let empty = 50.;
    let space = star + empty;

    var cam_xy = camera.position.xy;
    cam_xy.y *= -1.;

    let offset = (cam_xy * 50.) % space;

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


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4(stars(in.pos.xyz), 1.0);
}
