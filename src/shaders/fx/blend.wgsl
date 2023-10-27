struct Blend {
    io_mix: f32,
}

@group(0) @binding(0) var write_fx: binding_array<texture_storage_2d<rgba16float, write>, 32>;
@group(0) @binding(1) var read_fx: binding_array<texture_2d<f32>, 32>;
@group(1) @binding(0) var<uniform> fx_io: FxIO; 
@group(2) @binding(0) var<uniform> globals: Blend; 

fn in_color(in_pos: vec2<u32>) -> vec3<f32> {
    // The filter kernel is applied with a radius, specified in texture
    // coordinates, so that the radius will vary across mip resolutions.
    let x = 2u;
    let y = 2u;

    // Take 9 samples around current texel:
    // a - b - c
    // d - e - f
    // g - h - i
    // === ('e' is the current texel) ===
    var a = textureLoad(read_fx[fx_io.in_idx], vec2<u32>(in_pos.x - x, in_pos.y + y), 0).rgb;
    var b = textureLoad(read_fx[fx_io.in_idx], vec2<u32>(in_pos.x,     in_pos.y + y), 0).rgb;
    var c = textureLoad(read_fx[fx_io.in_idx], vec2<u32>(in_pos.x + x, in_pos.y + y), 0).rgb;

    var d = textureLoad(read_fx[fx_io.in_idx], vec2<u32>(in_pos.x - x, in_pos.y), 0).rgb;
    var e = textureLoad(read_fx[fx_io.in_idx], vec2<u32>(in_pos.x,     in_pos.y), 0).rgb;
    var f = textureLoad(read_fx[fx_io.in_idx], vec2<u32>(in_pos.x + x, in_pos.y), 0).rgb;

    var g = textureLoad(read_fx[fx_io.in_idx], vec2<u32>(in_pos.x - x, in_pos.y - y), 0).rgb;
    var h = textureLoad(read_fx[fx_io.in_idx], vec2<u32>(in_pos.x,     in_pos.y - y), 0).rgb;
    var i = textureLoad(read_fx[fx_io.in_idx], vec2<u32>(in_pos.x + x, in_pos.y - y), 0).rgb;

    // Apply weighted distribution, by using a 3x3 tent filter:
    //  1   | 1 2 1 |
    // -- * | 2 4 2 |
    // 16   | 1 2 1 |
    var upsample = e*4.0;
    upsample += (b+d+f+h)*2.0;
    upsample += (a+c+g+i);
    upsample *= 1.0 / 16.0;

    return upsample;
}

@compute
@workgroup_size(8, 8, 1)
fn blend(@builtin(global_invocation_id) pos: vec3<u32>) {
    var out_size = vec2<u32>(
        vec2<f32>(textureDimensions(read_fx[fx_io.out_idx])) / fx_io.out_downscale
    );

    if any(out_size < pos.xy) {
        return;
    }

    let out_pos = pos.xy;

    // TODO check if downscale is below 1.0
    let fx_downscale = fx_io.in_downscale / fx_io.out_downscale;
    let in_pos = vec2<u32>(vec2<f32>(out_pos) / fx_downscale);

    let in_color = in_color(in_pos);
    let out_color = textureLoad(read_fx[fx_io.out_idx], out_pos, 0).rgb;

    let result = mix(in_color, out_color, globals.io_mix);
    textureStore(write_fx[fx_io.out_idx], out_pos, vec4<f32>(result, 1.0));
}

@compute
@workgroup_size(8, 8, 1)
fn add(@builtin(global_invocation_id) pos: vec3<u32>) {
    var out_size = vec2<u32>(
        vec2<f32>(textureDimensions(read_fx[fx_io.out_idx])) / fx_io.out_downscale
    );

    if any(out_size < pos.xy) {
        return;
    }

    let out_pos = pos.xy;

    // TODO check if downscale is below 1.0
    let fx_downscale = fx_io.in_downscale / fx_io.out_downscale;
    let in_pos = vec2<u32>(vec2<f32>(out_pos) / fx_downscale);

    let in_color = textureLoad(read_fx[fx_io.in_idx], in_pos, 0).rgb;
    let out_color = textureLoad(read_fx[fx_io.out_idx], out_pos, 0).rgb;

    textureStore(write_fx[fx_io.out_idx], out_pos, vec4<f32>(in_color + out_color, 1.0));
}
