use super::FxState;
use crate::{model::GfxState, traits::CustomShader};
use egui_wgpu::wgpu;

pub struct Blend {
    additive_pipeline: wgpu::ComputePipeline,
}

pub struct BlendCompute<'a> {
    pub input: &'a wgpu::BindGroup,
    pub output: &'a wgpu::BindGroup,
    pub count_x: u32,
    pub count_y: u32,
}

pub enum BlendType {
    ADDITIVE,
    BLEND,
    REPLACE,
}

impl Blend {
    pub fn add<'a>(&'a self, compute: BlendCompute<'a>, c_pass: &mut wgpu::ComputePass<'a>) {
        let BlendCompute {
            input,
            output,
            count_x,
            count_y,
        } = compute;

        c_pass.set_pipeline(&self.additive_pipeline);
        c_pass.set_bind_group(0, input, &[]);
        c_pass.set_bind_group(1, output, &[]);
        c_pass.dispatch_workgroups(count_x, count_y, 1);
    }

    pub fn new(gfx_state: &GfxState, fx_state: &FxState) -> Self {
        let device = &gfx_state.device;

        let blend_shader = device.create_shader("fx/blend.wgsl", "Blend");

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Blend layout"),
            bind_group_layouts: &[&fx_state.bind_group_layout, &fx_state.bind_group_layout],
            push_constant_ranges: &[],
        });

        // TODO multiple entry points for different types of blend
        let additive_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Blend pipeline"),
            layout: Some(&pipeline_layout),
            module: &blend_shader,
            entry_point: "additive",
        });

        Self { additive_pipeline }
    }
}