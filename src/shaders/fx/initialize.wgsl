@group(0) @binding(0) var fx_texture: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var frame_texture: texture_2d<f32>;

@compute
@workgroup_size(8, 8, 1)
fn init(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let pos = global_invocation_id.xy;
    let fx_size = vec2<u32>(textureDimensions(fx_texture));

    if any(fx_size < pos) {
        return;
    }

    var out = textureLoad(frame_texture, pos, 0);
    textureStore(fx_texture, pos, out);
}
