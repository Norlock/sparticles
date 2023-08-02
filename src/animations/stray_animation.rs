use crate::{
    model::{Clock, ComputeState, GfxState},
    traits::{Animation, CreateAnimation, CustomShader},
};
use egui_wgpu_backend::wgpu;
use wgpu::util::DeviceExt;

pub struct StrayAnimation {
    pub stray_radians: f32,
    pub from_sec: f32,
    pub until_sec: f32,
}

impl StrayAnimation {
    fn create_buffer_content(&self) -> [f32; 4] {
        [self.stray_radians, self.from_sec, self.until_sec, 0.]
    }
}

impl CreateAnimation for StrayAnimation {
    fn create_animation(
        self: Box<Self>,
        gfx_state: &GfxState,
        compute: &ComputeState,
    ) -> Box<dyn Animation> {
        Box::new(StrayAnimationState::new(*self, compute, &gfx_state.device))
    }
}

struct StrayAnimationState {
    pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
    bind_group_nr: usize,
}

impl Animation for StrayAnimationState {
    fn update(&mut self, clock: &Clock, _gfx_state: &GfxState) {
        self.bind_group_nr = clock.get_bindgroup_nr();
    }

    fn compute<'a>(&'a self, compute: &'a ComputeState, compute_pass: &mut wgpu::ComputePass<'a>) {
        compute_pass.set_pipeline(&self.pipeline);
        compute_pass.set_bind_group(0, &compute.bind_groups[self.bind_group_nr], &[]);
        compute_pass.set_bind_group(1, &self.bind_group, &[]);
        compute_pass.dispatch_workgroups(compute.dispatch_x_count, 1, 1);
    }
}

impl StrayAnimationState {
    fn new(animation: StrayAnimation, compute: &ComputeState, device: &wgpu::Device) -> Self {
        let anim_shader_str = include_str!("../shaders/stray_anim.wgsl");
        let shader = device.create_shader(anim_shader_str, "Stray animation");

        let animation_uniform = animation.create_buffer_content();

        let animation_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Stray buffer"),
            contents: bytemuck::cast_slice(&animation_uniform),
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
                        min_binding_size: wgpu::BufferSize::new(animation_uniform.len() as u64 * 4),
                    },
                    count: None,
                },
            ],
            label: None,
        });

        let animation_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &animation_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: animation_buffer.as_entire_binding(),
            }],
            label: Some("Stray animation"),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Stray layout"),
            bind_group_layouts: &[&compute.bind_group_layout, &animation_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Stray animation pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "main",
        });

        Self {
            pipeline,
            bind_group: animation_bind_group,
            bind_group_nr: 1,
        }
    }
}
