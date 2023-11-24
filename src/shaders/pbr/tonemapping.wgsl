struct Exposure {
    u_Exposure: f32
}

//uniform u_Exposure: f32;


const GAMMA: f32 = 2.2;
const INV_GAMMA: f32 = 1.0 / GAMMA;


// sRGB => XYZ => D65_2_D60 => AP1 => RRT_SAT
const ACESInMat: mat3x3<f32> = mat3x3(
    0.59719, 0.07600, 0.02840,
    0.35458, 0.90834, 0.13383,
    0.04823, 0.01566, 0.83777
);


// ODT_SAT => XYZ => D60_2_D65 => sRGB
const ACESOutMat: mat3x3<f32> = mat3x3(
    1.60475, -0.10208, -0.00327,
    -0.53108, 1.10813, -0.07276,
    -0.07367, -0.00605, 1.07602
);


// linear to sRGB approximation
// see http://chilliant.blogspot.com/2012/08/srgb-approximations-for-hlsl.html
fn linearTosRGB(color: vec3<f32>) -> vec3<f32> {
    return pow(color, vec3(INV_GAMMA));
}

fn sRGBToLinear(srgbIn: vec4<f32>) -> vec4<f32> {
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
fn RRTAndODTFit(color: vec3<f32>) -> vec3<f32> {
    let a = color * (color + 0.0245786) - 0.000090537;
    let b = color * (0.983729 * color + 0.4329510) + 0.238081;
    return a / b;
}


// tone mapping 
fn toneMapACES_Hill(input: vec3<f32>) -> vec3<f32> {
    var color = ACESInMat * input;

    // Apply RRT and ODT
    color = RRTAndODTFit(color);
    color = ACESOutMat * color;

    return saturate(color);
}


//fn toneMap(color: vec3<f32>) -> vec3<f32> {
//    color *= u_Exposure;
//
//    #ifdef TONEMAP_ACES_NARKOWICZ
//    color = toneMapACES_Narkowicz(color);
//    #endif
//
//    #ifdef TONEMAP_ACES_HILL
//    color = toneMapACES_Hill(color);
//    #endif
//
//    #ifdef TONEMAP_ACES_HILL_EXPOSURE_BOOST
//        // boost exposure as discussed in https://github.com/mrdoob/three.js/pull/19621
//        // this factor is based on the exposure correction of Krzysztof Narkowicz in his
//        // implemetation of ACES tone mapping
//    color /= 0.6;
//    color = toneMapACES_Hill(color);
//    #endif
//
//    return linearTosRGB(color);
//}
