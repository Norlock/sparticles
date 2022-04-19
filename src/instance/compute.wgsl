struct Particle {
  pos_x: f32;
  pos_y: f32;
  pos_z: f32;
  size: f32;
  col_r: f32;
  col_g: f32;
  col_b: f32;
  col_a: f32;
  vel_x: f32;
  vel_y: f32;
  vel_z: f32;
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
[[group(0), binding(0)]] var<storage, read_write> particles: Particles;
//[[group(0), binding(1)]] var<storage, read_write> particlesDst : Particles;

// https://github.com/austinEng/Project6-Vulkan-Flocking/blob/master/data/shaders/computeparticles/particle.comp
[[stage(compute), workgroup_size(64)]]
fn main([[builtin(global_invocation_id)]] global_id: vec3<u32>) {
  let total = arrayLength(&particles.data);
  let index = global_id.x;
  if (index > total) {
    return;
  }

  var particle = particles.data[index];

  // Write back
  particles.data[index].pos_x = particle.pos_x + particle.vel_x;
  particles.data[index].pos_y = particle.pos_y + particle.vel_y;
  particles.data[index].pos_z = particle.pos_z + particle.vel_z;
}
