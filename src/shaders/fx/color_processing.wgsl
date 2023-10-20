struct ColorCorrection {
    gamma: f32,
    contrast: f32,
    brightness: f32,
}

@group(0) @binding(0) var write_fx: binding_array<texture_storage_2d<rgba8unorm, write>, 32>;
@group(0) @binding(1) var read_fx: binding_array<texture_2d<f32>, 32>;
@group(0) @binding(2) var fx_blend: texture_storage_2d<rgba8unorm, read_write>;

@group(1) @binding(0) var<uniform> globals: ColorCorrection; 
@group(1) @binding(1) var<uniform> fx_meta: FxMeta; 

@compute
@workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let pos = global_invocation_id.xy;
    let size = textureDimensions(fx_blend);

    if any(size < pos) {
        return;
    }

    var out: vec3<f32>;

    if fx_meta.in_idx == -1 {
        out = textureLoad(fx_blend, pos).rgb;
    } else {
        out = textureLoad(read_fx[fx_meta.in_idx], pos, 0).rgb;
    }

    out = pow(out, vec3<f32>(1.0 / globals.gamma));
    out = (out - 0.5) * globals.contrast + 0.5 + globals.brightness;

    if fx_meta.out_idx == -1 {
        textureStore(fx_blend, pos, vec4<f32>(out, 1.0));
    } else {
        textureStore(write_fx[fx_meta.out_idx], pos, vec4<f32>(out, 1.0));
    }
}