use crate::instance::color::Color;

pub const FIELD_COUNT: usize = 11;

pub struct Particle {
    pub position: cgmath::Vector3<f32>,
    pub color: Color,
    pub velocity: cgmath::Vector3<f32>,
    pub size: f32,
    pub spawned_at: u128,
    pub lifetime_ms: u128,
    pub friction_coefficient: f32,
    pub mass: f32,
}

impl Particle {
    pub fn update(&mut self, delta: f32) {
        let x_force = self.velocity.x * self.mass;
        let y_force = self.velocity.y * self.mass;
        let z_force = self.velocity.z * self.mass;

        self.velocity.x = x_force * self.friction_coefficient / self.mass;
        self.velocity.y = y_force * self.friction_coefficient / self.mass;
        self.velocity.z = z_force * self.friction_coefficient / self.mass;

        self.position.x += self.velocity.x * delta;
        self.position.y += self.velocity.y * delta;
        self.position.z += self.velocity.z * delta;
    }

    pub fn map_instance(&self, instances: &mut Vec<f32>) {
        instances.push(self.position.x);
        instances.push(self.position.y);
        instances.push(self.position.z);
        instances.push(self.size);
        instances.push(self.color.r);
        instances.push(self.color.g);
        instances.push(self.color.b);
        instances.push(self.color.a);
        instances.push(self.velocity.x);
        instances.push(self.velocity.y);
        instances.push(self.velocity.z);
    }

    pub fn size_of() -> wgpu::BufferAddress {
        (FIELD_COUNT * 4) as wgpu::BufferAddress
    }

    pub fn create_instance_vec(num_particles: usize) -> Vec<f32> {
        Vec::with_capacity(num_particles * FIELD_COUNT)
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
