use super::{
    post_process::{CreateFxOptions, FxIOUniform, PingPongState},
    FxState,
};
use crate::{traits::CustomShader, util::UniformContext};
use egui_wgpu::wgpu;
use encase::ShaderType;
use serde::{Deserialize, Serialize};

pub struct BlendPass {
    add_pipeline: wgpu::ComputePipeline,
    blend_pipeline: wgpu::ComputePipeline,

    io_bg: wgpu::BindGroup,
    io_uniform: FxIOUniform,
}

#[derive(ShaderType, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BlendUniform {
    /// Number between 0 and 1. (0) Is col from input (1) is col from output
    pub io_mix: f32,
    pub aspect: f32,
}

pub struct BlendSettings<'a> {
    pub io_uniform: FxIOUniform,
    pub blend_layout: &'a wgpu::BindGroupLayout,
}

impl BlendPass {
    pub fn add_blend<'a>(
        &'a self,
        ping_pong: &mut PingPongState,
        fx_state: &'a FxState,
        blend_bg: &'a wgpu::BindGroup,
        c_pass: &mut wgpu::ComputePass<'a>,
    ) {
        let (count_x, count_y) = fx_state.count_out(&self.io_uniform);

        c_pass.set_pipeline(&self.add_pipeline);
        c_pass.set_bind_group(0, fx_state.bind_group(ping_pong), &[]);
        c_pass.set_bind_group(1, &self.io_bg, &[]);
        c_pass.set_bind_group(2, blend_bg, &[]);
        c_pass.dispatch_workgroups(count_x, count_y, 1);

        ping_pong.swap();
    }

    pub fn lerp_blend<'a>(
        &'a self,
        ping_pong: &mut PingPongState,
        fx_state: &'a FxState,
        blend_bg: &'a wgpu::BindGroup,
        c_pass: &mut wgpu::ComputePass<'a>,
    ) {
        let (count_x, count_y) = fx_state.count_out(&self.io_uniform);

        c_pass.set_pipeline(&self.blend_pipeline);
        c_pass.set_bind_group(0, fx_state.bind_group(ping_pong), &[]);
        c_pass.set_bind_group(1, &self.io_bg, &[]);
        c_pass.set_bind_group(2, blend_bg, &[]);
        c_pass.dispatch_workgroups(count_x, count_y, 1);

        ping_pong.swap();
    }

    pub fn new(options: &CreateFxOptions, settings: BlendSettings) -> Self {
        let CreateFxOptions {
            gfx_state,
            fx_state,
        } = options;

        let device = &gfx_state.device;
        let blend_shader = device.create_shader("fx/blend.wgsl", "Blend");

        let io_ctx = UniformContext::from_uniform(&settings.io_uniform, device, "IO");

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Blend layout"),
            bind_group_layouts: &[
                &fx_state.bind_group_layout,
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
        let blend_pipeline = create_pipeline("lerp_blend");

        Self {
            add_pipeline,
            blend_pipeline,
            io_bg: io_ctx.bg,
            io_uniform: settings.io_uniform,
        }
    }
}
