@group(0) @binding(0) var dst_texture: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(1) var src_texture: texture_2d<f32>;
@group(0) @binding(2) var frame_texture: texture_2d<f32>;
@group(1) @binding(0) var<uniform> bloom: Bloom; 

fn apply_blur(is_horizontal: bool, pos: vec2<u32>) {
    let size = vec2<u32>(textureDimensions(dst_texture));

    if (any(size < pos)) {
        return;
    }

    // Weight to apply for gaussian blur
    var weight = array<f32, 5>(
        bloom.weight_1, 
        bloom.weight_2, 
        bloom.weight_3, 
        bloom.weight_4,
        bloom.weight_5
    );

    // Size of a single texel
    let tex_offset = 1.0 / vec2<f32>(size);
    var frame_texel = textureLoad(src_texture, pos, 0);
    var result = frame_texel.rgb * weight[0];
    
    // Blur
    if is_horizontal {
        for(var i = 1u; i < 5u; i++) {
            result += textureLoad(src_texture, pos + vec2<u32>(u32(tex_offset.x * f32(i)), 0u), 0).rgb * weight[i];
            result += textureLoad(src_texture, pos - vec2<u32>(u32(tex_offset.x * f32(i)), 0u), 0).rgb * weight[i];
        }
    } else {
        for(var i = 1u; i < 5u; i++) {
            result += textureLoad(src_texture, pos + vec2<u32>(0u, u32(tex_offset.y * f32(i))), 0).rgb * weight[i];
            result += textureLoad(src_texture, pos - vec2<u32>(0u, u32(tex_offset.y * f32(i))), 0).rgb * weight[i];
        }
    }
    textureStore(dst_texture, pos, vec4<f32>(result, 1.0));
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
fn split(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let pos = global_invocation_id.xy;
    let size = vec2<u32>(textureDimensions(dst_texture));

    if (any(size < pos)) {
        return;
    }

    let tex_offset = 1.0 / vec2<f32>(size);
    var frame_texel = textureLoad(frame_texture, pos, 0);

    // TODO vec3 in uniform
    let brightness = dot(frame_texel.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    
    if bloom.br_treshold < brightness {
        textureStore(dst_texture, pos, vec4<f32>(frame_texel.rgb, 1.0));
    } else {
        textureStore(dst_texture, pos, vec4<f32>(0.0));
    }
}

