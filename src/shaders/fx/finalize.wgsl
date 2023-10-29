struct Viewport {
    dimensions: vec2<f32>,
}

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

@group(0) @binding(1) var read_fx: binding_array<texture_2d<f32>, 32>;
//@group(1) @binding(1) var s: sampler;
@group(1) @binding(0) var<uniform> fx_io: FxIO; 
@group(2) @binding(0) var<uniform> view: Viewport; 

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let tex_size = vec2<f32>(textureDimensions(read_fx[0])) / view.dimensions;
    let tex_pos = pos.xy * tex_size;

    var out = textureLoad(read_fx[fx_io.out_idx], vec2<i32>(tex_pos), 0).rgb;
    return vec4<f32>(out, 1.0);
}
