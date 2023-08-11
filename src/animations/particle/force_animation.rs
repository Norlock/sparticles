use egui_wgpu::wgpu::{self, util::DeviceExt, Device};
use glam::Vec3;

use crate::{
    model::{Clock, GfxState, LifeCycle, SpawnState},
    traits::{Animation, CreateAnimation, CustomShader},
};

#[derive(Clone, Copy)]
pub struct ForceUniform {
    pub life_cycle: LifeCycle,
    pub velocity: Vec3,
    /// Applied on a 1.0 particle size unit
    pub mass_per_unit: f32,
}

impl ForceUniform {
    fn create_buffer_content(&self) -> [f32; 4] {
        [
            self.velocity.x,
            self.velocity.y,
            self.velocity.z,
            self.mass_per_unit,
        ]
    }
}

impl CreateAnimation for ForceUniform {
    fn into_animation(
        self: Box<Self>,
        gfx_state: &GfxState,
        spawner: &SpawnState,
    ) -> Box<dyn Animation> {
        Box::new(ForceAnimation::new(*self, spawner, &gfx_state.device))
    }
}

pub struct ForceAnimation {
    pipeline: wgpu::ComputePipeline,
    uniform: ForceUniform,
    bind_group: wgpu::BindGroup,
    should_animate: bool,
}

impl Animation for ForceAnimation {
    fn update(&mut self, clock: &Clock, _gfx_state: &GfxState) {
        let uniform = &self.uniform;
        let current_sec = uniform.life_cycle.get_current_sec(clock);
        self.should_animate = uniform.life_cycle.shoud_animate(current_sec);
    }

    fn compute<'a>(
        &'a self,
        spawner: &'a SpawnState,
        clock: &Clock,
        compute_pass: &mut wgpu::ComputePass<'a>,
    ) {
        if !self.should_animate {
            return;
        }

        let nr = clock.get_bindgroup_nr();

        compute_pass.set_pipeline(&self.pipeline);
        compute_pass.set_bind_group(0, &spawner.bind_groups[nr], &[]);
        compute_pass.set_bind_group(1, &self.bind_group, &[]);
        compute_pass.dispatch_workgroups(spawner.dispatch_x_count, 1, 1);
    }

    fn recreate(&self, gfx_state: &GfxState, spawner: &SpawnState) -> Box<dyn Animation> {
        Box::new(Self::new(self.uniform, spawner, &gfx_state.device))
    }
}

impl ForceAnimation {
    fn new(uniform: ForceUniform, spawner: &SpawnState, device: &Device) -> Self {
        let shader = device.create_shader("force_anim.wgsl", "Force animation");

        let buffer_content = uniform.create_buffer_content();

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Force buffer"),
            contents: bytemuck::cast_slice(&buffer_content),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let animation_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // Uniform data
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(buffer_content.len() as u64 * 4),
                    },
                    count: None,
                },
            ],
            label: None,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &animation_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("Force animation bind group"),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Force animation layout"),
            bind_group_layouts: &[&spawner.bind_group_layout, &animation_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Force animation pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "main",
        });

        Self {
            pipeline,
            uniform,
            bind_group,
            should_animate: false,
        }
    }
}
