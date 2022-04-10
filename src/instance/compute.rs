use crate::instance::particle::Particle;
use wgpu::util::DeviceExt;

use crate::instance::particle::Instance;

// number of single-particle calculations (invocations) in each gpu work group
const PARTICLES_PER_GROUP: u32 = 64;

pub struct ComputeData {
    pub num_particles: u32,
    pub particle_buffers: Vec<wgpu::Buffer>,
    pub bind_groups: Vec<wgpu::BindGroup>,
    pub work_group_count: u32,
    pub pipeline: wgpu::ComputePipeline,
    pub frame: usize,
}

impl ComputeData {
    pub fn new(device: &wgpu::Device) -> Self {
        let particles = Particle::generate_particles();
        let mut particle_buffers = Vec::<wgpu::Buffer>::new();
        let mut particle_bind_groups = Vec::<wgpu::BindGroup>::new();

        let compute_shader = device.create_shader_module(&wgpu::include_wgsl!("./compute.wgsl"));

        let num_particles = particles.len();
        let field_count = 11;
        let mut instances = Vec::with_capacity(num_particles * field_count);

        for particle in particles.iter() {
            particle.to_instance(&mut instances);
        }

        for i in 0..2 {
            particle_buffers.push(
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("Particle Buffer {}", i)),
                    contents: bytemuck::cast_slice(&instances),
                    usage: wgpu::BufferUsages::VERTEX
                        | wgpu::BufferUsages::STORAGE
                        | wgpu::BufferUsages::COPY_DST,
                }),
            );
        }
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
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(buffer_size),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(buffer_size),
                        },
                        count: None,
                    },
                ],
                label: None,
            });

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("compute"),
                bind_group_layouts: &[&compute_bind_group_layout],
                push_constant_ranges: &[],
            });

        // create two bind groups, one for each buffer as the src
        // where the alternate buffer is used as the dst
        for i in 0..2 {
            particle_bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &compute_bind_group_layout,
                entries: &[
                    //wgpu::BindGroupEntry {
                    //binding: 0,
                    //resource: sim_param_buffer.as_entire_binding(),
                    //},
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: particle_buffers[i].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: particle_buffers[(i + 1) % 2].as_entire_binding(), // bind to opposite buffer
                    },
                ],
                label: None,
            }));
        }

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
            particle_buffers,
            bind_groups: particle_bind_groups,
            pipeline,
            num_particles: num_particles as u32,
            frame: 0,
        }
    }
}
