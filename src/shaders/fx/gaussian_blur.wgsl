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

    let start_x = pos.x * globals.downscale;
    let end_x = start_x + globals.downscale;
    let start_y = pos.y * globals.downscale;
    let end_y = start_y + globals.downscale;

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

struct Neighbour {
    pos: vec2<u32>,
    fx_pos: vec2<u32>,
    dist: u32,
    pct: f32,
}

fn get_offset(is_horizontal: bool) -> vec2<f32> {
    if is_horizontal {
        return vec2<f32>(1., 0.);
    } else {
        return vec2<f32>(0., 1.);
    }
}

// pos = 19 
// scale = 8 
// fx_pos = 19 / 8 == 2 
// local_pos = 19 % 8 = 3 
// center = 8 / 2 = 4 
// d_center = 4 - 3
// pct = 1 / 8
fn get_neighbour_x(pos: vec2<f32>, color: vec3<f32>, scale: f32, max_x: f32) -> vec3<f32> {
    let fx_pos = pos / scale;
    let local_pos = pos.x % scale;
    let center = scale / 2.;

    if local_pos == center {
        return color;
    } 

    let d_center = abs(center - local_pos);
    let pct = d_center / scale;

    if center < local_pos {
        // right neighbour
        let nb_pos = fx_pos + get_offset(false);
        if nb_pos.x < max_x {
            let nb_col = textureLoad(src_texture, vec2<u32>(nb_pos), 0).rgb;
            return mix(color, nb_col, pct);
        }
    } else {
        // left neighbour
        let nb_pos = fx_pos - get_offset(false);
        if 0. <= nb_pos.x {
            let nb_col = textureLoad(src_texture, vec2<u32>(nb_pos), 0).rgb;
            return mix(color, nb_col, pct);
        }

    }
    return color;
}

fn get_neighbour_y(pos: vec2<f32>, color: vec3<f32>, scale: f32, max_y: f32) -> vec3<f32> {
    let fx_pos = pos / scale;
    let local_pos = pos.y % scale;
    let center = scale / 2.;

    if local_pos == center {
        return color;
    } 
 
    let d_center = abs(center - local_pos);
    let pct = d_center / scale;

    if center < local_pos {
        // down neighbour
        let nb_pos = fx_pos + get_offset(false);
        if nb_pos.y < max_y {
            let nb_col = textureLoad(src_texture, vec2<u32>(nb_pos), 0).rgb;
            return mix(color, nb_col, pct);
        }
    } else {
        // up neighbour
        let nb_pos = fx_pos - get_offset(false);
        if 0. <= nb_pos.y {
            let nb_col = textureLoad(src_texture, vec2<u32>(nb_pos), 0).rgb;
            return mix(color, nb_col, pct);
        }
    }

    return color;
}

@compute
@workgroup_size(8, 8, 1)
fn upscale(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let fx_size = textureDimensions(fx_blend);
    let pos = global_invocation_id.xy;

    if any(fx_size < pos) {
        return;
    }

    let scale = globals.downscale;

    var result = textureLoad(src_texture, vec2<u32>(pos / scale), 0).rgb;

    result = get_neighbour_x(pos, result, scale, fx_size.x);
    result = get_neighbour_y(pos, result, scale, fx_size.y);

    textureStore(dst_texture, vec2<u32>(pos), vec4<f32>(result, 1.0));
}
