@group(0) @binding(0) var write_fx: binding_array<texture_storage_2d<rgba8unorm, write>, 32>;
@group(0) @binding(2) var frame_texture: texture_2d<f32>;

@compute
@workgroup_size(8, 8, 1)
fn init(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let pos = global_invocation_id.xy;
    let size = textureDimensions(frame_texture);

    if any(size < pos) {
        return;
    }

    let out = textureLoad(frame_texture, vec2<i32>(pos), 0);

    textureStore(write_fx[0], pos, out);
}
