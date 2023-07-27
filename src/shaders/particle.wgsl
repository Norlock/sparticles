// Vertex shader

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) pos_uv: vec4<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vert_idx: u32,
) -> VertexOutput {
    var out: VertexOutput;

    var uvs: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
      vec2<f32>(0.0, 1.0),
      vec2<f32>(0.0, 0.0),
      vec2<f32>(1.0, 1.0),
      vec2<f32>(1.0, 0.0),
    );

    // vertex positions
    var vert_poss: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
      vec2<f32>(-1.0, -1.0),
      vec2<f32>(1.0, -1.0),
      vec2<f32>(-1.0, 1.0),
      vec2<f32>(1.0, 1.0),
    );

    out.pos_uv = vec4<f32>(vert_poss[vert_idx], uvs[vert_idx]);
    out.clip_position = vec4<f32>(vert_poss[vert_idx], 0.0, 1.0);

    return out;
}

@group(0) @binding(0)
var base_texture: texture_2d<f32>;
@group(0) @binding(1)
var base_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let len = length(in.pos_uv.xy);

    let texture_color = textureSample(base_texture, base_sampler, in.pos_uv.zw);

    if (1.0 < len) {
        discard;
    }

    var color = vec4<f32>(0.3, 0.5, 0.3, 1.0);

    return texture_color * color;
}
