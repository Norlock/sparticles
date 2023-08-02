use egui_wgpu_backend::wgpu;
use glam::Vec4;
use wgpu::util::DeviceExt;

use crate::{
    model::{Clock, ComputeState, GfxState},
    traits::*,
};

pub struct ColorAnimation {
    pub from_color: Vec4,
    pub to_color: Vec4,
    pub from_sec: f32,
    pub until_sec: f32,
}

impl ColorAnimation {
    fn create_buffer_content(&self) -> [f32; 10] {
        [
            self.from_color.x,
            self.from_color.y,
            self.from_color.z,
            self.from_color.w,
            self.to_color.x,
            self.to_color.y,
            self.to_color.z,
            self.to_color.w,
            self.from_sec,
            self.until_sec,
        ]
    }
}

impl CreateAnimation for ColorAnimation {
    fn create_animation(
        self: Box<Self>,
        gfx_state: &GfxState,
        compute: &ComputeState,
    ) -> Box<dyn Animation> {
        Box::new(ColorAnimationPipeline::new(
            *self,
            compute,
            &gfx_state.device,
        ))
    }
}

struct ColorAnimationPipeline {
    pipeline: wgpu::ComputePipeline,
    animation_bind_group: wgpu::BindGroup,
}

impl Animation for ColorAnimationPipeline {
    fn update(&mut self, _clock: &Clock, _gfx_state: &GfxState) {}

    fn compute<'a>(
        &'a self,
        clock: &Clock,
        compute: &'a ComputeState,
        compute_pass: &mut wgpu::ComputePass<'a>,
    ) {
        compute_pass.set_pipeline(&self.pipeline);
        compute_pass.set_bind_group(0, &compute.bind_groups[clock.get_bindgroup_nr()], &[]);
        compute_pass.set_bind_group(1, &self.animation_bind_group, &[]);
        compute_pass.dispatch_workgroups(compute.dispatch_x_count, 1, 1);
    }
}

impl ColorAnimationPipeline {
    fn new(animation: ColorAnimation, compute: &ComputeState, device: &wgpu::Device) -> Self {
        let anim_shader_str = include_str!("../shaders/color_anim.wgsl");
        let shader = device.create_shader(anim_shader_str, "Color animation");

        let animation_uniform = animation.create_buffer_content();

        let animation_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Color buffer"),
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
                        min_binding_size: animation_uniform.cal_buffer_size(),
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
            label: Some("Color animation"),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute layout"),
            bind_group_layouts: &[&compute.bind_group_layout, &animation_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Color animation pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "main",
        });

        Self {
            pipeline,
            animation_bind_group,
        }
    }
}
