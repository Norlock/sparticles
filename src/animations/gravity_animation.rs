use egui_wgpu_backend::wgpu;
use glam::Vec3;
use wgpu::util::DeviceExt;

use crate::model::clock::Clock;
use crate::model::{ComputeState, GfxState, LifeCycle};
use crate::traits::*;

#[derive(Debug)]
pub struct GravityAnimation {
    gravitational_force: f32,
    dead_zone: f32,
    mass: f32,
    life_cycle: LifeCycle,
    should_animate: bool,
    start_pos: Vec3,
    end_pos: Vec3,
    current_pos: Vec3,
}

pub struct GravityAnimationOptions {
    /// In newton
    pub gravitational_force: f32,
    /// Use to exclude extreme gravitational pulls, e.g. 20.
    pub dead_zone: f32,
    pub mass: f32,
    pub life_cycle: LifeCycle,
    pub start_pos: Vec3,
    pub end_pos: Vec3,
}

impl GravityAnimation {
    pub fn new(props: GravityAnimationOptions) -> Self {
        Self {
            gravitational_force: props.gravitational_force,
            dead_zone: props.dead_zone,
            mass: props.mass,
            life_cycle: props.life_cycle,
            start_pos: props.start_pos,
            end_pos: props.end_pos,
            current_pos: props.start_pos,
            should_animate: false,
        }
    }

    fn create_buffer_content(&self) -> [f32; 6] {
        [
            self.gravitational_force,
            self.dead_zone,
            self.mass,
            self.current_pos.x,
            self.current_pos.y,
            self.current_pos.z,
        ]
    }

    fn update(&mut self, clock: &Clock) {
        let current_sec = self.life_cycle.get_current_sec(clock);

        self.should_animate = self.life_cycle.shoud_animate(current_sec);

        if self.should_animate {
            let fraction = self.life_cycle.get_fraction(current_sec);
            self.current_pos = self.start_pos.lerp(self.end_pos, fraction);
        }
    }
}

impl CreateAnimation for GravityAnimation {
    fn create_animation(
        self: Box<Self>,
        gfx_state: &GfxState,
        compute: &ComputeState,
    ) -> Box<dyn Animation> {
        Box::new(GravityAnimationState::new(
            *self,
            compute,
            &gfx_state.device,
        ))
    }
}

pub struct GravityAnimationState {
    pipeline: wgpu::ComputePipeline,
    animation: GravityAnimation,
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl Animation for GravityAnimationState {
    fn compute<'a>(
        &'a self,
        clock: &Clock,
        compute: &'a ComputeState,
        compute_pass: &mut wgpu::ComputePass<'a>,
    ) {
        if !self.animation.should_animate {
            return;
        }

        compute_pass.set_pipeline(&self.pipeline);
        compute_pass.set_bind_group(0, &compute.bind_groups[clock.get_bindgroup_nr()], &[]);
        compute_pass.set_bind_group(1, &self.bind_group, &[]);
        compute_pass.dispatch_workgroups(compute.dispatch_x_count, 1, 1);
    }

    fn update(&mut self, clock: &Clock, gfx_state: &GfxState) {
        self.animation.update(&clock);

        if self.animation.should_animate {
            let buffer_content = self.animation.create_buffer_content();

            gfx_state
                .queue
                .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&buffer_content));
        }
    }
}

impl GravityAnimationState {
    fn new(animation: GravityAnimation, compute: &ComputeState, device: &wgpu::Device) -> Self {
        let shader_str_raw = include_str!("../shaders/gravity_anim.wgsl");
        let shader = device.create_shader(shader_str_raw, "Gravity animation");

        let buffer_content = animation.create_buffer_content();

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Gravitational buffer"),
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
            label: Some("Gravity animation bind group"),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Gravity animation layout"),
            bind_group_layouts: &[&compute.bind_group_layout, &animation_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Gravity animation pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "main",
        });

        Self {
            pipeline,
            animation,
            buffer,
            bind_group,
        }
    }
}
