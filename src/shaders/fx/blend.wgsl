struct Blend {
    io_mix: f32,
}

@group(0) @binding(0) var fx_tex: binding_array<texture_storage_2d<rgba16float, read_write>, 16>;
@group(1) @binding(0) var<uniform> fx_io: FxIO; 
@group(2) @binding(0) var<uniform> blend: Blend; 

fn in_color(in_pos: vec2<u32>) -> vec3<f32> {
    let zero = vec2<u32>(0u);

    // The filter kernel is applied with a radius, specified in texture
    // coordinates, so that the radius will vary across mip resolutions.
    let x = 2u;
    let y = 2u;

    // Take 9 samples around current texel:
    // a - b - c
    // d - e - f
    // g - h - i
    // === ('e' is the current texel) ===
    let a_pos = vec2<u32>(in_pos.x - x, in_pos.y + y);
    let b_pos = vec2<u32>(in_pos.x,     in_pos.y + y);
    let c_pos = vec2<u32>(in_pos.x + x, in_pos.y + y);
    let d_pos = vec2<u32>(in_pos.x - x, in_pos.y);
    let e_pos = vec2<u32>(in_pos.x,     in_pos.y);
    let f_pos = vec2<u32>(in_pos.x + x, in_pos.y);
    let g_pos = vec2<u32>(in_pos.x - x, in_pos.y - y);
    let h_pos = vec2<u32>(in_pos.x,     in_pos.y - y);
    let i_pos = vec2<u32>(in_pos.x + x, in_pos.y - y);

    var a = textureLoad(fx_tex[fx_io.in_idx], a_pos).rgb;
    var b = textureLoad(fx_tex[fx_io.in_idx], b_pos).rgb;
    var c = textureLoad(fx_tex[fx_io.in_idx], c_pos).rgb;

    var d = textureLoad(fx_tex[fx_io.in_idx], d_pos).rgb;
    var e = textureLoad(fx_tex[fx_io.in_idx], e_pos).rgb;
    var f = textureLoad(fx_tex[fx_io.in_idx], f_pos).rgb;

    var g = textureLoad(fx_tex[fx_io.in_idx], g_pos).rgb;
    var h = textureLoad(fx_tex[fx_io.in_idx], h_pos).rgb;
    var i = textureLoad(fx_tex[fx_io.in_idx], i_pos).rgb;

    // Apply weighted distribution, by using a 3x3 tent filter:
    //  1   | 1 2 1 |
    // -- * | 2 4 2 |
    // 16   | 1 2 1 |
    var upsample = e*4.0;
    upsample += (b+d+f+h)*2.0;
    upsample += (a+c+g+i);
    upsample *= 1.0 / 16.0;

    //return upsample;
    return upsample;
}

@compute
@workgroup_size(8, 8, 1)
fn lerp_blend(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pos = global_id.xy;

    if any(vec2<u32>(fx_io.out_size_x, fx_io.out_size_y) < pos) {
        return;
    }

    let downscale = (fx_io.in_downscale / fx_io.out_downscale);
    let in_pos = pos / downscale;
    let out_pos = pos;

    let in_color = in_color(in_pos);
    let out_color = textureLoad(fx_tex[fx_io.out_idx], out_pos).rgb;

    let result = mix(in_color, out_color, blend.io_mix);

    textureStore(fx_tex[fx_io.out_idx], out_pos, vec4<f32>(result, 1.0));
}

@compute
@workgroup_size(8, 8, 1)
fn add_blend(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pos = global_id.xy;

    if any(vec2<u32>(fx_io.out_size_x, fx_io.out_size_y) < pos) {
        return;
    }

    let downscale = (fx_io.in_downscale / fx_io.out_downscale);
    let in_pos = pos / downscale;
    let out_pos = pos;

    let in_color = textureLoad(fx_tex[fx_io.in_idx], in_pos).rgb;
    let out_color = textureLoad(fx_tex[fx_io.out_idx], out_pos).rgb;

    textureStore(fx_tex[fx_io.out_idx], out_pos, vec4<f32>(in_color + out_color, 1.0));
}
