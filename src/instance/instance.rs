use super::emitter::SpawnData;
use super::particle::Particle;
use crate::{clock::Clock, instance::emitter::Emitter};
use wgpu::util::DeviceExt;

pub struct Instance {
    pub particle_buffer: wgpu::Buffer,
    pub frame: usize,
    pub clock: Clock,
    pub emitters: Vec<Emitter>,
    pub particles: Vec<Particle>,
}

impl Instance {
    pub fn new(device: &wgpu::Device) -> Self {
        let particles: Vec<Particle> = Vec::new();

        let particle_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("Particle Buffer")),
            size: 0,
            mapped_at_creation: true,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            particle_buffer,
            frame: 0,
            clock: Clock::new(),
            emitters: Vec::new(),
            particles,
        }
    }

    pub fn update(&mut self, device: &wgpu::Device) {
        self.clock.update();
        let elapsed_ms = self.clock.lifetime_ms();

        self.particles
            .retain(|particle| elapsed_ms - particle.spawned_at < particle.lifetime_ms);

        for particle in self.particles.iter_mut() {
            particle.update(self.clock.delta_sec());
        }

        let mut data = SpawnData {
            elapsed_ms,
            particles: &mut self.particles,
        };

        for emitter in self.emitters.iter_mut() {
            emitter.spawn(&mut data);
        }

        let mut instances = Particle::create_instance_vec(self.particles.len());

        for particle in self.particles.iter() {
            Particle::map_instance(particle, &mut instances);
        }

        self.particle_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Particle Buffer")),
            contents: bytemuck::cast_slice(&instances),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        self.frame += 1;
    }
}
