// Vertex shader
struct InstanceInput {
    [[location(5)]] model_matrix_0: vec4<f32>;
    [[location(6)]] model_matrix_1: vec4<f32>;
    [[location(7)]] model_matrix_2: vec4<f32>;
    [[location(8)]] model_matrix_3: vec4<f32>;
    [[location(9)]] color: vec4<f32>;
};

struct CameraUniform {
    view_proj: mat4x4<f32>;
};
[[group(0), binding(0)]] 
var<uniform> camera: CameraUniform;

struct VertexInput {
    [[builtin(vertex_index)]] vertex_idx: u32;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] color: vec4<f32>;
    [[location(1)]] position: vec3<f32>;
};

[[stage(vertex)]]
fn vs_main(
        model: VertexInput,
        instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
      instance.model_matrix_0,
      instance.model_matrix_1,
      instance.model_matrix_2,
      instance.model_matrix_3,
    );

    var vertices = mat4x4<f32>(
        vec4<f32>(-1.0, -1.0, 0.0, 1.0),
        vec4<f32>(-1.0, 1.0, 0.0, 1.0),
        vec4<f32>(1.0, -1.0, 0.0, 1.0),
        vec4<f32>(1.0, 1.0, 0.0, 1.0),
    );

    var out: VertexOutput;

    let position = vertices[model.vertex_idx];
    out.color = instance.color;
    out.clip_position = camera.view_proj * model_matrix * position;
    out.position = position.xyz;
    return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let distance = in.position.x * in.position.x + in.position.y * in.position.y;
    if (0.95 <= distance) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    } else {
        return vec4<f32>(in.color.xyz * (1.0 - distance), in.color.w);
    }
}
