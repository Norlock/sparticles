// check declarations
@group(0) @binding(0)
var<uniform> camera: Camera;
@group(1) @binding(1) 
var<storage, read> particles: array<Particle>;
@group(2) @binding(1)
var<storage, read> lights: array<Particle>;
@group(3) @binding(0)
var base_texture: texture_2d<f32>;
@group(3) @binding(1)
var base_sampler: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) pos_uv: vec4<f32>,
    @location(2) world_space: vec3<f32>,
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
    out.world_space = world_space;

    return out;
}

@fragment
fn fs_main_circle(in: VertexOutput) -> @location(0) vec4<f32> {
    let len = length(in.pos_uv.xy);

    let texture_color = textureSample(base_texture, base_sampler, in.pos_uv.zw);

    if (1.0 < len) {
        discard;
    } 

    let x = in.pos_uv.x;
    let y = in.pos_uv.y;
    let normal = vec3<f32>(x, y, sqrt(1. - x * x - y * y));
    let world_normal = vec4<f32>(normal, 0.0) * camera.view_matrix;
    
    var result = vec3<f32>(0.0);

    // TODO calc from light instead of from particle 
    for (var i = 0u; i < arrayLength(&lights); i++) { 
        let light = lights[i];
        let light_pos = light.position;

        let distance = length(light_pos - in.world_space);
        
        var strength = 1.0 - distance * 0.04;

        if (strength <= 0.0) {
            continue;
        }

        let ambient_color = light.color.rgb * strength;

        let light_dir = normalize(light_pos - in.world_space);
        let view_dir = normalize(camera.view_pos.xyz - in.world_space);
        let half_dir = normalize(view_dir + light_dir);

        let diffuse_strength = max(dot(world_normal.xyz, light_dir), 0.0);
        let diffuse_color = diffuse_strength * ambient_color;

        let specular_strength = pow(max(dot(world_normal.xyz, half_dir), 0.0), 32.0);
        let specular_color = specular_strength * ambient_color;

        result += diffuse_color + specular_color;
    }


    if (length(result) == 0.) {
        discard;
    } else {
        return vec4<f32>(result * in.color.rgb * texture_color.rgb, in.color.a);
    }
}

//@fragment
//fn fs_main_quad(in: VertexOutput) -> @location(0) vec4<f32> {
//    let world_normal = normalize(vec4<f32>(in.position, 0.0, 0.0)) * camera.view_matrix;
//    
//    var result = vec3<f32>(0.0);
//
//    for (var i = 0u; i < arrayLength(&lights); i++) { 
//        let light = lights[i];
//        let light_pos = light.position;
//
//        let distance = length(light_pos - in.world_space);
//        
//        var strength = 1.0 - distance * 0.05;
//
//        if (strength <= 0.0) {
//            continue;
//        }
//
//        let ambient_color = light.color.rgb * strength;
//
//        let light_dir = normalize(light_pos - in.world_space);
//        let view_dir = normalize(camera.view_pos.xyz - in.world_space);
//        let half_dir = normalize(view_dir + light_dir);
//
//        let diffuse_strength = max(dot(world_normal.xyz, light_dir), 0.0);
//        let diffuse_color = diffuse_strength * ambient_color;
//
//        let specular_strength = pow(max(dot(world_normal.xyz, half_dir), 0.0), 32.0);
//        let specular_color = specular_strength * ambient_color;
//
//        result += diffuse_color + specular_color;
//    }
//    
//    let texture_color = textureSample(base_texture, base_sampler, in.uv);
//    return vec4<f32>(result * in.color.rgb * texture_color.rgb, in.color.a);
//}
//
//@fragment
//fn fs_bubbles(in: VertexOutput) -> @location(0) vec4<f32> {
//    let len = length(in.position);
//
//    if (1.0 < len) {
//      return vec4<f32>(0.0);
//    }
//
//    //var dist = 1.0 / len * 0.5;
//    //var dist = pow(dist, 1.5);
//
//    //let col = dist * in.color.xyz;
//    //let glow = vec3<f32>(1.0 - exp(-col));
//    
//    var dist = pow(len, 3.0);
//
//    var col = dist * in.color;
//    return 1.0 - exp(-col);
//}
