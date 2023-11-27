@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(2) @binding(0) var<storage, read> particles: array<Particle>;
@group(2) @binding(2) var<uniform> em: Emitter; 


struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @location(1) split: vec4<f32>,
}

@group(1) @binding(0) var albedo_tex: texture_2d<f32>;
@group(1) @binding(1) var albedo_s: sampler;
@group(1) @binding(2) var normal_tex: texture_2d<f32>;
@group(1) @binding(3) var normal_s: sampler;

@group(1) @binding(4) var metal_rough_tex: texture_2d<f32>;
@group(1) @binding(5) var metal_rough_s: sampler;
@group(1) @binding(6) var emissive_tex: texture_2d<f32>;
@group(1) @binding(7) var emissive_s: sampler;
@group(1) @binding(8) var ao_tex: texture_2d<f32>;
@group(1) @binding(9) var ao_s: sampler;

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

