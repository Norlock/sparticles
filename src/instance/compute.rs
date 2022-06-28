use super::emitter::SpawnData;
use super::particle::{Particle, FIELD_COUNT};
use crate::{clock::Clock, instance::emitter::Emitter};
use wgpu::util::DeviceExt;

pub struct Compute {
    pub particle_buffer: wgpu::Buffer,
    pub frame: usize,
    pub clock: Clock,
    pub emitters: Vec<Emitter>,
    pub particles: Vec<Particle>,
}

impl Compute {
    pub fn new(device: &wgpu::Device) -> Self {
        let particles = Particle::generate_particles();

        let mut instances = Particle::map_to_instances(&particles);

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

        // buffer for simulation parameters uniform
        //let delta: u8 = if instances.is_empty() { 0 } else { 1 };

        let metadata = [1_u8].to_vec();

        let metadata_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Simulation Parameter Buffer"),
            contents: bytemuck::cast_slice(&metadata),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let metadata_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("Metadata bind group layout"),
            });

        let metadata_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &&metadata_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: metadata_buffer.as_entire_binding(),
            }],
            label: Some("Metadata bind group"),
        });

        let instances = Particle::map_to_instances(&self.particles);

        self.particle_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Particle Buffer")),
            contents: bytemuck::cast_slice(&instances),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
    }
}
