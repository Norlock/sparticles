@group(0) @binding(0) 
var<uniform> camera: CameraUniform;

@group(1) @binding(0) 
var<storage, read> particles: array<Particle>;

@group(1) @binding(2) var<uniform> em: Emitter; 

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    // normal
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

fn outer_ring(len: f32, color: vec3<f32>, v_pos: vec2<f32>, idx: f32) -> vec4<f32> {
    var angle = atan(v_pos.y / v_pos.x);
    var delta = 1.0 - (len - 0.9) * 10.;
    var sum = vec4<f32>(0.0);

    for (var i = 1; i < 3; i++) {
        var formula: f32;
        if i % 2 == 1 {
            formula = sin(em.elapsed_sec * 5.5);
        } else {
            formula = sin(em.elapsed_sec * 4.5);
        }

        var glow = sin(8. * angle * f32(i + 2) + idx + formula);

        glow *= smoothstep(0.1, 1.0, delta);

        sum += glow;
    }

    return sum;
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let v_pos = in.v_pos.xy;
    let len = length(v_pos);

    if 1.0 < len {
        discard;
    }

    var strength = 1.0 - len * 0.7;
    var color = in.color.rgb * strength;

    let x = v_pos.x;
    let y = v_pos.y;
    let idx = in.v_pos.z;

    let normal = sqrt(1. - x * x - y * y);

    var effect = create_layers(v_pos, normal, idx, em.elapsed_sec);
    effect *= 1. - 0.02 / color.rgb;
    effect += 0.5;

    return vec4<f32>(color * effect * normal, 1.0);

    //if (0.9 < len) {
    //    return outer_ring(len, color.rgb, v_pos, idx);
    //} else {
    //    return vec4<f32>(color * effect * normal, 1.0);
    //}
}
