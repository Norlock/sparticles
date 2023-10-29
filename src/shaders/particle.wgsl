@group(0) @binding(0) 
var<uniform> camera: CameraUniform;

@group(2) @binding(0) 
var<storage, read> particles: array<Particle>;

@group(3) @binding(0) 
var<storage, read> light_particles: array<Particle>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) pos_uv: vec4<f32>,
    @location(1) world_space: vec4<f32>,
    @location(2) color: vec4<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vert_idx: u32,
    @builtin(instance_index) instance_idx: u32,
) -> VertexOutput {
    var uvs: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
      vec2<f32>(0., 0.),
      vec2<f32>(1., 0.),
      vec2<f32>(0., 1.),
      vec2<f32>(1., 1.),
    );

    let p = particles[instance_idx];

    if (p.lifetime == -1.) {
        var out: VertexOutput;
        out.clip_position = vec4<f32>(-9999.);
        return out;
    }
    
    let world_space: vec4<f32> = 
        vec4<f32>(
            p.pos_size.xyz + 
            camera.rotated_vertices[vert_idx].xyz * 
            p.pos_size.w, 1.0
    );

    var out: VertexOutput;
    out.pos_uv = vec4<f32>(camera.vertex_positions[vert_idx], uvs[vert_idx]);
    out.world_space = world_space;
    out.color = p.color;
    out.clip_position = camera.view_proj * world_space;

    return out;
}

@group(1) @binding(0) var base_texture: texture_2d<f32>;
@group(1) @binding(1) var base_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let len = length(in.pos_uv.xy);

    let texture_color = textureSample(base_texture, base_sampler, in.pos_uv.zw);

    if (1.0 < len) {
        discard;
    }

    let x = in.pos_uv.x;
    let y = in.pos_uv.y;

    let normal = vec3<f32>(x, y, sqrt(1. - x * x - y * y));
    let world_normal = (vec4<f32>(normal, 0.) * camera.view).xyz;

    var result = vec3<f32>(0.0);


    for (var i = 0u; i < arrayLength(&light_particles); i++) { 
        let light = light_particles[i];
        let light_pos = light.pos_size.xyz;

        let distance = length(light_pos - in.world_space.xyz);
        let strength = 1.0 - distance * 0.04;
        let ambient_color = light.color.rgb * strength;

        if (strength <= 0.0) {
            continue;
        }

        let light_dir = normalize(light_pos - in.world_space.xyz);
        let view_dir = normalize(camera.view_pos.xyz - in.world_space.xyz);
        let half_dir = normalize(view_dir + light_dir);

        let diffuse_strength = max(dot(world_normal, light_dir), 0.0);
        let diffuse_color = diffuse_strength * ambient_color;

        let specular_strength = pow(max(dot(world_normal, half_dir), 0.0), 32.0);
        let specular_color = specular_strength * ambient_color;

        result += diffuse_color + specular_color;
    }

    return vec4<f32>(result * in.color.rgb * texture_color.rgb, in.color.a);
}
