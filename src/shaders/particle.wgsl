struct CameraUniform {
    view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    rotated_vertices: mat4x4<f32>,
    vertex_positions: mat4x2<f32>,
    view_pos: vec4<f32>,
};

@group(1) @binding(0) 
var<uniform> camera: CameraUniform;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) pos_uv: vec4<f32>,
    @location(1) world_space: vec4<f32>,
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

    // vertex positions
    let world_space: vec4<f32> = 
        camera.rotated_vertices[vert_idx]; // particle size (1.0)

    var out: VertexOutput;
    out.pos_uv = vec4<f32>(camera.vertex_positions[vert_idx], uvs[vert_idx]);
    out.world_space = world_space;
    out.clip_position = camera.view_proj * world_space;

    return out;
}

@group(0) @binding(0)
var base_texture: texture_2d<f32>;
@group(0) @binding(1)
var base_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let len = length(in.world_space.xyz);

    let texture_color = textureSample(base_texture, base_sampler, in.pos_uv.zw);

    if (1.0 < len) {
        discard;
    }

    let color = vec4<f32>(0.5, 0.5, 0.5, 1.0);

    let x = in.pos_uv.x;
    let y = in.pos_uv.y;

    let normal = vec3<f32>(x, y, sqrt(1. - x * x - y * y));
    let world_normal = vec4<f32>(normal, 0.) * camera.view;

    let light_pos = vec3<f32>(-10., 0., -10.);
    let light_dir = normalize(light_pos - in.world_space.xyz);
    let view_dir = normalize(camera.view_pos.xyz - in.world_space.xyz);
    let half_dir = normalize(view_dir + light_dir);

    let distance = length(light_pos - in.world_space.xyz);
    let strength = 1.0 - distance * 0.04;

    let ambient_color = color * strength;

    let diffuse_strength = max(dot(world_normal.xyz, light_dir), 0.0);
    let diffuse_color = diffuse_strength * ambient_color;

    let specular_strength = pow(max(dot(world_normal.xyz, half_dir), 0.0), 32.0);
    let specular_color = specular_strength * ambient_color;

    return (diffuse_color + specular_color) * texture_color;
}
