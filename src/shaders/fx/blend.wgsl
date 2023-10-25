struct Blend {
    io_mix: f32,
}

@group(0) @binding(0) var write_fx: binding_array<texture_storage_2d<rgba16float, write>, 32>;
@group(0) @binding(1) var read_fx: binding_array<texture_2d<f32>, 32>;
@group(1) @binding(0) var<uniform> fx_io: FxIO; 
@group(2) @binding(0) var<uniform> blend: Blend; 

@compute
@workgroup_size(8, 8, 1)
fn additive(@builtin(global_invocation_id) pos: vec3<u32>) {
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

    let result = mix(in_color, out_color, blend.io_mix);
    textureStore(write_fx[fx_io.out_idx], out_pos, vec4<f32>(out_color + result, 1.0));
}
