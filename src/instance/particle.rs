const NUM_INSTANCES_PER_ROW: u32 = 10;
const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
    0.0,
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
);

pub struct Particle {
    pub position: cgmath::Vector3<f32>,
    pub color: cgmath::Vector4<f32>,
    pub velocity: cgmath::Vector3<f32>,
    pub size: f32,
}

impl Particle {
    pub fn to_instance(&self) -> Instance {
        Instance {
            position: [self.position.x, self.position.y, self.position.z, self.size],
            color: self.color.into(),
            velocity: self.velocity.into(),
        }
    }

    // TODO replace instance with particles
    pub fn generate_particles() -> Vec<Particle> {
        (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let position = cgmath::Vector3 {
                        x: x as f32 * 4.0,
                        y: 0.0,
                        z: z as f32 * 4.0,
                    } - INSTANCE_DISPLACEMENT;

                    // this is needed so an object at (0, 0, 0) won't get scaled to zero
                    // as Quaternions can effect scale if they're not created correctly

                    Particle {
                        position,
                        color: cgmath::Vector4::new(0.0, 0.2, 1.0, 1.0),
                        size: 0.5,
                        velocity: cgmath::Vector3::new(10., 10., 0.),
                    }
                })
            })
            .collect::<Vec<_>>()
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance {
    position: [f32; 4],
    color: [f32; 4],
    velocity: [f32; 3],
}

impl Instance {
    pub fn descriptor<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Instance>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5 not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We'll have to reassemble the mat4 in
                // the shader.
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}
