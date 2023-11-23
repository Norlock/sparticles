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
    @location(3) tangent: vec4<f32>,
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

struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @location(1) split: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    let p = particles[in.instance_idx];

    if p.lifetime == -1. {
        var out: VertexOutput;
        out.world_pos = camera.view_pos.xyz - 1000.;
        return out;
    }

    var out: VertexOutput;
    out.uv = in.uv;
    out.color = p.color;
    out.world_pos = (p.model * vec4(in.position, 1.0)).xyz * p.scale;
    out.normal = normalize(in.normal);
    out.tangent = normalize(in.tangent.xyz);
    out.bitangent = normalize(cross(out.normal, out.tangent) * in.tangent.w);
    out.clip_position = camera.view_proj * vec4(out.world_pos, 1.0);

    return out;
}

@group(1) @binding(0) var albedo_tex: texture_2d<f32>;
@group(1) @binding(1) var albedo_s: sampler;
@group(1) @binding(2) var normal_tex: texture_2d<f32>;
@group(1) @binding(3) var normal_s: sampler;

@group(1) @binding(4) var metal_rough_tex: texture_2d<f32>;
@group(1) @binding(5) var metal_rough_s: sampler;
// TODO use emissive tex
@group(1) @binding(8) var ao_tex: texture_2d<f32>;
@group(1) @binding(9) var ao_s: sampler;

// ----------------------------------------------------------------------------
// Easy trick to get tangent-normals to world-space to keep PBR code simplified.
// Don't worry if you don't get what's going on; you generally want to do normal 
// mapping the usual way for performance anyways; I do plan make a note of this 
// technique somewhere later in the normal mapping tutorial.
fn get_normal_from_map(in: VertexOutput) -> vec3<f32> {
    let tangent_normal = textureSample(normal_tex, normal_s, in.uv).xyz * 2.0 - 1.0;
    let TBN = mat3x3(in.normal, in.tangent, in.bitangent);

    return normalize(TBN * tangent_normal);
}

fn fresnel_schlick(cos_theta: f32, F0: vec3<f32>) -> vec3<f32> {
    return F0 + (1.0 - F0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

fn distribution_ggx(N: vec3<f32>, H: vec3<f32>, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let NdotH = max(dot(N, H), 0.0);
    let NdotH2 = NdotH * NdotH;

    let num = a2;
    var denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return num / denom;
}

fn geometry_schlick_ggx(NdotV: f32, roughness: f32) -> f32 {
    let r = (roughness + 1.0);
    let k = (r * r) / 8.0;

    let num = NdotV;
    let denom = NdotV * (1.0 - k) + k;

    return num / denom;
}

fn geometry_smith(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, roughness: f32) -> f32 {
    let NdotV = max(dot(N, V), 0.0);
    let NdotL = max(dot(N, L), 0.0);
    let ggx2 = geometry_schlick_ggx(NdotV, roughness);
    let ggx1 = geometry_schlick_ggx(NdotL, roughness);

    return ggx1 * ggx2;
}

@fragment
fn fs_model(in: VertexOutput) -> FragmentOutput {
    let albedo = pow(textureSample(albedo_tex, albedo_s, in.uv).rgb, vec3<f32>(2.2));
    let metallic_roughness = textureSample(metal_rough_tex, metal_rough_s, in.uv).rg;
    let metallic = metallic_roughness.r;
    let roughness = metallic_roughness.g;
    let ao = textureSample(ao_tex, ao_s, in.uv).r;

    let N = get_normal_from_map(in);
    let V = normalize(camera.view_pos.xyz - in.world_pos);

    let F0 = mix(vec3(0.04), albedo, metallic);

    var Lo = vec3(0.0);

    for (var i = 0u; i < arrayLength(&light_particles); i++) {
        let light = light_particles[i];
        let light_pos = light.model.w.xyz;
        let light_col = light.color.rgb;

        // calculate per-light radiance
        let L = normalize(light_pos - in.world_pos);
        let H = normalize(V + L);

        let distance = length(light_pos - in.world_pos);
        let attenuation = 1.0 / (distance * distance);
        let radiance = light_col * attenuation;

        // Cook-Torrance BRDF
        let NDF = distribution_ggx(N, H, roughness);
        let G = geometry_smith(N, V, L, roughness);
        let F = fresnel_schlick(max(dot(H, V), 0.0), F0);

        let numerator = NDF * G * F;
        let denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0);
        let specular = numerator / (denominator + 0.0001);

        var kD = vec3(1.0) - F;
        kD *= 1.0 - metallic;

        let NdotL = max(dot(N, L), 0.0);

        Lo += (kD * albedo / PI + specular) * radiance * NdotL;
    }

    var out: FragmentOutput;

    let ambient = vec3(0.03) * albedo * ao;
    var color = ambient + Lo;

    // HDR tone mapping
    color = color / (color + vec3(1.0));
    // Gamma correct
    color = pow(color, vec3(1.0 / 2.2));

    out.color = vec4(color, 1.0);

    if any(vec3<f32>(camera.bloom_treshold) < out.color.rgb) {
        out.split = out.color;
    }

    return out;
}

@fragment
fn fs_circle(in: VertexOutput) -> FragmentOutput {
    let v_pos = in.uv * 2. - 1.;

    let texture_color = textureSample(albedo_tex, albedo_s, in.uv);

    if 1.0 < length(v_pos) {
        discard;
    }

    let x = v_pos.x;
    let y = v_pos.y;

    let normal = vec4<f32>(x, y, sqrt(1. - x * x - y * y), 0.);
    let world_normal = (normal * camera.view).xyz;

    var result = vec3<f32>(0.0);

    for (var i = 0u; i < arrayLength(&light_particles); i++) {
        let light = light_particles[i];
        let light_pos = light.model.w.xyz;

        let distance = length(light_pos - in.world_pos.xyz);
        let strength = 1.0 - distance * 0.04;
        let ambient_color = acesFilm(light.color.rgb) * strength;

        if strength <= 0.0 {
            continue;
        }

        let light_dir = normalize(light_pos - in.world_pos.xyz);
        let view_dir = normalize(camera.view_pos.xyz - in.world_pos.xyz);
        let half_dir = normalize(view_dir + light_dir);

        let diffuse_strength = max(dot(world_normal, light_dir), 0.0);
        let diffuse_color = diffuse_strength * ambient_color;

        let specular_strength = pow(max(dot(world_normal, half_dir), 0.0), 32.0);
        let specular_color = specular_strength * ambient_color;

        result += diffuse_color + specular_color;
    }

    var out: FragmentOutput;
    out.color = vec4<f32>(in.color.rgb * texture_color.rgb * result, in.color.a);

    if any(vec3<f32>(camera.bloom_treshold) < out.color.rgb) {
        out.split = out.color;
    }

    return out;
}
