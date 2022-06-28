const NUM_INSTANCES_PER_ROW: u32 = 2;
const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
    0.0,
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
);

pub const FIELD_COUNT: usize = 11;

pub struct Particle {
    pub position: cgmath::Vector3<f32>,
    pub color: cgmath::Vector4<f32>,
    pub velocity: cgmath::Vector3<f32>,
    pub size: f32,
}

impl Particle {
    pub fn map_instance(&self, instances: &mut Vec<f32>) {
        instances.push(self.position.x);
        instances.push(self.position.y);
        instances.push(self.position.z);
        instances.push(self.size);
        instances.push(self.color.x);
        instances.push(self.color.y);
        instances.push(self.color.z);
        instances.push(self.color.w);
        instances.push(self.velocity.x);
        instances.push(self.velocity.y);
        instances.push(self.velocity.z);
    }

    pub fn map_to_instances(vec: &Vec<Particle>) -> Vec<f32> {
        let mut instances = Particle::create_instance_vec(vec.len());

        for particle in vec.iter() {
            Particle::map_instance(particle, &mut instances);
        }

        instances
    }

    pub fn size_of() -> wgpu::BufferAddress {
        (FIELD_COUNT * 4) as wgpu::BufferAddress
    }

    pub fn create_instance_vec(num_particles: usize) -> Vec<f32> {
        Vec::with_capacity(num_particles * FIELD_COUNT)
    }

    // TODO remove if emitter is done.
    pub fn generate_particles() -> Vec<Particle> {
        (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let position = cgmath::Vector3 {
                        x: x as f32 * 4.0,
                        y: 0.0,
                        z: z as f32 * 4.0,
                    } - INSTANCE_DISPLACEMENT;

                    Particle {
                        position,
                        color: cgmath::Vector4::new(0.0, 0.2, 1.0, 1.0),
                        size: 0.5,
                        velocity: cgmath::Vector3::new(0.001, 0.001, 0.),
                    }
                })
            })
            .collect::<Vec<_>>()
    }

    pub fn descriptor<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: Particle::size_of(),
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}
