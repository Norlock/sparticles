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
        let particles = Particle::generate_particles();

        let mut instances = Particle::create_instance_vec(particles.len());

        for particle in particles.iter() {
            particle.map_instance(&mut instances);
        }

        let particle_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Particle Buffer")),
            contents: bytemuck::cast_slice(&instances),
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
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
        for particle in self.particles.iter_mut() {
            particle.position.x += 0.01;
            particle.position.y += 0.01;
        }

        let mut data = SpawnData {
            clock: &self.clock,
            particles: &mut self.particles,
            num_spawned_particles: 0,
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
    }
}
