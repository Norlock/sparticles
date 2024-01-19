@group(0) @binding(1) var terrain_map: texture_cube<f32>;
@group(0) @binding(2) var terrain_s: sampler;
@group(1) @binding(0) var<uniform> camera: Camera;

var<private> full_triangle: array<vec2<f32>, 3> = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -3.0),
    vec2<f32>(-1.0, 1.0),
    vec2<f32>(3.0, 1.0)
);

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec3<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vert_idx: u32) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(full_triangle[vert_idx], 0.999, 1.);

    let unprojected = camera.inv_proj * out.clip_position;
    out.uv = camera.inv_view * unprojected.xyz;

    return out;
}

@fragment
fn fs_draw_terrain(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(terrain_map, terrain_s, in.uv).rgb;
    color = color / (color + vec3(1.0));

    return vec4(linear_to_srgb(color), 1.);
}


@fragment
fn fs_irradiance_convolution(in: VertexOutput) -> @location(0) vec4<f32> {
	// The world vector acts as the normal of a tangent surface
    // from the origin, aligned to WorldPos. Given this normal, calculate all
    // incoming radiance of the environment. The result of this radiance
    // is the radiance of light coming from -Normal direction, which is what
    // we use in the PBR shader to sample irradiance.

    let N = vec3(in.clip_position.xy, 1.0);
    var irradiance = vec3(0.0);   
    
    // tangent space calculation from origin point
    var up = vec3(0.0, 1.0, 0.0);
    var right = normalize(cross(up, N));
    up = normalize(cross(N, right));

    //var sampleDelta = 0.025;
    var sampleDelta = 0.05;
    var nr_samples = 0.0;

    for (var phi = 0.0; phi < 2.0 * PI; phi += sampleDelta) {
        for (var theta = 0.0; theta < 0.5 * PI; theta += sampleDelta) {
            // spherical to cartesian (in tangent space)
            var tangent_sample = vec3(sin(theta) * cos(phi), sin(theta) * sin(phi), cos(theta));
            // tangent space to world
            var sample_vec = tangent_sample.x * right + tangent_sample.y * up + tangent_sample.z * N;

            irradiance += textureSample(terrain_map, terrain_s, sample_vec).rgb * cos(theta) * sin(theta);
            nr_samples += 1.;
        }
    }

    irradiance = PI * irradiance * (1.0 / nr_samples);

    return vec4(irradiance, 1.0);
}
