@group(0) @binding(1) var read_fx: binding_array<texture_2d<f32>, 32>;
@group(0) @binding(2) var fx_blend: texture_storage_2d<rgba8unorm, read_write>;

@group(1) @binding(0) var<uniform> fx_meta: FxMeta; 

@compute
@workgroup_size(8, 8, 1)
fn additive(@builtin(global_invocation_id) pos: vec3<u32>) {
    let fx_size = textureDimensions(fx_blend);

    if any(fx_size < pos.xy) {
        return;
    }

    let frame_color = textureLoad(fx_blend, pos.xy).rgb;
    let fx_color = textureLoad(read_fx[fx_meta.in_idx], pos.xy, 0).rgb;

    textureStore(fx_blend, pos.xy, vec4<f32>(frame_color + fx_color, 1.0));
}
