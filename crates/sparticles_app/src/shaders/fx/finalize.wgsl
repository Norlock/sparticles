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

@group(0) @binding(0) var read_fx: binding_array<texture_2d<f32>, 16>;
@group(0) @binding(1) var s: sampler;
@group(1) @binding(0) var<uniform> fx_io: FxIO; 
@group(2) @binding(1) var cube_read: texture_cube<f32>;
@group(2) @binding(2) var cube_s: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let col = textureSample(read_fx[fx_io.out_idx], s, in.uv);
    //for (var i = 0; i < 6; i++) {
    //    let col2 = textureSample(cube_read, cube_s, vec3(in.uv, 0.)).rgb;
    //    if any(vec3(0.) < col2) {
    //        return vec4(1.0);
    //    }
    //}

    let col2 = textureSample(cube_read, cube_s, vec3(in.uv, -1.)).rgb;
    return col + vec4(col2, 1.0);
}
