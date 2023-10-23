struct ColorCorrection {
    gamma: f32,
    contrast: f32,
    brightness: f32,
}

@group(0) @binding(0) var write_fx: binding_array<texture_storage_2d<rg11b10float, write>, 32>;
@group(0) @binding(1) var read_fx: binding_array<texture_2d<f32>, 32>;

@group(1) @binding(0) var<uniform> globals: ColorCorrection; 
@group(1) @binding(1) var<uniform> fx_meta: FxMeta; 

@compute
@workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let pos = global_invocation_id.xy;
    let size = vec2<u32>(textureDimensions(read_fx[0]));

    if any(size < pos) {
        return;
    }

    var out = textureLoad(read_fx[fx_meta.in_idx], pos, 0).rgb;

    out = pow(out, vec3<f32>(1.0 / globals.gamma));
    out = (out - 0.5) * globals.contrast + 0.5 + globals.brightness;

    textureStore(write_fx[fx_meta.out_idx], pos, out);
}
