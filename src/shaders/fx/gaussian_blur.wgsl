@group(0) @binding(0) var write_fx: binding_array<texture_storage_2d<rgba8unorm, write>, 32>;
@group(0) @binding(1) var read_fx: binding_array<texture_2d<f32>, 32>;

@group(1) @binding(0) var<uniform> globals: GaussianBlur; 
@group(1) @binding(1) var<uniform> fx_meta: FxMeta; 

fn apply_blur(pos: vec2<i32>, offset: vec2<i32>) {
    let size = vec2<i32>(textureDimensions(read_fx[fx_meta.out_idx]));

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
            var col = textureLoad(read_fx[fx_meta.out_idx], tex_pos, 0).rgb;

            result += col * coeff;
        }
    }

    textureStore(write_fx[fx_meta.out_idx], pos, vec4<f32>(result, 1.0));
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

    var frame_size = vec2<u32>(textureDimensions(read_fx[fx_meta.in_idx]));
    let fx_size_f32 = ceil(vec2<f32>(frame_size) / f32(globals.downscale));
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
                result += textureLoad(read_fx[fx_meta.in_idx], vec2<u32>(x, y), 0).rgb;
                weight++;
            }
        }
    }

    // Averaging out
    result /= f32(weight); 

    if any(vec3<f32>(globals.br_treshold) < result) {
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
    let read_tex = read_fx[fx_meta.out_idx];

    if local_pos == center {
        return color;
    } 

    let d_center = abs(center - local_pos);
    let pct = d_center / scale;

    if center < local_pos {
        // right neighbour
        let nb_pos = fx_pos + get_offset(false);
        if nb_pos.x < max_x {
            let nb_col = textureLoad(read_tex, vec2<u32>(nb_pos), 0).rgb;
            return mix(color, nb_col, pct);
        }
    } else {
        // left neighbour
        let nb_pos = fx_pos - get_offset(false);
        if 0. <= nb_pos.x {
            let nb_col = textureLoad(read_tex, vec2<u32>(nb_pos), 0).rgb;
            return mix(color, nb_col, pct);
        }

    }
    return color;
}

fn get_neighbour_y(pos: vec2<f32>, color: vec3<f32>, scale: f32, max_y: f32) -> vec3<f32> {
    let fx_pos = pos / scale;
    let local_pos = pos.y % scale;
    let center = scale / 2.;
    let read_tex = read_fx[fx_meta.out_idx];

    if local_pos == center {
        return color;
    } 
 
    let d_center = abs(center - local_pos);
    let pct = d_center / scale;

    if center < local_pos {
        // down neighbour
        let nb_pos = fx_pos + get_offset(false);
        if nb_pos.y < max_y {
            let nb_col = textureLoad(read_tex, vec2<u32>(nb_pos), 0).rgb;
            return mix(color, nb_col, pct);
        }
    } else {
        // up neighbour
        let nb_pos = fx_pos - get_offset(false);
        if 0. <= nb_pos.y {
            let nb_col = textureLoad(read_tex, vec2<u32>(nb_pos), 0).rgb;
            return mix(color, nb_col, pct);
        }
    }

    return color;
}

@compute
@workgroup_size(8, 8, 1)
fn upscale(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let fx_size = vec2<f32>(textureDimensions(read_fx[0]));
    let pos = vec2<f32>(global_invocation_id.xy);

    if any(fx_size < pos) {
        return;
    }

    let scale = f32(globals.downscale);

    let read_tex = read_fx[fx_meta.out_idx];
    var result = textureLoad(read_tex, vec2<u32>(pos / scale), 0).rgb;

    result = get_neighbour_x(pos, result, scale, fx_size.x);
    result = get_neighbour_y(pos, result, scale, fx_size.y);

    let write_tex = write_fx[fx_meta.out_idx];
    textureStore(write_tex, vec2<u32>(pos), vec4<f32>(result, 1.0));
}
