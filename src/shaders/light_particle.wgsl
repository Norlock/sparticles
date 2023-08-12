struct CameraUniform {
    view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    rotated_vertices: mat4x4<f32>,
    vertex_positions: mat4x2<f32>,
    view_pos: vec4<f32>,
};

@group(1) @binding(0) 
var<uniform> camera: CameraUniform;

@group(2) @binding(0) 
var<storage, read> particles: array<Particle>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) world_space: vec4<f32>,
    @location(2) v_pos: vec2<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vert_idx: u32,
    @builtin(instance_index) instance_idx: u32,
) -> VertexOutput {
    var uvs: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
      vec2<f32>(0.0, 1.0),
      vec2<f32>(0.0, 0.0),
      vec2<f32>(1.0, 1.0),
      vec2<f32>(1.0, 0.0),
    );

    let p = particles[instance_idx];

    if (p.lifetime == -1.) {
        var out: VertexOutput;
        out.clip_position = vec4<f32>(-9999.);
        return out;
    }
    
    let world_space: vec4<f32> = 
        vec4<f32>(p.position + camera.rotated_vertices[vert_idx].xyz * p.size, 1.0);

    let v_pos = camera.vertex_positions[vert_idx];

    var out: VertexOutput;
    out.v_pos = v_pos;
    out.color = p.color;
    out.clip_position = camera.view_proj * world_space;
    out.world_space = world_space;

    return out;
}

@group(0) @binding(0)
var base_texture: texture_2d<f32>;
@group(0) @binding(1)
var base_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let len = length(in.v_pos);

    if (1.0 < len) {
        discard;
    }

    let x = in.v_pos.x;
    let y = in.v_pos.y;
    let normal = vec4<f32>(x, y, sqrt(1. - x * x - y * y), 0.);
    let world_normal = normal * camera.view;

    let light_dir = normalize(camera.view_pos.xyz - in.world_space.xyz);
    let diffuse_strength = max(dot(world_normal.xyz, light_dir), 0.0);

    return diffuse_strength * in.color;
}
