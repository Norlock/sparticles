use super::particle::Particle;
use crate::{clock::Clock, instance::emitter::Emitter};
use wgpu::util::DeviceExt;

pub struct Instance {
    pub buffer: wgpu::Buffer,
    pub emitters: Vec<Emitter>,
    pub num_particles: u32,
}

impl Instance {
    pub fn new(device: &wgpu::Device) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("Particle Buffer")),
            size: 0,
            mapped_at_creation: false,
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            buffer,
            emitters: Vec::new(),
            num_particles: 0,
        }
    }

    pub fn update(&mut self, device: &wgpu::Device, clock: &Clock) {
        for emitter in self.emitters.iter_mut() {
            emitter.spawn(&clock);
        }

        let num_particles = self.emitters.iter().map(|x| x.particles.len()).sum();
        let mut instances = Particle::create_instance_vec(num_particles);

        for emitter in self.emitters.iter_mut() {
            emitter.handle_particles(&mut instances, &clock);
            emitter.animate_emitter(&clock);
        }

        self.num_particles = num_particles as u32;

        self.buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Particle Buffer")),
            contents: bytemuck::cast_slice(&instances),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
    }
}
