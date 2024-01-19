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
@group(4) @binding(1) var terrain_map: texture_cube<f32>;
@group(4) @binding(2) var terrain_s: sampler;

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
    let albedo = pow(ALB * material.albedo_col.rgb, vec3(2.2));
    let metallic_roughness = textureSample(metal_rough_tex, metal_rough_s, in.uv).rg;
    let metallic = metallic_roughness.r;
    let roughness = metallic_roughness.g;
    let ao = textureSample(ao_tex, ao_s, in.uv).r;
    let specular_val = textureSample(specular_tex, specular_s, in.uv).a * material.specular_factor;
    let specular_color_val = textureSample(specular_color_tex, specular_color_s, in.uv).rgb * material.specular_color_factor;
    let emissive = material.emissive_factor * srgb_to_linear(textureSample(emissive_tex, emissive_s, in.uv)).rgb * material.emissive_strength;

    let F0 = mix(vec3(0.04), albedo, metallic);

    // View direction
    let V = normalize(camera.position - in.world_pos);
    let NdotV = max(dot(N, V), 0.0);

    let shininess = 0.01;
    let world_reflect = reflect(-V, N);
    let world_refract = refract(-V, N, 1. / material.ior);

    var reflection = textureSample(terrain_map, terrain_s, world_reflect).rgb;
    //var refraction = textureSample(terrain_map, terrain_s, world_refract).rgb;
    //var env_color = mix(reflection, refraction, 0.5);

    var Lo = vec3(0.0);
    var Shad = vec3(0.0);

    for (var i = 0u; i < arrayLength(&light_particles); i++) {
        let light = light_particles[i];
        let light_pos = light.model.w.xyz;
        let light_col = light.color.rgb;

        // Light direction
        let L = normalize(light_pos - in.world_pos);
        // Half direction
        let H = normalize(V + L);

        let NdotL = max(dot(N, L), 0.0);
        let HdotV = max(dot(H, V), 0.0);
        let WNdotL = max(dot(WN, L), 0.0);

        let distance = length(light_pos - in.world_pos);
        let radiance = light_col / (distance * distance);

        // Cook-Torrance BRDF
        let NDF = distribution_ggx(N, H, roughness);
        let G = geometry_smith(NdotV, NdotL, roughness);
        let F = fresnel_schlick(HdotV, F0);

        let numerator = NDF * G * F;

        let denominator = 4.0 * NdotV * NdotL;
        let specular = numerator / (denominator + 0.0001);

        let kD = (vec3(1.0) - F) * (1.0 - metallic);
        let lambert = lambert(kD * albedo);

        Shad += WNdotL * radiance;
        Lo += (lambert + specular) * radiance * NdotL;
    }

    // --------------- ENVIRONMENT --------------------
    var kS = fresnel_schlick(NdotV, F0);
    var kD = 1.0 - kS;

    kD *= 1.0 - metallic;

    var diffuse = reflection * albedo * ao;
    var ambient = (kD * diffuse);
    // --------------- ENVIRONMENT --------------------

    var hdr = Shad * vec3(0.4) * albedo * ao + Lo + emissive + ambient;

    var color = tonemap(hdr, camera.tonemap);

    color = linear_to_srgb(color).rgb;

    var out: FragmentOutput;

    out.color = vec4(color, 1.0);

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
