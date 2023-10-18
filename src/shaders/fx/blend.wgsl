@group(0) @binding(0) var write_fx: binding_array<texture_storage_2d<rgba8unorm, write>, 32>;
@group(0) @binding(1) var read_fx: binding_array<texture_2d<f32>, 32>;

@group(1) @binding(0) var<uniform> fx_meta: FxMeta; 

@compute
@workgroup_size(8, 8, 1)
fn additive(@builtin(global_invocation_id) pos_u32: vec3<u32>) {
    let pos = vec2<f32>(pos_u32.xy);
    let fx_size = vec2<f32>(textureDimensions(read_fx[fx_meta.in_idx]));

    if any(fx_size < pos) {
        return;
    }

    let frame_color = textureLoad(read_fx[0], pos_u32.xy, 0).rgb;
    let fx_color = textureLoad(read_fx[fx_meta.in_idx], vec2<u32>(pos) / fx_meta.in_downscale, 0).rgb;

    let result = frame_color + fx_color;

    textureStore(write_fx[0], pos_u32.xy, vec4<f32>(result, 1.0));
}
