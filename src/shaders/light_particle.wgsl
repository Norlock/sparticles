@group(0) @binding(0) 
var<uniform> camera: CameraUniform;

@group(2) @binding(0) 
var<storage, read> particles: array<Particle>;

@group(2) @binding(2) var<uniform> em: Emitter; 

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) world_space: vec4<f32>,
    @location(2) uv: vec4<f32>,
};

var<private> uvs: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
  vec2<f32>(0., 1. ),
  vec2<f32>(1., 1.),
  vec2<f32>(0., 0.),
  vec2<f32>(1., 0.),
);

@vertex
fn vs_main(
    @builtin(vertex_index) vert_idx: u32,
    @builtin(instance_index) instance_idx: u32,
) -> VertexOutput {

    let p = particles[instance_idx];

    if (p.lifetime == -1.) {
        var out: VertexOutput;
        out.clip_position = vec4<f32>(-9999.);
        return out;
    }
    
    let world_space: vec4<f32> = 
        vec4<f32>(p.pos_size.xyz + camera.rotated_vertices[vert_idx].xyz * p.pos_size.w, 1.0);

    var out: VertexOutput;
    out.color = p.color;
    out.world_space = world_space;
    out.clip_position = camera.view_proj * world_space;
    out.uv = vec4<f32>(uvs[vert_idx], f32(instance_idx), 0.);

    return out;
}

@group(1) @binding(0)
var base_texture: texture_2d<f32>;
@group(1) @binding(1)
var base_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let v_pos = in.uv.xy * 2. - 1.;

    let len = length(v_pos);
    let texture_color = textureSample(base_texture, base_sampler, in.uv.xy);

    if 1.0 < len {
        discard;
    }

    var strength = 1.0 - len * 0.7;
    var color = in.color.rgb * strength;

    let x = v_pos.x;
    let y = v_pos.y;
    let idx = in.uv.z;

    let normal = sqrt(1. - x * x - y * y);

    var effect = create_layers(v_pos, normal, idx, em.elapsed_sec);
    effect *= 1. - 0.02 / color.rgb;
    effect += 0.5;

    //let result = in.color
    return vec4<f32>(texture_color.rgb * effect, 1.0);
}
