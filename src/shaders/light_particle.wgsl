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

//@fragment
//fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
//
//    let v_pos = in.v_pos.xy;
//    let idx = in.v_pos.z;
//
//    let circle_len = length(v_pos * 2.);
//    let len = length(v_pos);
//    let x = v_pos.x;
//    let y = v_pos.y;
//    var strength = 1.0 - len;
//
//    if (circle_len <= 1.0) {
//        let normal = vec4<f32>(x, y, sqrt(1. - x * x - y * y), 0.);
//        let world_normal = normal * camera.view;
//
//        let light_dir = vec3<f32>(camera.view_pos.xyz);
//        let diffuse_strength = max(dot(world_normal.xyz, light_dir), 0.0);
//        var arg = pi() * atan((y + 2. ) / (x + 2.)) * sin(idx + em.elapsed_sec * 0.4) +
//            sin(2.0 + em.elapsed_sec * 0.1);
//
//        let noise = create_layers(v_pos, arg, em.elapsed_sec, em.delta_sec);
//
//        let color = vec4<f32>(in.color.rgb * diffuse_strength * noise, 1.0);
//        return (color + 1.) / 2.;
//    } else if (1. < len) {
//        discard;
//    } else {
//        var angle = atan(y / x);
//
//        //var pattern = sin(idx + em.elapsed_sec * 2.5 + angle * 15.);
//        //pattern += sin((idx + em.elapsed_sec - 1.) * 14. + angle * 23.);
//        //pattern += sin(idx + em.elapsed_sec * 1.4 + 11. + angle * 19.);
//
//        //let mix = max(abs(pattern), 1.0);
//
//        //let sin_dist = abs(sin(angle * 2. * 4.) * len);
//        
//        var strength = 1.0 - len;
//        //let d_sun = smoothstep(0.0, 1.0, distance);
//
//        //let strength = 1.0 - distance * 0.04;
//        
//        var color = in.color * strength;
//
//        //color *= mix(1.0, 0.0, 1. - strength);
//
//        return vec4<f32>(color);
//
//        //let r = max(min(d_sun * in.color.r, 1.0), 0.);
//        //let g = max(min(d_sun * in.color.g * 0.9, 1.0), 0.);
//        //let b = max(min(d_sun * in.color.b * 0.9, 1.0), 0.);
//        //let a = max(min(d_sun * 0.9, 1.0), 0.);
//
//        //if a < 0.1 || (r < 0.1 && b < 0.1 && a < 0.1) {
//        //    discard;
//        //} else {
//        //    return vec4<f32>(r, g, b, a);
//        //}
//    }
//}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {

    let v_pos = in.v_pos.xy;
    let idx = in.v_pos.z;

    let len = length(v_pos);

    if 1.0 < len {
        discard;
    }

    var strength = 1.0 - len * 0.7;
    var color = in.color.rgb * strength;

    let x = v_pos.x;
    let y = v_pos.y;
    let normal = sqrt(1. - x * x - y * y);


    var effect = create_layers(v_pos, idx, em.elapsed_sec);
    effect *= 1. - 0.02 / in.color.rgb;
    effect += 0.5;

    //effect = 0.02 / effect;

    //effect.r = effect.b;
    //effect.b = 0.;
    
        
    return vec4<f32>(color * normal * effect, 1.0);
}
