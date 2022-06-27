use super::emitter::SpawnData;
use super::particle::{Particle, ParticleFunctions, FIELD_COUNT};
use crate::{clock::Clock, instance::emitter::Emitter};
use wgpu::{util::DeviceExt, CommandEncoder};

pub struct Compute {
    pub num_particles: u32,
    pub particle_buffer: wgpu::Buffer,
    pub particle_bind_group: wgpu::BindGroup,
    pub particle_bind_group_layout: wgpu::BindGroupLayout,
    pub frame: usize,
    pub clock: Clock,
    pub emitters: Vec<Emitter>,
    pub instances: Vec<f32>,
}

impl Compute {
    pub fn new(device: &wgpu::Device) -> Self {
        let particles = Particle::generate_particles();

        let num_particles = particles.len();
        let mut instances = Particle::create_instance_vec(num_particles);

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

        // create compute bind layout group and compute pipeline layout

        let buffer_size = num_particles as u64 * Particle::size_of();

        let particle_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(buffer_size),
                    },
                    count: None,
                }],
                label: None,
            });

        let particle_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &particle_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: particle_buffer.as_entire_binding(),
            }],
            label: None,
        });

        Self {
            particle_buffer,
            particle_bind_group,
            particle_bind_group_layout,
            instances,
            num_particles: num_particles as u32,
            frame: 0,
            clock: Clock::new(),
            emitters: Vec::new(),
        }
    }

    pub fn update(&mut self, device: &wgpu::Device) {
        for i in 0..self.num_particles {
            let start_index = i as usize * FIELD_COUNT;

            self.instances.move_particle(start_index, 0.01, 0.01, 0.);
        }

        {
            let mut data = SpawnData {
                clock: &self.clock,
                instances: &mut self.instances,
                num_spawned_particles: 0,
            };

            for emitter in self.emitters.iter_mut() {
                emitter.spawn(&mut data);
            }

            if 0 < data.num_spawned_particles {
                self.num_particles += data.num_spawned_particles;

                //println!("particle count: {}", self.num_particles);
            }
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

        self.particle_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Particle Buffer")),
            contents: bytemuck::cast_slice(&self.instances),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
    }
}
