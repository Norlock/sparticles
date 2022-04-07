// Vertex shader
struct InstanceInput {
    [[location(5)]] position: vec4<f32>;
    [[location(6)]] color: vec4<f32>;
};

struct Camera {
    view_pos: vec4<f32>;
    view_proj: mat4x4<f32>;
};
[[group(0), binding(0)]]
var<uniform> camera: Camera;

struct VertexInput {
    [[builtin(vertex_index)]] vertex_idx: u32;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] color: vec4<f32>;
};

[[stage(vertex)]]
fn vs_main(
        model: VertexInput,
        instance: InstanceInput,
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
    let size = instance.position.w;
    let particle_position = instance.position.xyz;

    var world_space: vec3<f32> = 
        particle_position + 
        ((camera_right * vertex_position.x + camera_up * vertex_position.y) * size);

    out.color = instance.color;
    out.clip_position = camera.view_proj * vec4<f32>(world_space, 1.0);
    return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return in.color;
}
