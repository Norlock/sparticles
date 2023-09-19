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

@group(0) @binding(1) var output: texture_2d<f32>;
@group(1) @binding(1) var<uniform> globals: Offset;

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let fx_size = vec2<f32>(textureDimensions(output));

    if fx_size.x < globals.view_width || fx_size.y < globals.view_height {
        let scale_x = globals.view_width / fx_size.x;
        let scale_y = globals.view_height / fx_size.y;

        let x = i32(pos.x / scale_x);
        let y = i32(pos.y / scale_y);

        return textureLoad(output, vec2<i32>(x, y), 0);
    } else {
        return textureLoad(output, vec2<i32>(pos.xy) + globals.offset, 0);
    }
}
