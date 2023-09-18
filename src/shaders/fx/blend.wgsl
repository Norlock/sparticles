@group(0) @binding(1) var fx_texture: texture_2d<f32>;
@group(1) @binding(1) var frame_texture: texture_2d<f32>;
@group(1) @binding(0) var out_texture: texture_storage_2d<rgba8unorm, write>;

@compute
@workgroup_size(8, 8, 1)
fn additive(@builtin(global_invocation_id) pos_u32: vec3<u32>) {
    let pos = vec2<f32>(pos_u32.xy);
    let fx_size = vec2<f32>(textureDimensions(fx_texture));
    let frame_size = vec2<f32>(textureDimensions(frame_texture));

    if any(frame_size < pos) {
        return;
    }

    let scale = frame_size.x / fx_size.x;

    let frame_color = textureLoad(frame_texture, pos_u32.xy, 0).rgb;
    let fx_color = textureLoad(fx_texture, vec2<u32>(pos / scale), 0).rgb;

    let result = frame_color + fx_color;

    textureStore(out_texture, pos_u32.xy, vec4<f32>(result, 1.0));
}
