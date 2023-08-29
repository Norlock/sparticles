@group(0) @binding(0) var dst_texture: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(1) var src_texture: texture_2d<f32>;
@group(1) @binding(0) var fx_texture: texture_2d<f32>;

@compute
@workgroup_size(8, 8, 1)
fn additive(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let pos = global_invocation_id.xy;
    let size = vec2<u32>(textureDimensions(dst_texture));

    if any(size < pos) {
        return;
    }

    let src_color = textureLoad(src_texture, pos, 0).rgb;
    let fx_color = textureLoad(fx_texture, pos, 0).rgb;

    let result = src_color + fx_color;

    textureStore(dst_texture, pos, vec4<f32>(result, 1.0));
}
