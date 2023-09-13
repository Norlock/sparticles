struct ColorCorrection {
    gamma: f32,
    contrast: f32,
    brightness: f32,
}

@group(0) @binding(0) var dst_texture: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(1) var src_texture: texture_2d<f32>;
@group(1) @binding(0) var<uniform> globals: ColorCorrection; 

@compute
@workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let pos = global_invocation_id.xy;
    let size = textureDimensions(src_texture);

    if any(size < pos) {
        return;
    }

    var out = textureLoad(src_texture, pos, 0).rgb;

    out = pow(out, vec3<f32>(1.0 / globals.gamma));
    out = (out - 0.5) * globals.contrast + 0.5 + globals.brightness;
    textureStore(dst_texture, pos, vec4<f32>(out, 1.0));
}
