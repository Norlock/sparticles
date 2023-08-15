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

@group(2) @binding(2) var<uniform> em: Emitter; 

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) world_space: vec4<f32>,
    @location(2) v_pos: vec4<f32>,
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
        vec4<f32>(p.position + camera.rotated_vertices[vert_idx].xyz * p.size * 2.0, 1.0);

    let v_pos = camera.vertex_positions[vert_idx];

    var out: VertexOutput;
    out.v_pos = vec4<f32>(v_pos, f32(instance_idx), 0.);
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
    let len = length(in.v_pos.xy * 2.);
    var color = in.color;

    let x = in.v_pos.x;
    let y = in.v_pos.y;

    if (len <= 1.0) {
        let normal = vec4<f32>(x, y, sqrt(1. - x * x - y * y), 0.);
        let world_normal = normal * camera.view;

        let light_dir = normalize(camera.view_pos.xyz - in.world_space.xyz);
        let diffuse_strength = max(dot(world_normal.xyz, light_dir), 0.0);

        let dis = (1.0 + perlin_noise(vec3<f32>(in.v_pos.xy, in.v_pos.z + em.elapsed_sec * 0.08 ) * 4.0))
            * (1.0 + (worley(in.v_pos.xy, 64.0) 
            + 0.5 * worley(2.0 * in.v_pos.xy, 64.0) 
            + 0.25 * worley(4.0 * in.v_pos.xy, 64.0)));

        return vec4<f32>(vec3<f32>(dis / 4.0), 1.0);
    } else {

        let alpha_loss = sin(0.75 * em.elapsed_sec);

        color.a = max(1.0 - len * pow(alpha_loss, 2.0), 0.0);
        discard;
        //let d_sun = sin(3.14 / (len + 1.1)) * 10.0;
        //
        //var color = in.color;
        //color.r = d_sun;
        //color.g = d_sun * 0.7;
    }

    //return diffuse_strength * in.color * sun_color;
    return color;
}
