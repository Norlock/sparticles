use super::post_process::CreateFxOptions;
use super::post_process::FxIOUniform;
use super::FxState;
use crate::traits::*;
use crate::util::UniformContext;
use egui_wgpu::wgpu;

pub struct BlurPass {
    pub blur_pipeline_x: wgpu::ComputePipeline,
    pub blur_pipeline_y: wgpu::ComputePipeline,
    pub split_pipeline: wgpu::ComputePipeline,

    pub io_uniform: FxIOUniform,
    io_ctx: UniformContext,
}

#[derive(Debug)]
pub struct BlurPassSettings<'a> {
    pub blur_layout: &'a wgpu::BindGroupLayout,
    pub io_uniform: FxIOUniform,
}

impl BlurPass {
    pub fn compute_hor_ver<'a>(
        &'a self,
        fx_state: &'a FxState,
        blur_bg: &'a wgpu::BindGroup,
        c_pass: &mut wgpu::ComputePass<'a>,
    ) {
        let (count_x, count_y) = fx_state.count_out(&self.io_uniform);

        c_pass.set_pipeline(&self.blur_pipeline_x);
        c_pass.set_bind_group(0, &fx_state.bg, &[]);
        c_pass.set_bind_group(1, &self.io_ctx.bg, &[]);
        c_pass.set_bind_group(2, &blur_bg, &[]);
        c_pass.dispatch_workgroups(count_x, count_y, 1);

        c_pass.set_pipeline(&self.blur_pipeline_y);
        c_pass.set_bind_group(0, &fx_state.bg, &[]);
        c_pass.set_bind_group(1, &self.io_ctx.bg, &[]);
        c_pass.set_bind_group(2, &blur_bg, &[]);
        c_pass.dispatch_workgroups(count_x, count_y, 1);
    }

    pub fn compute_split<'a>(
        &'a self,
        fx_state: &'a FxState,
        blur_bg: &'a wgpu::BindGroup,
        c_pass: &mut wgpu::ComputePass<'a>,
    ) {
        c_pass.set_pipeline(&self.split_pipeline);
        c_pass.set_bind_group(0, &fx_state.bg, &[]);
        c_pass.set_bind_group(1, &self.io_ctx.bg, &[]);
        c_pass.set_bind_group(2, &blur_bg, &[]);
        c_pass.dispatch_workgroups(fx_state.count_x, fx_state.count_y, 1);
    }

    pub fn resize(&mut self, options: &CreateFxOptions) {
        self.io_uniform.resize(&self.io_ctx, options);
    }

    pub fn new(options: &CreateFxOptions, settings: BlurPassSettings) -> Self {
        let CreateFxOptions {
            gfx_state,
            fx_state,
        } = options;

        let BlurPassSettings {
            blur_layout,
            io_uniform,
        } = settings;

        let device = &gfx_state.device;
        let blur_shader = device.create_shader("fx/gaussian_blur.wgsl", "Gaussian blur");

        let io_ctx = UniformContext::from_uniform(&io_uniform, device, "IO");

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Split layout"),
            bind_group_layouts: &[&fx_state.bg_layout, &io_ctx.bg_layout, &blur_layout],
            push_constant_ranges: &[],
        });

        let new_pipeline = |entry_point: &str| -> wgpu::ComputePipeline {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Gaussian blur pipeline"),
                layout: Some(&pipeline_layout),
                module: &blur_shader,
                entry_point,
            })
        };

        let blur_pipeline_x = new_pipeline("apply_blur_x");
        let blur_pipeline_y = new_pipeline("apply_blur_y");
        let split_pipeline = new_pipeline("split_bloom");

        Self {
            blur_pipeline_x,
            blur_pipeline_y,
            split_pipeline,
            io_uniform,
            io_ctx,
        }
    }
}
