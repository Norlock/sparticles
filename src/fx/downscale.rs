use super::{FxIOUniform, FxOptions, FxState};
use crate::{model::GfxState, traits::CustomShader, util::UniformContext};
use egui_wgpu::wgpu;

pub struct Downscale {
    pipeline: wgpu::ComputePipeline,
    io_uniform: FxIOUniform,
    io_ctx: UniformContext,
}

pub struct DownscaleSettings {
    pub io_uniform: FxIOUniform,
}

impl Downscale {
    pub fn compute<'a>(
        &'a self,
        fx_state: &'a FxState,
        gfx_state: &mut GfxState,
        c_pass: &mut wgpu::ComputePass<'a>,
    ) {
        let (count_x, count_y) = fx_state.count_out(&self.io_uniform);

        gfx_state.begin_scope(
            &format!(
                "Downscale from {} to {}",
                self.io_uniform.in_downscale, self.io_uniform.out_downscale
            ),
            c_pass,
        );

        c_pass.set_pipeline(&self.pipeline);
        c_pass.set_bind_group(0, &fx_state.bg, &[]);
        c_pass.set_bind_group(1, &self.io_ctx.bg, &[]);
        c_pass.dispatch_workgroups(count_x, count_y, 1);

        gfx_state.end_scope(c_pass);
    }

    pub fn resize(&mut self, options: &FxOptions) {
        self.io_uniform.resize(&self.io_ctx.buf, options);
    }

    pub fn new(options: &FxOptions, io_uniform: FxIOUniform) -> Self {
        let FxOptions {
            gfx_state,
            fx_state,
        } = options;

        let device = &gfx_state.device;
        let shader = device.create_shader_builtin(&["fx/downscale.wgsl"], "Downscale");

        let io_ctx = UniformContext::from_uniform(&io_uniform, device, "Downscale");

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Downscale layout"),
            bind_group_layouts: &[&fx_state.bg_layout, &io_ctx.bg_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Downscale pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "downscale",
        });

        Self {
            pipeline,
            io_ctx,
            io_uniform,
        }
    }
}
