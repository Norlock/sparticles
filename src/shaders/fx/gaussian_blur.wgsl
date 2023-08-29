@group(0) @binding(1) var frame_texture: texture_2d<f32>;
@group(0) @binding(2) var depth_texture: texture_2d<f32>;
// fx dst
@group(1) @binding(0) var dst_texture: texture_storage_2d<rgba8unorm, write>;
// fx src
@group(1) @binding(1) var src_texture: texture_2d<f32>;
@group(2) @binding(0) var<uniform> global: Bloom; 

fn get_offset(is_horizontal: bool) -> vec2<u32> {
    if is_horizontal {
        return vec2<u32>(1u, 0u);
    } else {
        return vec2<u32>(0u, 1u);
    }
}

fn apply_blur(is_horizontal: bool, pos: vec2<u32>) {
    let size = vec2<u32>(textureDimensions(src_texture));

    if any(size < pos) {
        return;
    }

    // Radius must be uniform field 
    let radius_f32 = f32(global.radius);

    var result = textureLoad(src_texture, pos, 0).rgb;
    var total_weight = pow(radius_f32, 2.);

    // delta offset
    var offset = get_offset(is_horizontal);

    for(var i = 1u; i < global.radius; i++) {
        var weight = pow(radius_f32 - f32(i), 2.); // Quadratic blur fall off
        var i_pos = pos + offset * i; // increased pos
        var d_pos = pos - offset * i; // decreased pos

        if all(i_pos < size) {
            result += textureLoad(src_texture, i_pos, 0).rgb * weight;
            total_weight += weight;
        }

        if all(0u <= d_pos) {
            result += textureLoad(src_texture, d_pos, 0).rgb * weight;
            total_weight += weight;
        }
    }

    textureStore(dst_texture, pos, vec4<f32>(result / f32(total_weight), 1.0));
}

@compute
@workgroup_size(8, 8, 1)
fn blur_x(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    apply_blur(true, global_invocation_id.xy);
}

@compute
@workgroup_size(8, 8, 1)
fn blur_y(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    apply_blur(false, global_invocation_id.xy);
}

@compute
@workgroup_size(8, 8, 1)
fn split_bloom(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let pos = global_invocation_id.xy;
    let fx_size = vec2<u32>(textureDimensions(dst_texture));
    let frame_size = vec2<u32>(textureDimensions(frame_texture));

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
                result += textureLoad(frame_texture, vec2<u32>(x, y), 0).rgb;
                weight++;
            }
        }
    }

    // Averaging out
    result /= f32(weight); 

    // TODO vec3 in uniform
    let brightness = dot(result, vec3<f32>(0.2126, 0.7152, 0.0722));
    
    if global.br_treshold < brightness {
        textureStore(dst_texture, pos, vec4<f32>(result, 1.0));
    } else {
        textureStore(dst_texture, pos, vec4<f32>(0.0));
    }
}

