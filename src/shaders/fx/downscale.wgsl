@group(0) @binding(0) var fx_tex: binding_array<texture_storage_2d<rgba16float, read_write>, 16>;
@group(1) @binding(0) var<uniform> fx_io: FxIO; 

@compute
@workgroup_size(8, 8, 1)
fn downscale(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let pos = global_invocation_id.xy;

    if any(vec2<u32>(fx_io.out_size_x, fx_io.out_size_y) < pos) {
        return;
    }

    let downscale = 2u; // Always 2u from previous
    let start_x = pos.x * downscale;
    let end_x = start_x + downscale;
    let start_y = pos.y * downscale;
    let end_y = start_y + downscale;

    var weight = 0.;
    var result = vec3<f32>(0.0);

    for (var x = start_x; x < end_x; x++) {
        for (var y = start_y; y < end_y; y++) {
            if x < fx_io.in_size_x && y < fx_io.in_size_y {
                result += textureLoad(fx_tex[fx_io.in_idx], vec2<u32>(x, y)).rgb;
                weight += 1.0;
            }
        }
    }

    // Averaging out
    result /= weight;

    textureStore(fx_tex[fx_io.out_idx], pos, vec4<f32>(result, 1.0));
}
