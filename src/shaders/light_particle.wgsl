// Includes declarations
@group(0) @binding(0)
var<uniform> camera: Camera;
@group(1) @binding(1) 
var<storage, read> particles: array<Particle>;
@group(2) @binding(0)
var base_texture: texture_2d<f32>;
@group(2) @binding(1)
var base_sampler: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) pos_uv: vec4<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_idx: u32,
    @builtin(instance_index) instance_idx: u32,
) -> VertexOutput {
    var uvs: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
      vec2<f32>(0.0, 1.0),
      vec2<f32>(0.0, 0.0),
      vec2<f32>(1.0, 1.0),
      vec2<f32>(1.0, 0.0),
    );

    let particle = particles[instance_idx];

    if (particle.lifetime == 0.) {
        var out: VertexOutput;
        out.clip_position = vec4<f32>(-9999.);
        return out;
    }

    let world_space: vec3<f32> = 
        particle.position.xyz + 
        camera.rotated_vertices[vertex_idx].xyz * particle.size;

    var out: VertexOutput;

    out.clip_position = camera.view_proj * vec4<f32>(world_space, 1.0);
    out.color = particle.color;
    out.pos_uv = vec4<f32>(camera.vertex_positions[vertex_idx], uvs[vertex_idx]);

    return out;
}

@fragment
fn fs_main_circle(in: VertexOutput) -> @location(0) vec4<f32> {
    let len = length(in.pos_uv.xy);
    let texture_color = textureSample(base_texture, base_sampler, in.pos_uv.zw);

    if (1.0 < len) {
        discard;
    }

    let distance = len - 1.0;

    var col =  in.color.rgb * step(0., -distance); 
    var glow = 0.0005 / distance;

    col += clamp(glow, 0., 1.); // remove artifacts

    return vec4<f32>(col * texture_color.rgb, 1.);
}

@fragment
fn fs_main_quad(in: VertexOutput) -> @location(0) vec4<f32> {
    let texture_color = textureSample(base_texture, base_sampler, in.pos_uv.zw);
    return in.color * texture_color;
}
