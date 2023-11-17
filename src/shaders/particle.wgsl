@group(0) @binding(0) 
var<uniform> camera: CameraUniform;

@group(2) @binding(0) 
var<storage, read> particles: array<Particle>;

@group(3) @binding(0) 
var<storage, read> light_particles: array<Particle>;

struct VertexInput {
    @builtin(vertex_index) vert_idx: u32,
    @builtin(instance_index) instance_idx: u32,
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_space: vec4<f32>,
    @location(1) color: vec4<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) normal: vec3<f32>,
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @location(1) split: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    let p = particles[in.instance_idx];

    if (p.lifetime == -1.) {
        var out: VertexOutput;
        out.clip_position = camera.view_pos - 1000.;
        return out;
    }
    
    let world_space: vec4<f32> = 
        vec4<f32>(
            p.pos_size.xyz + 
            in.position * 
            p.pos_size.w, 1.0
    );

    var out: VertexOutput;
    out.clip_position = camera.view_proj * world_space;
    out.world_space = world_space;
    out.color = p.color;
    out.uv = in.uv;
    out.normal = in.normal;

    return out;
}

@group(1) @binding(0) var diff_tex: texture_2d<f32>;
@group(1) @binding(1) var norm_tex: texture_2d<f32>;
@group(1) @binding(4) var s: sampler;

@fragment
fn fs_circle(in: VertexOutput) -> FragmentOutput {
    let v_pos = in.uv * 2. - 1.;

    let texture_color = textureSample(diff_tex, s, in.uv);

    if (1.0 < length(v_pos)) {
        discard;
    }

    let x = v_pos.x;
    let y = v_pos.y;

    let normal = vec3<f32>(x, y, sqrt(1. - x * x - y * y));
    let world_normal = (vec4<f32>(normal, 0.) * camera.view).xyz;

    var result = vec3<f32>(0.0);


    for (var i = 0u; i < arrayLength(&light_particles); i++) { 
        let light = light_particles[i];
        let light_pos = light.pos_size.xyz;

        let distance = length(light_pos - in.world_space.xyz);
        let strength = 1.0 - distance * 0.04;
        let ambient_color = acesFilm(light.color.rgb) * strength;

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

    var out: FragmentOutput;
    out.color = vec4<f32>(texture_color.rgb, in.color.a);

    if any(vec3<f32>(camera.bloom_treshold) < out.color.rgb) {
        out.split = out.color;
    }  

    return out;
}

@fragment
fn fs_model(in: VertexOutput) -> FragmentOutput {

    let diff_color = textureSample(diff_tex, s, in.uv).rgb;
    //let norm_color = textureSample(norm_tex, s, in.uv).rgb;

    var result = vec3<f32>(0.0);

    // Create the lighting vectors
    //let tangent_normal = object_normal.xyz * 2.0 - 1.0;

    for (var i = 0u; i < arrayLength(&light_particles); i++) { 
        let light = light_particles[i];
        let light_pos = light.pos_size.xyz;

        let distance = length(light_pos - in.world_space.xyz);
        let strength = 1.0 - distance * 0.04;
        let ambient_color = acesFilm(light.color.rgb) * strength;

        if (strength <= 0.0) {
            continue;
        }

        let light_dir = normalize(light_pos - in.world_space.xyz);
        let view_dir = normalize(camera.view_pos.xyz - in.world_space.xyz);
        let half_dir = normalize(view_dir + light_dir);

        let diffuse_strength = max(dot(in.normal, light_dir), 0.0);
        let diffuse_color = diffuse_strength * ambient_color;

        let specular_strength = pow(max(dot(in.normal, half_dir), 0.0), 32.0);
        let specular_color = specular_strength * ambient_color;

        result += diffuse_color + specular_color;
    }

    var out: FragmentOutput;
    out.color = vec4<f32>(result * diff_color, in.color.a);

    if any(vec3<f32>(camera.bloom_treshold) < out.color.rgb) {
        out.split = out.color;
    }  

    return out;
}
