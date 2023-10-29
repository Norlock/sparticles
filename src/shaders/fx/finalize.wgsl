struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var uvs: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
      vec2<f32>(0., 0.),
      vec2<f32>(1., 0.),
      vec2<f32>(0., 1.),
      vec2<f32>(1., 1.),
    );

    var vertices: array<vec4<f32>, 4> = array<vec4<f32>, 4>(
        vec4<f32>(-1., -1., 0., 1.),
        vec4<f32>(1.0, -1., 0., 1.),
        vec4<f32>(-1., 1.0, 0., 1.),
        vec4<f32>(1.0, 1.0, 0., 1.),
    );

    var out: VertexOutput;
    out.pos = vertices[vertex_index];
    out.uv = uvs[vertex_index];
    return out;
}

@group(0) @binding(1) var read_fx: binding_array<texture_2d<f32>, 32>;
@group(0) @binding(4) var s: sampler;
@group(1) @binding(0) var<uniform> fx_io: FxIO; 

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(read_fx[fx_io.out_idx], s, in.uv);
}
