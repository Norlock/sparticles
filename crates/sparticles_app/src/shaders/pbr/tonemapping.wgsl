struct Exposure {
    u_Exposure: f32
}

//uniform u_Exposure: f32;


const GAMMA: f32 = 2.2;
const INV_GAMMA: f32 = 1.0 / GAMMA;


// sRGB => XYZ => D65_2_D60 => AP1 => RRT_SAT
const ACES_IN: mat3x3<f32> = mat3x3(
    0.59719, 0.07600, 0.02840,
    0.35458, 0.90834, 0.13383,
    0.04823, 0.01566, 0.83777
);


// ODT_SAT => XYZ => D60_2_D65 => sRGB
const ACES_OUT: mat3x3<f32> = mat3x3(
    1.60475, -0.10208, -0.00327,
    -0.53108, 1.10813, -0.07276,
    -0.07367, -0.00605, 1.07602
);


// linear to sRGB approximation
// see http://chilliant.blogspot.com/2012/08/srgb-approximations-for-hlsl.html
fn linear_to_srgb(color: vec3<f32>) -> vec3<f32> {
    return pow(color, vec3(INV_GAMMA));
}

fn srgb_to_linear(srgbIn: vec4<f32>) -> vec4<f32> {
    return vec4(pow(srgbIn.xyz, vec3(GAMMA)), srgbIn.w);
}


fn aces_narkowicz(col: vec3<f32>) -> vec3<f32> {
    let a = 2.51f;
    let b = 0.03f;
    let c = 2.43f;
    let d = 0.59f;
    let e = 0.14f;

    return saturate((col * (a * col + b)) / (col * (c * col + d) + e));
}


// ACES filmic tone map approximation
// see https://github.com/TheRealMJP/BakingLab/blob/master/BakingLab/ACES.hlsl
fn rrt_and_odt_fit(color: vec3<f32>) -> vec3<f32> {
    let a = color * (color + 0.0245786) - 0.000090537;
    let b = color * (0.983729 * color + 0.4329510) + 0.238081;
    return a / b;
}


// tone mapping 
fn aces_hill(input: vec3<f32>) -> vec3<f32> {
    var color = ACES_IN * input;

    // Apply RRT and ODT
    color = rrt_and_odt_fit(color);
    color = ACES_OUT * color;

    return saturate(color);
}

//fn linearToneMapping(color: vec3<f32>) -> vec3<f32> {
//    let exposure = 1.;
//    return clamp(exposure * color, 0., 1.);
//}
//
//fn simpleReinhardToneMapping(in: vec3<f32>) -> vec3<f32> {
//    let exposure = 1.5;
//    return in * exposure / (1. + in / exposure);
//}
//
//fn lumaBasedReinhardToneMapping(color: vec3<f32>) -> vec3<f32> {
//    let luma = dot(color, vec3(0.2126, 0.7152, 0.0722));
//    let toneMappedLuma = luma / (1. + luma);
//    return color * toneMappedLuma / luma;
//}
//
//fn whitePreservingLumaBasedReinhardToneMapping(color: vec3<f32>) -> vec3<f32> {
//    let white = 2.;
//    let luma = dot(color, vec3(0.2126, 0.7152, 0.0722));
//    let toneMappedLuma = luma * (1. + luma / (white * white)) / (1. + luma);
//    return color * toneMappedLuma / luma;
//}
//
//fn RomBinDaHouseToneMapping(color: vec3<f32>) -> vec3<f32> {
//    return exp(-1.0 / (2.72 * color + 0.15));
//}

fn uchimura(x: vec3<f32>) -> vec3<f32> {
    let P = vec3(1.0);  // max display brightness
    let a = vec3(1.0);  // contrast
    let m = vec3(0.22); // linear section start
    let l = vec3(0.4);  // linear section length
    let c = vec3(1.33); // black
    let b = vec3(0.0);  // pedestal

    let one = vec3(1.0);

    // Uchimura 2017, "HDR theory and practice"
    // Math: https://www.desmos.com/calculator/gslcdxvipg
    // Source: https://www.slideshare.net/nikuque/hdr-theory-and-practicce-jp
    let l0 = ((P - m) * l) / a;
    let L0 = m - m / a;
    let L1 = m + (one - m) / a;
    let S0 = m + l0;
    let S1 = m + a * l0;
    let C2 = (a * P) / (P - S1);
    let CP = -C2 / P;

    let w0 = one - smoothstep(vec3(0.), m, x);
    let w2 = step(m + l0, x);
    let w1 = one - w0 - w2;

    let T = m * pow(x / m, c) + b;
    let S = P - (P - S1) * exp(CP * (x - S0));
    let L = m + a * (x - m);

    return T * w0 + L * w1 + S * w2;
}

fn lottes(x: vec3<f32>) -> vec3<f32> {
    // Lottes 2016, "Advanced Techniques and Optimization of HDR Color Pipelines"
    let a = vec3(1.6);
    let d = vec3(0.977);
    let hdrMax = vec3(8.0);
    let midIn = vec3(0.18);
    let midOut = vec3(0.267);

    // Can be precomputed
    let b = (-pow(midIn, a) + pow(hdrMax, a) * midOut) / ((pow(hdrMax, a * d) - pow(midIn, a * d)) * midOut);
    let c = (pow(hdrMax, a * d) * pow(midIn, a) - pow(hdrMax, a) * pow(midIn, a * d) * midOut) / ((pow(hdrMax, a * d) - pow(midIn, a * d)) * midOut);

    return pow(x, a) / (pow(x, a * d) * b + c);
}

fn tonemap(in: vec3<f32>, tonemap: u32) -> vec3<f32> {
    switch tonemap {
        case 0u {
            return aces_narkowicz(in);
        }
        case 1u {
            return aces_hill(in);
        } 
        case 2u {
            return uchimura(in);
        }
        case 3u {
            return lottes(in);
        }
        default {
            return vec3(1.0);
        }
    }

    //#if TONEMAP_ACES_HILL_EXPOSURE_BOOST
    //    // boost exposure as discussed in https://github.com/mrdoob/three.js/pull/19621
    //    // this factor is based on the exposure correction of Krzysztof Narkowicz in his
    //    // implemetation of ACES tone mapping
    //color /= 0.6;
    //color = aces_hill(color);
    //#endif
}
