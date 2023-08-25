struct BlurUniform {
    horizontal: bool,
}

//uniform sampler2D image;
@group(0) @binding(0)
var t_image: texture_2d<f32>;

@fragment
fn fs_main(in: vec2<f32>) -> vec4<f32> {             
    var weight = array<f32, 5>(0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216 );
    var tex_offset: vec2<f32>  = 1.0 / textureSize(t_image, 0); // gets size of single texel
    var result: vec3<f32> = texture(image, TexCoords).rgb * weight[0]; // current fragment's contribution

    if horizontal {
        for(var i = 1; i < 5; ++i) {
            result += texture(image, TexCoords + vec2<f32>(tex_offset.x * f32(i), 0.0)).rgb * weight[i];
            result += texture(image, TexCoords - vec2<f32>(tex_offset.x * f32(i), 0.0)).rgb * weight[i];
        }
    }
    else {
        for(var i = 1; i < 5; ++i) {
            result += texture(image, TexCoords + vec2<f32>(0.0, tex_offset.y * f32(i))).rgb * weight[i];
            result += texture(image, TexCoords - vec2<f32>(0.0, tex_offset.y * f32(i))).rgb * weight[i];
        }
    }

    return vec4<f32>(result, 1.0);
}
