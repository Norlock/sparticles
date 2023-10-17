@group(0) @binding(0) var output: binding_array<texture_storage_2d<rgba8unorm, write>, 32>;
@group(0) @binding(2) var frame_texture: texture_2d<f32>;

@compute
@workgroup_size(8, 8, 1)
fn init(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    //let fx_size = vec2<f32>(textureDimensions(output));
    let pos = global_invocation_id.xy;
    let size = textureDimensions(frame_texture);

    if any(size < pos) {
        return;
    }

    //if fx_size.x < globals.view_width || fx_size.y < globals.view_height {
    //    let scale_x = globals.view_width / fx_size.x;
    //    let scale_y = globals.view_height / fx_size.y;

    //    let x = i32(pos.x / scale_x);
    //    let y = i32(pos.y / scale_y);

    //    return textureLoad(output, vec2<i32>(x, y), 0, 0);
    //} else {
    //}
    let out = textureLoad(frame_texture, vec2<i32>(pos.xy), 0);

    textureStore(output[0], pos, out);
}
