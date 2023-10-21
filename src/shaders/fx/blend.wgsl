@group(0) @binding(0) var write_fx: binding_array<texture_storage_2d<rgba8unorm, write>, 32>;
@group(0) @binding(1) var read_fx: binding_array<texture_2d<f32>, 32>;

@group(1) @binding(0) var<uniform> fx_meta: FxMeta; 

@compute
@workgroup_size(8, 8, 1)
fn additive(@builtin(global_invocation_id) pos: vec3<u32>) {
    let fx_size = textureDimensions(write_fx[0]);

    if any(fx_size < pos.xy) {
        return;
    }

    let blend_color = textureLoad(read_fx[fx_meta.out_idx], pos.xy, 0).rgb;
    let fx_color = textureLoad(read_fx[fx_meta.in_idx], pos.xy, 0).rgb;

    textureStore(write_fx[fx_meta.out_idx], pos.xy, vec4<f32>(blend_color + fx_color, 1.0));
}
