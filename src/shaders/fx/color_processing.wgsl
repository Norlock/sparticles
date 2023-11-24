struct ColorCorrection {
    gamma: f32,
    contrast: f32,
    brightness: f32,
}

@group(0) @binding(0) var fx_tex: binding_array<texture_storage_2d<rgba16float, read_write>, 16>;

@group(1) @binding(0) var<uniform> fx_io: FxIO; 
@group(2) @binding(0) var<uniform> globals: ColorCorrection; 

fn gamma(col: vec3<f32>) -> vec3<f32> {
    return pow(col, vec3<f32>(1.0 / globals.gamma));
}

@compute
@workgroup_size(16, 16, 1)
fn general(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pos = global_id.xy;

    if any(vec2<u32>(fx_io.out_size_x, fx_io.out_size_y) < pos) {
        return;
    }

    var out = textureLoad(fx_tex[fx_io.in_idx], pos).rgb;

    out = gamma(out);
    out = (out - 0.5) * globals.contrast + 0.5 + globals.brightness;

    textureStore(fx_tex[fx_io.out_idx], pos, vec4<f32>(out, 1.0));
}

@compute
@workgroup_size(16, 16, 1)
fn tonemap(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pos = global_id.xy;

    if any(vec2<u32>(fx_io.out_size_x, fx_io.out_size_y) < pos) {
        return;
    }

    let hdr = textureLoad(fx_tex[fx_io.in_idx], pos).rgb;      

    // tone mapping
    var sdr = aces_narkowicz(hdr);

    // also gamma correct
    sdr = gamma(sdr);

    textureStore(fx_tex[fx_io.out_idx], pos, vec4<f32>(sdr, 1.0));
}
