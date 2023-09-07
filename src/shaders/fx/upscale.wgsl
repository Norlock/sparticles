struct Neighbour {
    pos: vec2<u32>,
    fx_pos: vec2<u32>,
    dist: u32,
    pct: f32,
}

@group(0) @binding(1) var src_texture: texture_2d<f32>;
@group(1) @binding(0) var dst_texture: texture_storage_2d<rgba8unorm, write>;

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
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let upscaled_size = vec2<f32>(textureDimensions(dst_texture));
    let fx_size = vec2<f32>(textureDimensions(src_texture));
    let pos = vec2<f32>(global_invocation_id.xy);

    if any(upscaled_size < pos) {
        return;
    }

    let scale = upscaled_size.x / fx_size.x;

    var result = textureLoad(src_texture, vec2<u32>(pos / scale), 0).rgb;

    result = get_neighbour_x(pos, result, scale, fx_size.x);
    result = get_neighbour_y(pos, result, scale, fx_size.y);

    textureStore(dst_texture, vec2<u32>(pos), vec4<f32>(result, 1.0));
}
