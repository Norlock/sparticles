use super::emitter::SpawnData;
use super::particle::Instance;
use super::particle::Particle;
use crate::instance::particle::FIELD_COUNT;
use crate::{clock::Clock, instance::emitter::Emitter};
use wgpu::util::DeviceExt;

// number of single-particle calculations (invocations) in each gpu work group
const PARTICLES_PER_GROUP: u32 = 64;

pub struct Compute {
    pub num_particles: u32,
    pub particle_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub work_group_count: u32,
    pub pipeline: wgpu::ComputePipeline,
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

        // buffer for simulation parameters uniform

        //let sim_param_data = [
        //0.04f32, // deltaT
        //0.1,     // rule1Distance
        //0.025,   // rule2Distance
        //0.025,   // rule3Distance
        //0.02,    // rule1Scale
        //0.05,    // rule2Scale
        //0.005,   // rule3Scale
        //]
        //.to_vec();

        //let sim_param_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //label: Some("Simulation Parameter Buffer"),
        //contents: bytemuck::cast_slice(&sim_param_data),
        //usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        //});

        // create compute bind layout group and compute pipeline layout

        let instance_size = std::mem::size_of::<Instance>() as u64;
        let buffer_size = num_particles as u64 * instance_size;

        let compute_bind_group_layout =
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

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("compute"),
                bind_group_layouts: &[&compute_bind_group_layout],
                push_constant_ranges: &[],
            });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &compute_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: particle_buffer.as_entire_binding(),
            }],
            label: None,
        });

        // calculates number of work groups from PARTICLES_PER_GROUP constant
        let work_group_count =
            ((particles.len() as f32) / (PARTICLES_PER_GROUP as f32)).ceil() as u32;

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("compute pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: "main",
        });

        Self {
            work_group_count,
            particle_buffer,
            bind_group,
            pipeline,
            num_particles: num_particles as u32,
            frame: 0,
            clock: Clock::new(),
            emitters: Vec::new(),
        }
    }

    pub fn update(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
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

        let new_particles_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Particle Buffer"),
            contents: bytemuck::cast_slice(&instances),
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC,
        });

        self.num_particles += instances.len() as u32 / FIELD_COUNT as u32;
    }
}
