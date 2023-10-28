@group(0) @binding(0) var write_fx: binding_array<texture_storage_2d<rgba16float, write>, 32>;
@group(0) @binding(1) var read_fx: binding_array<texture_2d<f32>, 32>;

@group(1) @binding(0) var<uniform> fx_io: FxIO; 
@group(2) @binding(0) var<uniform> globals: GaussianBlur; 

fn apply_blur(pos: vec2<i32>, offset: vec2<i32>) {
    let size = vec2<i32>(
        vec2<f32>(textureDimensions(read_fx[0])) / fx_io.out_downscale
    );

    if any(size < pos) {
        return;
    }

    let edge = globals.radius;
    let two_ss = 2. * globals.sigma * globals.sigma;
    let lhs = 1. / sqrt(two_ss * pi());
    var result = vec3<f32>(0.);

    for (var i = -edge; i < edge; i++) {
        var tex_offset = offset * i;
        var tex_pos = pos + tex_offset;

        if (all(vec2<i32>(0) < tex_pos) && all(tex_pos < size)) {
            var t_off = vec2<f32>(tex_offset * tex_offset);
            var rhs = exp(-(t_off.x + t_off.y) / two_ss);

            var coeff = lhs * rhs * globals.intensity;
            var col = textureLoad(read_fx[fx_io.out_idx], tex_pos, 0).rgb;

            result += col * coeff;
        }
    }

    textureStore(write_fx[fx_io.out_idx], pos, vec4<f32>(result, 1.0));
}

@compute
@workgroup_size(8, 8, 1)
fn apply_blur_x(@builtin(global_invocation_id) pos: vec3<u32>) {
    apply_blur(vec2<i32>(pos.xy), vec2<i32>(1, 0));
}

@compute
@workgroup_size(8, 8, 1)
fn apply_blur_y(@builtin(global_invocation_id) pos: vec3<u32>) {
    apply_blur(vec2<i32>(pos.xy), vec2<i32>(0, 1));
}

@compute
@workgroup_size(8, 8, 1)
fn split_bloom(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let pos = global_invocation_id.xy;

    var frame_size = vec2<u32>(textureDimensions(read_fx[fx_io.in_idx]));

    if any(frame_size < pos) {
        return;
    }

    let copy = textureLoad(read_fx[fx_io.in_idx], pos, 0);
    let hdr = copy.rgb * globals.hdr_mul;

    // Ping pong asymetric
    if fx_io.in_idx != fx_io.out_idx {
        textureStore(write_fx[fx_io.in_idx], pos, copy);
    }

    if any(vec3<f32>(globals.br_treshold) < hdr) {
        // Convert to HDR
        textureStore(write_fx[fx_io.out_idx], pos, vec4<f32>(hdr, 1.0));
    } else {
        textureStore(write_fx[fx_io.out_idx], pos, vec4<f32>(0.0));
    }
}
