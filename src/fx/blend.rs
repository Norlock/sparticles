use std::rc::Rc;

use super::FxState;
use crate::{model::GfxState, traits::CustomShader};
use egui_wgpu::wgpu;

pub struct Blend {
    additive_pipeline: wgpu::ComputePipeline,
    count_x: u32,
    count_y: u32,
}

pub enum BlendType {
    ADDITIVE,
    BLEND,
    REPLACE,
}

impl Blend {
    pub fn add<'a>(
        &'a self,
        input: &'a Rc<wgpu::BindGroup>,
        output: &'a Rc<wgpu::BindGroup>,
        c_pass: &mut wgpu::ComputePass<'a>,
    ) {
        c_pass.set_pipeline(&self.additive_pipeline);
        c_pass.set_bind_group(0, &input, &[]);
        c_pass.set_bind_group(1, &output, &[]);
        c_pass.dispatch_workgroups(self.count_x, self.count_y, 1);
    }

    pub fn resize(&mut self, fx_state: &FxState) {
        self.count_x = fx_state.count_x;
        self.count_y = fx_state.count_y;
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

        Self {
            additive_pipeline,
            count_x: fx_state.count_x,
            count_y: fx_state.count_y,
        }
    }
}
