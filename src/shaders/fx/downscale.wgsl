@group(0) @binding(0) var write_fx: binding_array<texture_storage_2d<rgba16float, write>, 32>;
@group(0) @binding(1) var read_fx: binding_array<texture_2d<f32>, 32>;

@group(1) @binding(0) var<uniform> fx_io: FxIO; 

@compute
@workgroup_size(8, 8, 1)
fn downscale(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let pos = global_invocation_id.xy;

    let input_size = vec2<u32>(
        vec2<f32>(textureDimensions(read_fx[0])) / fx_io.in_downscale
    );

    if any(input_size < pos) {
        return;
    }

    // Ping pong asymetric
    if fx_io.in_idx != fx_io.out_idx {
        var copy = textureLoad(read_fx[fx_io.in_idx], pos, 0);
        textureStore(write_fx[fx_io.in_idx], pos, copy);
    }

    let output_size = vec2<u32>(
        vec2<f32>(textureDimensions(read_fx[0])) / fx_io.out_downscale
    );

    if any(output_size < pos) {
        return;
    }

    let downscale = u32(fx_io.out_downscale / fx_io.in_downscale);
    
    let start_x = pos.x * downscale;
    let end_x = start_x + downscale;
    let start_y = pos.y * downscale;
    let end_y = start_y + downscale;

    var weight = 0.;
    var result = vec3<f32>(0.0);

    for (var x = start_x; x < end_x; x++) {
        for (var y = start_y; y < end_y; y++) {
            if x < input_size.x && y < input_size.y {
                result += textureLoad(read_fx[fx_io.in_idx], vec2<u32>(x, y), 0).rgb;
                weight += 1.0;
            }
        }
    }

    // Averaging out
    result /= weight;

    textureStore(write_fx[fx_io.out_idx], pos, vec4<f32>(result, 1.0));
}
