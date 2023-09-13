// input
@group(0) @binding(1) var input_texture: texture_2d<f32>;

// fx dst
@group(1) @binding(0) var dst_texture: texture_storage_2d<rgba8unorm, write>;
// fx src
@group(1) @binding(1) var src_texture: texture_2d<f32>;

@group(2) @binding(0) var<uniform> globals: Bloom; 
@group(2) @binding(1) var depth_texture: texture_2d<f32>;

fn apply_blur(is_horizontal: bool, pos: vec2<i32>) {
    let size = vec2<i32>(textureDimensions(src_texture));

    if any(size < pos) {
        return;
    }

    var result = vec3<f32>(0.);
    var weight = 0.;

    let edge = globals.radius;
    let edge_f32 = f32(edge);
    let sub = length(vec2<f32>(edge_f32) * globals.weight) * globals.decay + globals.weight;

    for (var x = -edge; x <= edge; x++) {
        for (var y =  -edge; y <= edge; y++) {
            var offset = vec2<i32>(x, y);
            var nb_pos = pos + offset;

            if all(0 < nb_pos) && all(nb_pos < size) {
                var nb_col = textureLoad(src_texture, nb_pos, 0).rgb;
                var coeff = sub - (length(vec2<f32>(offset)) * globals.weight * globals.decay);
                result += nb_col * coeff;
                weight += coeff;
            }
        }
    }

    textureStore(dst_texture, pos, vec4<f32>(result / weight, 1.0));
}

@compute
@workgroup_size(8, 8, 1)
fn blur_x(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    apply_blur(true, vec2<i32>(global_invocation_id.xy));
}

@compute
@workgroup_size(8, 8, 1)
fn blur_y(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    apply_blur(false, vec2<i32>(global_invocation_id.xy));
}

@compute
@workgroup_size(8, 8, 1)
fn split_bloom(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let pos = global_invocation_id.xy;
    let fx_size = vec2<u32>(textureDimensions(dst_texture));
    let frame_size = vec2<u32>(textureDimensions(input_texture));

    if any(fx_size < pos) {
        return;
    }

    let start_x = pos.x * globals.kernel_size;
    let end_x = start_x + globals.kernel_size;
    let start_y = pos.y * globals.kernel_size;
    let end_y = start_y + globals.kernel_size;

    var weight = 0u;
    var result = vec3<f32>(0.0);

    for (var x = start_x; x < end_x; x++) {
        for (var y = start_y; y < end_y; y++) {
            if x < frame_size.x && y < frame_size.y {
                result += textureLoad(input_texture, vec2<u32>(x, y), 0).rgb;
                weight++;
            }
        }
    }

    // Averaging out
    result /= f32(weight); 

    if any(globals.br_treshold < result) {
        // Convert to HDR
        textureStore(dst_texture, pos, vec4<f32>(result * 20., 1.0));
    } else {
        textureStore(dst_texture, pos, vec4<f32>(0.0));
    }
}

