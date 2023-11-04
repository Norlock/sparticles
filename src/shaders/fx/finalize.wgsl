struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

var<private> positions: array<vec2<f32>, 3> = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -3.0),
    vec2<f32>(-1.0, 1.0),
    vec2<f32>(3.0, 1.0)
);

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    out.pos = vec4<f32>(positions[vertex_index], 0., 1.);
    out.uv = out.pos.xy * 0.5 + 0.5;
    out.uv.y = 1.0 - out.uv.y;
    return out;
}

@group(0) @binding(0) var read_fx: binding_array<texture_2d<f32>, 32>;
@group(0) @binding(1) var s: sampler;
@group(1) @binding(0) var<uniform> fx_io: FxIO; 

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(read_fx[fx_io.out_idx], s, in.uv);
}
