@group(0) @binding(0) var<uniform> camera: Camera;
@group(2) @binding(0) var<storage, read> particles: array<Particle>;
@group(2) @binding(2) var<uniform> em: Emitter; 

struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @location(1) split: vec4<f32>,
}

struct MaterialUniform {
    emissive_factor: vec3<f32>,
    emissive_strength: f32,
    ior: f32,
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
@group(1) @binding(10) var<uniform> mat_globals: MaterialUniform;

// BRDF approximation sample count (higher is better quality and slower)
const BRDF_SAMPLE_COUNT: i32 = 64;

// reflection convolution sample count (higher is better quality and slower)
const REFLECTION_SAMPLE_COUNT: i32 = 256;

// irradiance convolution step (lower is better quality and slower)
const sampleDelta: f32 = 0.1;

fn fresnel_schlick(HdotV: f32, F0: vec3<f32>) -> vec3<f32> {
    return F0 + (1. - F0) * pow(clamp(1. - HdotV, 0., 1.), 5.);
}

fn lambert(color: vec3<f32>) -> vec3<f32> {
    return color / PI;
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

fn geometry_smith(NdotV: f32, NdotL: f32, roughness: f32) -> f32 {
    let ggx2 = geometry_schlick_ggx(NdotV, roughness);
    let ggx1 = geometry_schlick_ggx(NdotL, roughness);

    return ggx1 * ggx2;
}

//fn ImportanceSampleGGX(Xi: vec2<f32>, N: vec3<f32>, roughness: f32, tangent: vec3<f32>, bitangent: vec3<f32>) -> vec3<f32> {
//    var a = roughness * roughness;
//
//    var phi = 2.0 * PI * Xi.x;
//    var cosTheta = sqrt((1.0 - Xi.y) / (1.0 + (a * a - 1.0) * Xi.y));
//    var sinTheta = sqrt(1.0 - cosTheta * cosTheta);
//
//    var H = vec3(
//        cos(phi) * sinTheta,
//        sin(phi) * sinTheta,
//        cosTheta
//    );
//
//    var up;
//    if abs(N.z) < 0.999 {
//        up = vec3(0.0, 0.0, 1.0);
//    } else {
//        up = vec3(1.0, 0.0, 0.0);
//    }
//
//    var sampleVec = tangent * H.x + bitangent * H.y + N * H.z;
//    return normalize(sampleVec);
//}  

fn brdf(diffuse: vec3<f32>, specular: vec3<f32>) -> vec3<f32> {
    return diffuse + specular;
}
