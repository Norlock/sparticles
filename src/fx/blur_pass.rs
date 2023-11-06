use super::FxIOSwapCtx;
use super::FxIOUniform;
use super::FxOptions;
use super::FxState;
use crate::model::GfxState;
use crate::traits::*;
use egui_wgpu::wgpu;

pub struct BlurPass {
    pub blur_pipeline_x: wgpu::ComputePipeline,
    pub blur_pipeline_y: wgpu::ComputePipeline,
    pub split_pipeline: wgpu::ComputePipeline,

    io_ctx: FxIOSwapCtx,
}

#[derive(Debug)]
pub struct BlurPassSettings<'a> {
    pub blur_layout: &'a wgpu::BindGroupLayout,
    pub io_idx: (u32, u32),
    pub downscale: f32,
}

impl BlurPass {
    /// Computes horizontal vertical gaussian blur
    pub fn compute_gaussian<'a>(
        &'a self,
        fx_state: &'a FxState,
        gfx_state: &mut GfxState,
        blur_bg: &'a wgpu::BindGroup,
        c_pass: &mut wgpu::ComputePass<'a>,
    ) {
        gfx_state.begin_scope("Gaussian", c_pass);

        let (count_x, count_y) = fx_state.count_out(&self.io_ctx.uniforms[0]);

        c_pass.set_pipeline(&self.blur_pipeline_x);
        c_pass.set_bind_group(0, &fx_state.bg, &[]);
        c_pass.set_bind_group(1, &self.io_ctx.bgs[0], &[]);
        c_pass.set_bind_group(2, &blur_bg, &[]);
        c_pass.dispatch_workgroups(count_x, count_y, 1);

        c_pass.set_pipeline(&self.blur_pipeline_y);
        c_pass.set_bind_group(0, &fx_state.bg, &[]);
        c_pass.set_bind_group(1, &self.io_ctx.bgs[1], &[]);
        c_pass.set_bind_group(2, &blur_bg, &[]);
        c_pass.dispatch_workgroups(count_x, count_y, 1);

        gfx_state.end_scope(c_pass);
    }

    pub fn split<'a>(
        &'a self,
        fx_state: &'a FxState,
        gfx_state: &mut GfxState,
        blur_bg: &'a wgpu::BindGroup,
        c_pass: &mut wgpu::ComputePass<'a>,
    ) {
        gfx_state.begin_scope("Split", c_pass);

        c_pass.set_pipeline(&self.split_pipeline);
        c_pass.set_bind_group(0, &fx_state.bg, &[]);
        c_pass.set_bind_group(1, &self.io_ctx.bgs[0], &[]);
        c_pass.set_bind_group(2, &blur_bg, &[]);
        c_pass.dispatch_workgroups(fx_state.count_x, fx_state.count_y, 1);

        gfx_state.end_scope(c_pass);
    }

    pub fn resize(&mut self, options: &FxOptions) {
        self.io_ctx.resize(options);
    }

    pub fn new(options: &FxOptions, settings: BlurPassSettings) -> Self {
        let FxOptions {
            gfx_state,
            fx_state,
        } = options;

        let BlurPassSettings {
            blur_layout,
            io_idx: (in_idx, out_idx),
            downscale,
        } = settings;

        let device = &gfx_state.device;
        let blur_shader = device.create_shader("fx/gaussian_blur.wgsl", "Gaussian blur");

        let io_ping = FxIOUniform::asymetric_scaled(options.fx_state, in_idx, out_idx, downscale);
        let io_pong = FxIOUniform::asymetric_scaled(options.fx_state, out_idx, in_idx, downscale);
        let io_ctx = FxIOSwapCtx::new([io_ping, io_pong], device, "IO Swap blur");

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
            io_ctx,
        }
    }
}
