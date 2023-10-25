@group(0) @binding(0) var write_fx: binding_array<texture_storage_2d<rgba16float, write>, 32>;
@group(0) @binding(1) var read_fx: binding_array<texture_2d<f32>, 32>;
@group(1) @binding(0) var<uniform> fx_io: FxIO; 

@compute
@workgroup_size(8, 8, 1)
fn additive(@builtin(global_invocation_id) pos: vec3<u32>) {
    var blend_size = vec2<u32>(
        vec2<f32>(textureDimensions(read_fx[fx_io.out_idx])) / fx_io.out_downscale
    );

    if any(blend_size < pos.xy) {
        return;
    }

    let blend_pos = pos.xy;

    // TODO check if downscale is below 1.0
    let fx_downscale = fx_io.in_downscale / fx_io.out_downscale;
    let fx_pos = vec2<u32>(vec2<f32>(blend_pos) / fx_downscale);

    let blend_color = textureLoad(read_fx[fx_io.out_idx], blend_pos, 0).rgb;
    let fx_color = textureLoad(read_fx[fx_io.in_idx], fx_pos, 0).rgb;

    textureStore(write_fx[fx_io.out_idx], blend_pos, vec4<f32>(blend_color + fx_color, 1.0));
}
