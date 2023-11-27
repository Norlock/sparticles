struct VertexInput {
    @builtin(vertex_index) vert_idx: u32,
    @builtin(instance_index) instance_idx: u32,
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) world_pos: vec4<f32>,
    @location(2) uv: vec2<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    let p = particles[in.instance_idx];

    if is_decayed(em, p) {
        var out: VertexOutput;
        out.clip_position = vec4(camera.position, 0.0) - 1000.;
        return out;
    }

    var out: VertexOutput;
    out.color = p.color;
    out.world_pos = vec4<f32>(p.model.w.xyz + in.position * p.scale, 1.0);
    out.clip_position = camera.view_proj * out.world_pos;
    out.uv = in.uv;

    return out;
}

@fragment
fn fs_circle(in: VertexOutput) -> FragmentOutput {
    let v_pos = in.uv * 2. - 1.;

    let len = length(v_pos);
    let diff_color = textureSample(albedo_tex, albedo_s, in.uv).rgb;

    if 1.0 < len {
        discard;
    }

    let normal = sqrt(1. - v_pos.x * v_pos.x - v_pos.y * v_pos.y);

    var out: FragmentOutput;
    out.color = vec4<f32>(in.color.rgb * diff_color * normal, in.color.a);

    if any(camera.bloom_treshold < out.color.rgb) {
        out.split = out.color;
    }

    return out;
}

@fragment
fn fs_model(in: VertexOutput) -> FragmentOutput {
    // TODO aanpassen!
    let v_pos = in.uv * 2. - 1.;

    let len = length(v_pos);
    let texture_color = textureSample(albedo_tex, albedo_s, in.uv).rgb;

    if 1.0 < len {
        discard;
    }

    let x = v_pos.x;
    let y = v_pos.y * -1.;
    let normal = sqrt(1. - x * x - y * y);

    var out: FragmentOutput;
    out.color = vec4<f32>(texture_color.rgb * in.color.rgb * normal, 1.0);

    if any(camera.bloom_treshold < out.color.rgb) {
        out.split = out.color;
    }

    return out;
}

//var strength = 1.0 - len * 0.7;
//var color = in.color.rgb * strength;

//var effect = create_layers(v_pos, normal, idx, em.elapsed_sec);
//effect *= 1. - 0.02 / color.rgb;
//effect += 0.5;
//out.color = vec4<f32>(texture_color.rgb * in.color.rgb * effect, 1.0);
