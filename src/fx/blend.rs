use super::PostProcessState;
use crate::{
    model::GfxState,
    traits::{CustomShader, PostProcessFx},
};
use egui_wgpu::wgpu;

pub struct Blend {
    pipeline: wgpu::ComputePipeline,
    count_x: u32,
    count_y: u32,
}

impl PostProcessFx for Blend {
    fn compute<'a>(&'a self, input: Vec<&'a wgpu::BindGroup>, c_pass: &mut wgpu::ComputePass<'a>) {
        // 0 is frame, 1 is fx,
        c_pass.set_pipeline(&self.pipeline);
        c_pass.set_bind_group(0, input[0], &[]);
        c_pass.set_bind_group(1, input[1], &[]);
        c_pass.dispatch_workgroups(self.count_x, self.count_y, 1);
    }

    fn enabled(&self) -> bool {
        true
    }

    fn resize(&mut self, _: &GfxState, dispatch_xy: &[u32; 2]) {
        self.count_x = dispatch_xy[0];
        self.count_y = dispatch_xy[1];
    }
}

impl Blend {
    pub fn new(
        gfx_state: &GfxState,
        pp: &PostProcessState,
        fx_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let device = &gfx_state.device;

        let blend_shader = device.create_shader("fx/blend.wgsl", "Blend");

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Blend layout"),
            bind_group_layouts: &[&pp.bind_group_layout, fx_layout],
            push_constant_ranges: &[],
        });

        // TODO multiple entry points for different types of blend
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Blend pipeline"),
            layout: Some(&pipeline_layout),
            module: &blend_shader,
            entry_point: "additive",
        });

        let count_x = pp.res.count_x;
        let count_y = pp.res.count_y;

        Self {
            pipeline,
            count_x,
            count_y,
        }
    }
}
