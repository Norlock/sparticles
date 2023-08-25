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

@group(0) @binding(0)
var frame_texture: texture_2d<f32>;
@group(0) @binding(1)
var fx_texture: texture_2d<f32>;

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let pos = vec2<i32>(position.xy);
    let frame_texture = textureLoad(frame_texture, pos, 0);
    let fx_texture = textureLoad(fx_texture, pos, 0);

    return vec4<f32>(frame_texture.rgb * fx_texture.rgb, 1.0);
}
