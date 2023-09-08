// input
@group(0) @binding(1) var input_texture: texture_2d<f32>;

// fx dst
@group(1) @binding(0) var dst_texture: texture_storage_2d<rgba8unorm, write>;
// fx src
@group(1) @binding(1) var src_texture: texture_2d<f32>;

@group(2) @binding(0) var<uniform> global: Bloom; 
@group(2) @binding(1) var depth_texture: texture_2d<f32>;

fn apply_blur(is_horizontal: bool, pos: vec2<i32>) {
    let size = vec2<i32>(textureDimensions(src_texture));

    if any(size < pos) {
        return;
    }

    // delta offset
    var current = textureLoad(src_texture, pos, 0).rgb;
    var largest = vec3<f32>(0.);

    for (var x = -1; x < 2; x++) {
        for (var y = -1; y < 2; y++) {
            var offset = pos + vec2<i32>(x, y);

            if all(0 < offset) && all(offset < size) {
                var nb = textureLoad(src_texture, pos, 0).rgb;
                largest = max(largest, nb);
            }
        }
    }

    textureStore(dst_texture, pos, vec4<f32>(current, 1.0));
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

    let start_x = pos.x * global.kernel_size;
    let end_x = start_x + global.kernel_size;
    let start_y = pos.y * global.kernel_size;
    let end_y = start_y + global.kernel_size;

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

    if any(global.br_treshold < result) {
        textureStore(dst_texture, pos, vec4<f32>(result, 1.0));
    } else {
        textureStore(dst_texture, pos, vec4<f32>(0.0));
    }
}

