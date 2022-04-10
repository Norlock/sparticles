struct Particle {
  pos_x: f32;
  pos_y: f32;
  pos_z: f32;
  size: f32;
  color_r: f32;
  color_g: f32;
  color_b: f32;
  color_a: f32;
  velocity_x: f32;
  velocity_y: f32;
  velocity_z: f32;
};

//struct SimParams {
//  deltaT : f32;
//  rule1Distance : f32;
//  rule2Distance : f32;
//  rule3Distance : f32;
//  rule1Scale : f32;
//  rule2Scale : f32;
//  rule3Scale : f32;
//};

struct Particles {
  data : [[stride(44)]] array<Particle>;
};

//[[group(0), binding(0)]] var<uniform> params : SimParams;
[[group(0), binding(0)]] var<storage, read> particlesSrc: Particles;
[[group(0), binding(1)]] var<storage, read_write> particlesDst : Particles;

// https://github.com/austinEng/Project6-Vulkan-Flocking/blob/master/data/shaders/computeparticles/particle.comp
[[stage(compute), workgroup_size(64)]]
fn main([[builtin(global_invocation_id)]] global_id: vec3<u32>) {
  let total = arrayLength(&particlesSrc.data);
  let index = global_id.x;
  if (index >= total) {
    return;
  }

  //var position_size : vec4<f32> = particlesSrc.data[index].position_size;

  var particle = particlesSrc.data[index];

  var vel: vec3<f32> = vec3<f32>(0.01, 0.0, 0.0);

  // Write back
  //particles.particles[index].position_size = vec4<f32>(position, size);
  particlesDst.data[index].pos_x = particle.pos_x + 0.005;
  //particlesDst.particles[index].vel = vVel;
}
