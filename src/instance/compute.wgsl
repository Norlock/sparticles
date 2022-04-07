struct Particle {
  pos : vec2<f32>;
  vel : vec2<f32>;
};

struct SimParams {
  deltaT : f32;
  //rule1Distance : f32;
  //rule2Distance : f32;
  //rule3Distance : f32;
  //rule1Scale : f32;
  //rule2Scale : f32;
  //rule3Scale : f32;
};

struct Particles {
  particles : [[stride(16)]] array<Particle>;
};

[[group(0), binding(0)]] var<uniform> params : SimParams;
[[group(0), binding(1)]] var<storage, read> particlesSrc : Particles;
[[group(0), binding(2)]] var<storage, read_write> particlesDst : Particles;

// https://github.com/austinEng/Project6-Vulkan-Flocking/blob/master/data/shaders/computeparticles/particle.comp
[[stage(compute), workgroup_size(64)]]
fn main([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
  let total = arrayLength(&particlesSrc.particles);
  let index = global_invocation_id.x;
  if (index >= total) {
    return;
  }

  var vPos : vec3<f32> = particlesSrc.particles[index].pos;
  var vVel : vec3<f32> = particlesSrc.particles[index].vel;

  vPos = vPos + vVel;

  // Write back
  particlesDst.particles[index].pos = vPos;
  particlesDst.particles[index].vel = vVel;
}
