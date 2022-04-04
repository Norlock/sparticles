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

    let camera_right = 
        normalize(vec3<f32>(camera.view_proj.x.x, camera.view_proj.y.x, camera.view_proj.z.x));
    let camera_up = 
        normalize(vec3<f32>(camera.view_proj.x.y, camera.view_proj.y.y, camera.view_proj.z.y));

    let theta = vertices[model.vertex_idx].w;
    let sin_cos = vec2<f32>(cos(theta), sin(theta));

    let rotation = mat2x2<f32>(
      vec2<f32>(sin_cos.x, -sin_cos.y),
      vec2<f32>(sin_cos.y, sin_cos.x),
    );

    let vertex_position_raw = vertices[model.vertex_idx].xyz;
    let vertex_position_2d = rotation * vertex_position_raw.xy;
    var vertex_rotated: vec3<f32> = 
      (camera_right * vertex_position_2d.x) + 
      (camera_up * vertex_position_2d.y);

    out.color = instance.color;
    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(vertex_rotated, 1.0);
    out.position = vertex_position_raw;
    return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let distance = in.position.x * in.position.x + in.position.y * in.position.y;
    if (1.0 <= distance) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    } else {
        return vec4<f32>(in.color.xyz * (1.0 - distance), in.color.w);
    }
}
