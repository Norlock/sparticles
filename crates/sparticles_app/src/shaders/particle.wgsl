struct VertexInput {
    @builtin(vertex_index) vert_idx: u32,
    @builtin(instance_index) instance_idx: u32,
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) world_pos: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) normal: vec3<f32>,
    @location(4) tangent: vec3<f32>,
    @location(5) bitangent: vec3<f32>,
}

@group(3) @binding(0) var<storage, read> light_particles: array<Particle>;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    let p = particles[in.instance_idx];

    if is_decayed(em, p) {
        var out: VertexOutput;
        out.clip_position = vec4(camera.position, 0.0) - 1000.;
        return out;
    }

    var out: VertexOutput;
    out.uv = in.uv;
    out.color = p.color;
    out.world_pos = (p.model * vec4(in.position, 1.0)).xyz * p.scale;
    out.normal = in.normal;
    out.tangent = in.tangent.xyz;
    out.bitangent = in.bitangent;
    out.clip_position = camera.view_proj * vec4(out.world_pos, 1.0);

    return out;
}

fn apply_pbr(in: VertexOutput, N: vec3<f32>, WN: vec3<f32>, ALB: vec3<f32>) -> FragmentOutput {
    let albedo = pow(ALB, vec3(2.2));
    let metallic_roughness = textureSample(metal_rough_tex, metal_rough_s, in.uv).rg;
    let metallic = metallic_roughness.r;
    let roughness = metallic_roughness.g;
    let ao = textureSample(ao_tex, ao_s, in.uv).r;
    let emissive = pow(textureSample(emissive_tex, emissive_s, in.uv).rgb, vec3(2.2));

    let F0 = mix(vec3(0.04), albedo, metallic);
    let V = normalize(camera.position.xyz - in.world_pos);
    var Lo = vec3(0.0);
    var Diff = vec3(0.0);

    for (var i = 0u; i < arrayLength(&light_particles); i++) {
        let light = light_particles[i];
        let light_pos = light.model.w.xyz;
        let light_col = light.color.rgb;

        // calculate per-light radiance
        let L = normalize(light_pos - in.world_pos);
        let H = normalize(V + L);

        let distance = length(light_pos - in.world_pos);
        let radiance = light_col / (distance * distance);

        // Cook-Torrance BRDF
        let NDF = distribution_ggx(N, H, roughness);
        let G = geometry_smith(N, V, L, roughness);
        let F = fresnel_schlick(max(dot(H, V), 0.0), F0);

        let numerator = NDF * G * F;

        let NdotL = max(dot(N, L), 0.0);
        let denominator = 4.0 * max(dot(N, V), 0.0) * NdotL;
        let specular = numerator / (denominator + 0.0001);
        let kD = (vec3(1.0) - F) * (1.0 - metallic);

        Diff += max(dot(WN, L), 0.0) * radiance;
        Lo += (kD * albedo / PI + specular) * radiance * NdotL;
    }

    var out: FragmentOutput;

    let color = tonemap(Diff * vec3(0.4) * albedo * ao + Lo + emissive, camera.tonemap);

    out.color = vec4(linear_to_srgb(color), 1.0);

    if any(camera.bloom_treshold < out.color.rgb) {
        out.split = out.color;
    }

    return out;
}

@fragment
fn fs_model(in: VertexOutput) -> FragmentOutput {
    let tangent_normal = textureSample(normal_tex, normal_s, in.uv).rgb * 2.0 - 1.0;
    let TBN = mat3x3(in.normal, in.tangent, in.bitangent);

    let N = normalize(TBN * tangent_normal);
    let albedo = textureSample(albedo_tex, albedo_s, in.uv).rgb;

    return apply_pbr(in, N, in.normal, albedo);
}

@fragment
fn fs_circle(in: VertexOutput) -> FragmentOutput {
    let v_pos = in.uv * 2. - 1.;
    let texture_color = textureSample(albedo_tex, albedo_s, in.uv);

    if 1.0 < length(v_pos) {
        discard;
    }

    let x = v_pos.x;
    let y = v_pos.y * -1.;
    let WN = (vec4(x, y, sqrt(1. - x * x - y * y), 0.) * camera.view).xyz;

    return apply_pbr(in, WN, WN, in.color.rgb);
}
