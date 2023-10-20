@group(0) @binding(2) var fx_blend: texture_storage_2d<rgba8unorm, read_write>;
@group(0) @binding(3) var frame_texture: texture_2d<f32>;

@compute
@workgroup_size(8, 8, 1)
fn init(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let pos = global_invocation_id.xy;
    let size = textureDimensions(frame_texture);

    if any(size < pos) {
        return;
    }

    let out = textureLoad(frame_texture, vec2<i32>(pos.xy), 0);

    textureStore(fx_blend, pos, out);
}
