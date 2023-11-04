use super::{
    post_process::{FxIOUniform, FxOptions},
    FxState,
};
use crate::{traits::CustomShader, util::UniformContext};
use egui_wgpu::wgpu;
use encase::ShaderType;
use serde::{Deserialize, Serialize};

pub struct BlendPass {
    add_pipeline: wgpu::ComputePipeline,
    lerp_upscale_pipeline: wgpu::ComputePipeline,
    lerp_simple_pipeline: wgpu::ComputePipeline,
    io_ctx: UniformContext,
    io_uniform: FxIOUniform,
}

#[derive(ShaderType, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BlendUniform {
    /// Number between 0 and 1. (0) Is col from input (1) is col from output
    pub io_mix: f32,
}

pub struct BlendSettings<'a> {
    pub io_uniform: FxIOUniform,
    pub blend_layout: &'a wgpu::BindGroupLayout,
}

impl BlendPass {
    pub fn add_blend<'a>(
        &'a self,
        fx_state: &'a FxState,
        blend_bg: &'a wgpu::BindGroup,
        c_pass: &mut wgpu::ComputePass<'a>,
    ) {
        let (count_x, count_y) = fx_state.count_out(&self.io_uniform);

        c_pass.set_pipeline(&self.add_pipeline);
        c_pass.set_bind_group(0, &fx_state.bg, &[]);
        c_pass.set_bind_group(1, &self.io_ctx.bg, &[]);
        c_pass.set_bind_group(2, blend_bg, &[]);
        c_pass.dispatch_workgroups(count_x, count_y, 1);
    }

    /// Does a average based on multiple points, and mix IO
    pub fn lerp_upscale_blend<'a>(
        &'a self,
        fx_state: &'a FxState,
        blend_bg: &'a wgpu::BindGroup,
        c_pass: &mut wgpu::ComputePass<'a>,
    ) {
        let (count_x, count_y) = fx_state.count_out(&self.io_uniform);

        c_pass.set_pipeline(&self.lerp_upscale_pipeline);
        c_pass.set_bind_group(0, &fx_state.bg, &[]);
        c_pass.set_bind_group(1, &self.io_ctx.bg, &[]);
        c_pass.set_bind_group(2, blend_bg, &[]);
        c_pass.dispatch_workgroups(count_x, count_y, 1);
    }

    /// Does a mix of IO
    pub fn lerp_simple_blend<'a>(
        &'a self,
        fx_state: &'a FxState,
        blend_bg: &'a wgpu::BindGroup,
        c_pass: &mut wgpu::ComputePass<'a>,
    ) {
        let (count_x, count_y) = fx_state.count_out(&self.io_uniform);

        c_pass.set_pipeline(&self.lerp_simple_pipeline);
        c_pass.set_bind_group(0, &fx_state.bg, &[]);
        c_pass.set_bind_group(1, &self.io_ctx.bg, &[]);
        c_pass.set_bind_group(2, blend_bg, &[]);
        c_pass.dispatch_workgroups(count_x, count_y, 1);
    }

    pub fn resize(&mut self, options: &FxOptions) {
        self.io_uniform.resize(&self.io_ctx, options);
    }

    pub fn new(options: &FxOptions, settings: BlendSettings) -> Self {
        let FxOptions {
            gfx_state,
            fx_state,
        } = options;

        let device = &gfx_state.device;
        let blend_shader = device.create_shader("fx/blend.wgsl", "Blend");

        let io_ctx = UniformContext::from_uniform(&settings.io_uniform, device, "IO");

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Blend layout"),
            bind_group_layouts: &[
                &fx_state.bg_layout,
                &io_ctx.bg_layout,
                settings.blend_layout,
            ],
            push_constant_ranges: &[],
        });

        let create_pipeline = |entry_point: &str| -> wgpu::ComputePipeline {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Blend pipeline"),
                layout: Some(&pipeline_layout),
                module: &blend_shader,
                entry_point,
            })
        };

        let add_pipeline = create_pipeline("add_blend");
        let lerp_upscale_pipeline = create_pipeline("lerp_upscale_blend");
        let lerp_simple_pipeline = create_pipeline("lerp_simple_blend");

        Self {
            add_pipeline,
            lerp_upscale_pipeline,
            lerp_simple_pipeline,
            io_ctx,
            io_uniform: settings.io_uniform,
        }
    }
}
