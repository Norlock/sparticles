use super::post_process::CreateFxOptions;
use super::FxState;
use crate::animations::ItemAction;
use crate::model::GuiState;
use crate::traits::PostFx;
use crate::traits::*;
use crate::util::DynamicExport;
use egui_wgpu::wgpu;
use egui_winit::egui::Ui;

pub struct Upscale {
    pipeline: wgpu::ComputePipeline,
}

impl PostFx for Upscale {
    fn compute<'a>(
        &'a self,
        ping_pong_idx: &mut usize,
        fx_state: &'a FxState,
        c_pass: &mut wgpu::ComputePass<'a>,
    ) {
        //let fx_state = &self.fx_state;

        //c_pass.set_pipeline(&self.pipeline);
        //c_pass.set_bind_group(0, fx_inputs[0], &[]);
        //c_pass.set_bind_group(1, fx_state.bind_group(0), &[]);
        //c_pass.dispatch_workgroups(fx_state.count_x, fx_state.count_y, 1);
    }

    fn create_ui(&mut self, _: &mut Ui, _: &GuiState) {}

    fn reserved_space(&self) -> usize {
        1
    }
}

impl HandleAction for Upscale {
    fn selected_action(&mut self) -> &mut ItemAction {
        todo!()
    }

    fn reset_action(&mut self) {
        todo!()
    }

    fn export(&self) -> DynamicExport {
        todo!()
    }

    fn enabled(&self) -> bool {
        todo!()
    }
}

impl Upscale {
    pub fn new(options: &CreateFxOptions) -> Self {
        let CreateFxOptions {
            gfx_state,
            fx_state,
        } = options;

        let device = &gfx_state.device;

        let upscale_shader = device.create_shader("fx/upscale.wgsl", "Upscale");

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Upscale"),
            bind_group_layouts: &[&fx_state.bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Upscale pipeline"),
            layout: Some(&pipeline_layout),
            module: &upscale_shader,
            entry_point: "main",
        });

        Self { pipeline }
    }
}
