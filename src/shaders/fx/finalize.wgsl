@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    // Draws one big rectangle on screen
    // https://stackoverflow.com/questions/2588875/whats-the-best-way-to-draw-a-fullscreen-quad-in-opengl-3-2/59739538#59739538
    if vertex_index == 0u {
        return vec4<f32>(-1.0, -1.0, 0.0, 1.0);
    } else if vertex_index == 1u {
        return vec4<f32>(3.0, -1.0, 0.0, 1.0);
    } else {
        return vec4<f32>(-1.0, 3.0, 0.0, 1.0);
    }
}

struct Offset {
    offset: i32,
    view_width: f32,
    view_height: f32,
}

@group(0) @binding(2) var fx_blend: texture_storage_2d<rgba8unorm, read_write>;

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    var out = textureLoad(fx_blend, vec2<i32>(pos.xy), 0);
    return out;
}
