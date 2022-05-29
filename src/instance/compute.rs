use super::emitter::SpawnData;
use super::particle::Particle;
use crate::{clock::Clock, instance::emitter::Emitter};
use wgpu::{util::DeviceExt, CommandEncoder};

// number of single-particle calculations (invocations) in each gpu work group
const PARTICLES_PER_GROUP: u32 = 64;

pub struct Compute {
    pub num_particles: u32,
    pub particle_buffer: wgpu::Buffer,
    pub particle_bind_group: wgpu::BindGroup,
    pub particle_bind_group_layout: wgpu::BindGroupLayout,
    pub emitter_bind_group_layout: Option<wgpu::BindGroupLayout>,
    pub work_group_count: u32,
    pub compute_shader: wgpu::ShaderModule,
    pub frame: usize,
    pub clock: Clock,
    pub emitters: Vec<Emitter>,
}

impl Compute {
    pub fn new(device: &wgpu::Device) -> Self {
        let particles = Particle::generate_particles();

        let compute_shader = device.create_shader_module(&wgpu::include_wgsl!("./compute.wgsl"));

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

        // calculates number of work groups from PARTICLES_PER_GROUP constant
        let work_group_count =
            ((particles.len() as f32) / (PARTICLES_PER_GROUP as f32)).ceil() as u32;

        Self {
            work_group_count,
            particle_buffer,
            particle_bind_group,
            particle_bind_group_layout,
            compute_shader,
            num_particles: num_particles as u32,
            frame: 0,
            emitter_bind_group_layout: None,
            clock: Clock::new(),
            emitters: Vec::new(),
        }
    }

    pub fn update(&mut self, device: &wgpu::Device, encoder: &mut CommandEncoder) {
        // Try create buffer for emitted particles and copy to buffer.
        //if 1 <= self.frame {
        //return;
        //}
        let mut instances = Vec::new();

        let mut data = SpawnData {
            clock: &self.clock,
            instances: &mut instances,
        };

        for emitter in self.emitters.iter_mut() {
            emitter.spawn(&mut data);
        }

        // buffer for simulation parameters uniform

        let delta: u8 = if instances.is_empty() { 0 } else { 1 };

        let metadata = [delta].to_vec();

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

        if !instances.is_empty() {
            let emitted_particles = instances.len() as u32 / Particle::size_of() as u32;

            let emitter_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Emitter Buffer"),
                contents: bytemuck::cast_slice(&instances),
                usage: wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_SRC,
            });

            let buffer_size = emitted_particles as u64 * Particle::size_of();

            let emitter_bind_group_layout =
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

            let emitter_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &emitter_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: emitter_buffer.as_entire_binding(),
                }],
                label: None,
            });

            let compute_pipeline_layout =
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("compute"),
                    bind_group_layouts: &[
                        &metadata_bind_group_layout,
                        &self.particle_bind_group_layout,
                        &emitter_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });

            let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("compute pipeline"),
                layout: Some(&compute_pipeline_layout),
                module: &self.compute_shader,
                entry_point: "main",
            });

            let mut compute_pass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());

            compute_pass.set_pipeline(&pipeline);
            compute_pass.set_bind_group(0, &metadata_bind_group, &[]);
            compute_pass.set_bind_group(1, &self.particle_bind_group, &[]);
            compute_pass.set_bind_group(2, &emitter_bind_group, &[]);
            compute_pass.dispatch(self.work_group_count, 1, 1);
        } else {
            let compute_pipeline_layout =
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("compute"),
                    bind_group_layouts: &[
                        &metadata_bind_group_layout,
                        &self.particle_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });

            let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("compute pipeline"),
                layout: Some(&compute_pipeline_layout),
                module: &self.compute_shader,
                entry_point: "main",
            });

            let mut compute_pass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());

            compute_pass.set_pipeline(&pipeline);
            compute_pass.set_bind_group(0, &metadata_bind_group, &[]);
            compute_pass.set_bind_group(1, &self.particle_bind_group, &[]);
            compute_pass.dispatch(self.work_group_count, 1, 1);
        }
    }
}
