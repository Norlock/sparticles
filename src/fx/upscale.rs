use super::{FxState, FxStateOptions};
use crate::traits::*;
use crate::{model::GfxState, traits::PostFx};
use egui_wgpu::wgpu;
use egui_winit::egui::Ui;
use std::rc::Rc;

pub struct Upscale {
    fx_state: FxState,
    pipeline: wgpu::ComputePipeline,
}

impl PostFx for Upscale {
    fn resize(&mut self, gfx_state: &GfxState) {
        let config = &gfx_state.surface_config;
        self.fx_state.resize(config.fx_dimensions(), gfx_state);
    }

    fn compute<'a>(
        &'a self,
        fx_inputs: Vec<&'a Rc<wgpu::BindGroup>>,
        c_pass: &mut wgpu::ComputePass<'a>,
    ) {
        let fx_state = &self.fx_state;

        c_pass.set_pipeline(&self.pipeline);
        c_pass.set_bind_group(0, &fx_inputs[0], &[]);
        c_pass.set_bind_group(1, &fx_state.bind_group(0), &[]);
        c_pass.dispatch_workgroups(fx_state.count_x, fx_state.count_y, 1);
    }

    fn fx_state(&self) -> &FxState {
        &self.fx_state
    }

    fn output(&self) -> &Rc<wgpu::BindGroup> {
        self.fx_state.bind_group(1)
    }

    fn create_ui(&mut self, _ui: &mut Ui, _: &GfxState) {}
}

impl Upscale {
    pub fn new(gfx_state: &GfxState) -> Self {
        let device = &gfx_state.device;
        let config = &gfx_state.surface_config;

        let upscale_shader = device.create_shader("fx/upscale.wgsl", "Upscale");

        let fx_state = FxState::new(FxStateOptions {
            label: "upscale".to_string(),
            tex_dimensions: config.fx_dimensions(),
            gfx_state,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Upscale"),
            bind_group_layouts: &[&fx_state.bind_group_layout, &fx_state.bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Upscale pipeline"),
            layout: Some(&pipeline_layout),
            module: &upscale_shader,
            entry_point: "main",
        });

        Self { pipeline, fx_state }
    }
}
