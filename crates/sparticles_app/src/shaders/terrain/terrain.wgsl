struct Terrain {
    noise: f32,
    elapsed: f32,
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

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var<uniform> terrain_globals: Terrain;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    out.pos = vec4(positions[vertex_index], 0., 1.);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var N = noise(in.pos.xy * 3.7) * 0.5 + 0.5;
    return vec4(vec3(1.0) * N, 1.0);
}
