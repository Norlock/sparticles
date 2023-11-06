struct Blend {
    io_mix: f32,
}

@group(0) @binding(0) var fx_tex: binding_array<texture_storage_2d<rgba16float, read_write>, 16>;
@group(1) @binding(0) var<uniform> fx_io: FxIO; 
@group(2) @binding(0) var<uniform> blend: Blend; 

fn get_color(pos: vec2<i32>) -> vec3<f32> {
    let in_size = vec2<u32>(fx_io.in_size_x, fx_io.in_size_y);
    if any(pos < vec2<i32>(0)) || any(vec2<i32>(in_size) <= pos) {
        return vec3<f32>(0.);
    } else {
        return textureLoad(fx_tex[fx_io.in_idx], pos).rgb;
    }
}

fn in_color(in_pos: vec2<i32>) -> vec3<f32> {
    // The filter kernel is applied with a radius, specified in texture
    // coordinates, so that the radius will vary across mip resolutions.
    let x = 2;
    let y = 2;

    // Take 9 samples around current texel:
    // a - b - c
    // d - e - f
    // g - h - i
    // === ('e' is the current texel) ===
    let a_pos = vec2<i32>(in_pos.x - x, in_pos.y + y);
    let b_pos = vec2<i32>(in_pos.x,     in_pos.y + y);
    let c_pos = vec2<i32>(in_pos.x + x, in_pos.y + y);
    let d_pos = vec2<i32>(in_pos.x - x, in_pos.y);
    let e_pos = vec2<i32>(in_pos.x,     in_pos.y);
    let f_pos = vec2<i32>(in_pos.x + x, in_pos.y);
    let g_pos = vec2<i32>(in_pos.x - x, in_pos.y - y);
    let h_pos = vec2<i32>(in_pos.x,     in_pos.y - y);
    let i_pos = vec2<i32>(in_pos.x + x, in_pos.y - y);

    var a = get_color(a_pos);
    var b = get_color(b_pos);
    var c = get_color(c_pos);

    var d = get_color(d_pos);
    var e = get_color(e_pos);
    var f = get_color(f_pos);

    var g = get_color(g_pos);
    var h = get_color(h_pos);
    var i = get_color(i_pos);

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
@workgroup_size(16, 16, 1)
fn lerp_upscale_blend(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pos = global_id.xy;

    if any(vec2<u32>(fx_io.out_size_x, fx_io.out_size_y) < pos) {
        return;
    }

    let downscale = (fx_io.in_downscale / fx_io.out_downscale);
    let in_pos = pos / downscale;
    let out_pos = pos;

    let in_color = in_color(vec2<i32>(in_pos));
    let out_color = textureLoad(fx_tex[fx_io.out_idx], out_pos).rgb;

    let result = mix(in_color, out_color, blend.io_mix);

    textureStore(fx_tex[fx_io.out_idx], out_pos, vec4<f32>(result, 1.0));
}

@compute
@workgroup_size(16, 16, 1)
fn lerp_simple_blend(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pos = global_id.xy;

    if any(vec2<u32>(fx_io.out_size_x, fx_io.out_size_y) < pos) {
        return;
    }

    let in_color = textureLoad(fx_tex[fx_io.in_idx], pos).rgb;
    let out_color = textureLoad(fx_tex[fx_io.out_idx], pos).rgb;

    let result = mix(in_color, out_color, blend.io_mix);

    textureStore(fx_tex[fx_io.out_idx], pos, vec4<f32>(result, 1.0));
}

@compute
@workgroup_size(16, 16, 1)
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
