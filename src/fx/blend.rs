use super::{
    post_process::{CreateFxOptions, FxMetaUniform, MetaUniformCompute},
    FxState,
};
use crate::{
    model::{GfxState, GuiState},
    traits::{CustomShader, HandleAction, PostFx},
    util::{DynamicExport, ItemAction},
};
use egui_wgpu::wgpu;
use egui_winit::egui::Ui;

pub struct Blend {
    additive_pipeline: wgpu::ComputePipeline,
    blend_type: BlendType,
    pub meta_compute: MetaUniformCompute,
}

pub enum BlendType {
    ADDITIVE,
    BLEND,
    REPLACE,
}

impl PostFx for Blend {
    fn compute<'a>(
        &'a self,
        ping_pong_idx: &mut usize,
        fx_state: &'a FxState,
        c_pass: &mut wgpu::ComputePass<'a>,
    ) {
        match self.blend_type {
            BlendType::ADDITIVE => {
                c_pass.set_pipeline(&self.additive_pipeline);
            }
            BlendType::BLEND => {}
            BlendType::REPLACE => {}
        }

        c_pass.set_bind_group(0, fx_state.bind_group(*ping_pong_idx), &[]);
        c_pass.set_bind_group(1, &self.meta_compute.bind_group, &[]);
        c_pass.dispatch_workgroups(fx_state.count_x, fx_state.count_y, 1);

        *ping_pong_idx += 1;
    }

    fn create_ui(&mut self, ui: &mut Ui, ui_state: &GuiState) {}

    fn update(&mut self, gfx_state: &GfxState) {}
}

impl HandleAction for Blend {
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

impl Blend {
    pub fn new(options: &CreateFxOptions, blend_type: BlendType, fx_meta: FxMetaUniform) -> Self {
        let CreateFxOptions {
            gfx_state,
            fx_state,
        } = options;

        let device = &gfx_state.device;

        let blend_shader = device.create_shader("fx/blend.wgsl", "Blend");

        let meta_compute = fx_meta.into_compute(device);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Blend layout"),
            bind_group_layouts: &[&fx_state.bind_group_layout, &meta_compute.bind_group_layout],
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
            blend_type,
            meta_compute,
        }
    }
}
