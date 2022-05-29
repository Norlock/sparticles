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

struct Metadata {
   has_new_particles: i32;
};

struct Particles {
  data : [[stride(44)]] array<Particle>;
};

[[group(0), binding(0)]] var<uniform> params : Metadata;
[[group(1), binding(0)]] var<storage, read_write> particles: Particles;
[[group(2), binding(0)]] var<storage, read_write> new_particles: Particles;

[[stage(compute), workgroup_size(64)]]
fn main([[builtin(global_invocation_id)]] global_id: vec3<u32>) {
  let total = arrayLength(&particles.data);
  let index = global_id.x;
  if (index > total) {
    return;
  }

  var particle = particles.data[index];
  var a: i32 = 0;

  if (params.has_new_particles == 1) {
    particle.col_g = 1.0;
  } else {
    particle.col_g = 0.0;
  }

  particle.pos_x = particle.pos_x + particle.vel_x;
  particle.pos_y = particle.pos_y + particle.vel_y;
  particle.pos_z = particle.pos_z + particle.vel_z;

  // Write back
  particles.data[index] = particle;
}
