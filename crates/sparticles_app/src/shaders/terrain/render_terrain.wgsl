@group(0) @binding(1) var terrain_map: texture_cube<f32>;
@group(0) @binding(2) var terrain_s: sampler;
@group(1) @binding(0) var<uniform> camera: Camera;

var<private> full_triangle: array<vec2<f32>, 3> = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -3.0),
    vec2<f32>(-1.0, 1.0),
    vec2<f32>(3.0, 1.0)
);

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec3<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vert_idx: u32) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(full_triangle[vert_idx], 0., 1.);

    let unprojected = camera.inv_proj * out.clip_position;
    out.uv = camera.inv_view * unprojected.xyz;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(terrain_map, terrain_s, in.uv).rgb;
    color = color / (color + vec3(1.0));

    return vec4(linear_to_srgb(color), 1.);
}

