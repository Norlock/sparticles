// Vertex shader
struct Camera {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: Camera;

struct VertexInput {
    @builtin(vertex_index) vertex_idx: u32,
    @location(2) position: vec4<f32>,
    @location(3) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) position: vec2<f32>,
};

@vertex
fn vs_main(
        model: VertexInput,
) -> VertexOutput {

    var vertices = mat4x2<f32>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(1.0, 1.0),
    );

    var out: VertexOutput;

    let camera_right = 
        normalize(vec3<f32>(camera.view_proj.x.x, camera.view_proj.y.x, camera.view_proj.z.x));
    let camera_up = 
        normalize(vec3<f32>(camera.view_proj.x.y, camera.view_proj.y.y, camera.view_proj.z.y));

    let theta = 1.0;
    let sin_cos = vec2<f32>(cos(theta), sin(theta));

    let rotation = mat2x2<f32>(
      vec2<f32>(sin_cos.x, -sin_cos.y),
      vec2<f32>(sin_cos.y, sin_cos.x),
    );

    let vertex_position = rotation * vertices[model.vertex_idx];
    let size = model.position.w;
    let particle_position = model.position.xyz;

    var world_space: vec3<f32> = 
        particle_position + 
        camera_right * vertex_position.x * size + camera_up * vertex_position.y * size;

    out.color = model.color;
    out.clip_position = camera.view_proj * vec4<f32>(world_space, 1.0);
    out.position = vertex_position;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let len = length(in.position);

    if (1.0 < len) {
      return vec4<f32>(0.0,0.0,0.0,0.0);
    }

    if (in.color.w < 1.0) {
        return in.color;
    }

    var dist = 1.0 / len * 0.5;
    var dist = pow(dist, 1.5);

    let col = dist * in.color.xyz;
    return vec4<f32>(1.0 - exp(-col), 1.0);
}

@fragment
fn fs_bubbles(in: VertexOutput) -> @location(0) vec4<f32> {
    let len = length(in.position);

    if (1.0 < len) {
      return vec4<f32>(0.0,0.0,0.0,0.0);
    }

    var dist = pow(len, 3.0);

    var col = dist * in.color;
    return 1.0 - exp(-col);
}
