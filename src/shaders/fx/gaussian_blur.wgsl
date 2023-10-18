@group(0) @binding(0) var write_fx: binding_array<texture_storage_2d<rgba8unorm, write>, 32>;
@group(0) @binding(1) var read_fx: binding_array<texture_2d<f32>, 32>;
@group(0) @binding(2) var fx_blend: texture_storage_2d<rgba8unorm, read_write>;

@group(1) @binding(0) var<uniform> fx_meta: FxMeta; 
@group(2) @binding(0) var<uniform> globals: GaussianBlur; 

@compute
@workgroup_size(8, 8, 1)
fn apply_blur(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    // TODO hor ver
    let pos = vec2<i32>(global_invocation_id.xy);
    let size = vec2<i32>(textureDimensions(read_fx[fx_meta.out_idx]));

    if any(size < pos) {
        return;
    }

    let edge = globals.radius;
    let two_ss = 2. * globals.sigma * globals.sigma;
    var result = vec3<f32>(0.);

    for (var x = -edge; x <= edge; x++) {
        for (var y =  -edge; y <= edge; y++) {
            var offset = vec2<i32>(x, y);
            var nb_pos = pos + offset;

            if all(0 < nb_pos) && all(nb_pos < size) {
                var xf = f32(x);
                var yf = f32(y);

                var rhs = exp(-(xf * xf + yf * yf) / two_ss);
                var lhs = 1. / (two_ss * pi());

                var coeff = lhs * rhs * globals.intensity;
                var nb_col = textureLoad(read_fx[fx_meta.out_idx], nb_pos, 0).rgb;

                result += nb_col * coeff;
            }
        }
    }

    textureStore(write_fx[fx_meta.out_idx], pos, vec4<f32>(result, 1.0));
}

@compute
@workgroup_size(8, 8, 1)
fn split_bloom(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let pos = global_invocation_id.xy;

    let frame_size;
    let in_tex;

    if fx_meta.in_idx == -1 {
        in_tex = fx_blend;
        frame_size = vec2<u32>(textureDimensions(fx_blend));
    } else {
        in_tex = read_fx[fx_meta.in_idx];
        frame_size = vec2<u32>(textureDimensions(in_tex));
    }

    let fx_size_f32 = ceil(vec2<f32>(frame_size) / f32(fx_meta.out_downscale));
    let fx_size = vec2<u32>(fx_size_f32);

    if any(fx_size < pos) {
        return;
    }

    let start_x = pos.x * fx_meta.out_downscale;
    let end_x = start_x + fx_meta.out_downscale;
    let start_y = pos.y * fx_meta.out_downscale;
    let end_y = start_y + fx_meta.out_downscale;

    var weight = 0u;
    var result = vec3<f32>(0.0);

    for (var x = start_x; x < end_x; x++) {
        for (var y = start_y; y < end_y; y++) {
            if x < frame_size.x && y < frame_size.y {
                result += textureLoad(in_tex, vec2<u32>(x, y), 0).rgb;
                weight++;
            }
        }
    }

    // Averaging out
    result /= f32(weight); 

    if any(globals.br_treshold < result) {
        // Convert to HDR
        textureStore(write_fx[fx_meta.out_idx], pos, vec4<f32>(result * globals.hdr_mul, 1.0));
    } else {
        textureStore(write_fx[fx_meta.out_idx], pos, vec4<f32>(0.0));
    }
}

