struct CameraUniform {
    view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    rotated_vertices: mat4x4<f32>,
};

@group(1) @binding(0) 
var<uniform> camera: CameraUniform;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) pos_uv: vec4<f32>,
    @location(1) world_space: vec3<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vert_idx: u32,
) -> VertexOutput {
    var uvs: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
      vec2<f32>(0.0, 1.0),
      vec2<f32>(0.0, 0.0),
      vec2<f32>(1.0, 1.0),
      vec2<f32>(1.0, 0.0),
    );

    var vert_poss: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
      vec2<f32>(-1.0, -1.0),
      vec2<f32>(1.0, -1.0),
      vec2<f32>(-1.0, 1.0),
      vec2<f32>(1.0, 1.0),
    );

    // vertex positions
    let world_space: vec3<f32> = 
        camera.rotated_vertices[vert_idx].xyz; // particle size (1.0)

    var out: VertexOutput;
    out.pos_uv = vec4<f32>(vert_poss[vert_idx], uvs[vert_idx]);
    out.world_space = world_space;
    out.clip_position = camera.view_proj * vec4<f32>(world_space, 1.0);

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

    let x = in.pos_uv.x;
    let y = in.pos_uv.y;

    let color = vec4<f32>(0.5, 0.5, 0.5, 1.0);
    let normal = vec3<f32>(x, y, sqrt(1. - x * x - y * y));
    let world_normal = vec4<f32>(normal, 0.) * camera.view;

    let light_pos = vec3<f32>(-10., 0., 10.);
    let light_dir = normalize(light_pos - in.world_space);
    let diffuse_color = max(dot(world_normal.xyz, light_dir), 0.0);

    return texture_color * color * diffuse_color;
}
