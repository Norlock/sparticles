use std::any::Any;

use super::{FxIOUniform, FxOptions, FxState};
use crate::{
    model::{Camera, GfxState},
    shaders::{ShaderOptions, SDR_TONEMAPPING},
    traits::{HandleAction, PostFx, RegisterPostFx},
    util::{CommonBuffer, DynamicExport, ListAction, UniformContext},
};
use egui_wgpu::wgpu;
use encase::ShaderType;
use serde::{Deserialize, Serialize};

pub enum UpdateAction {
    UpdateBuffer,
}

#[allow(unused)]
pub struct ColorFx {
    pub color_uniform: ColorFxUniform,
    pub color_buffer: wgpu::Buffer,
    pub color_bg: wgpu::BindGroup,
    pub io_uniform: FxIOUniform,
    pub io_ctx: UniformContext,
    pub general_pipeline: wgpu::ComputePipeline,
    pub tonemap_pipeline: wgpu::ComputePipeline,
    pub selected_action: ListAction,
    pub enabled: bool,
    pub update_event: Option<UpdateAction>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct ColorFxSettings {
    pub color_uniform: ColorFxUniform,
    pub io_uniform: FxIOUniform,
}

pub struct RegisterColorFx;

impl RegisterPostFx for RegisterColorFx {
    fn tag(&self) -> &'static str {
        "color-processing"
    }

    fn create_default(&self, options: &FxOptions) -> Box<dyn PostFx> {
        let settings = ColorFxSettings {
            color_uniform: ColorFxUniform::default_rgb(),
            io_uniform: FxIOUniform::zero(&options.fx_state),
        };

        Box::new(ColorFx::new(options, settings))
    }

    fn import(&self, options: &FxOptions, value: serde_json::Value) -> Box<dyn PostFx> {
        let settings = serde_json::from_value(value).expect("Can't parse color processing Fx");

        Box::new(ColorFx::new(options, settings))
    }
}

#[derive(ShaderType, Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct ColorFxUniform {
    pub gamma: f32,
    pub contrast: f32,
    pub brightness: f32,
}

impl ColorFxUniform {
    pub fn default_srgb() -> Self {
        Self {
            gamma: 2.2,
            contrast: 2.5,
            brightness: 0.3,
        }
    }

    pub fn default_rgb() -> Self {
        Self {
            gamma: 1.0,
            contrast: 2.5,
            brightness: 0.3,
        }
    }
}

impl PostFx for ColorFx {
    fn resize(&mut self, options: &FxOptions) {
        self.io_uniform.resize(&self.io_ctx.buf, options);
    }

    fn compute<'a>(
        &'a self,
        fx_state: &'a FxState,
        gfx_state: &mut GfxState,
        c_pass: &mut wgpu::ComputePass<'a>,
    ) {
        gfx_state
            .profiler
            .begin_scope("Color Fx", c_pass, &gfx_state.device);
        c_pass.set_pipeline(&self.general_pipeline);
        c_pass.set_bind_group(0, &fx_state.bg, &[]);
        c_pass.set_bind_group(1, &self.io_ctx.bg, &[]);
        c_pass.set_bind_group(2, &self.color_bg, &[]);
        c_pass.dispatch_workgroups(fx_state.count_x, fx_state.count_y, 1);
        gfx_state.profiler.end_scope(c_pass).unwrap();
    }

    fn update(&mut self, gfx_state: &GfxState, _: &mut Camera) {
        match self.update_event.take() {
            Some(UpdateAction::UpdateBuffer) => {
                let queue = &gfx_state.queue;
                let color_content = CommonBuffer::uniform_content(&self.color_uniform);
                queue.write_buffer(&self.color_buffer, 0, &color_content);
            }
            None => {}
        }
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl HandleAction for ColorFx {
    fn selected_action(&mut self) -> &mut ListAction {
        &mut self.selected_action
    }

    fn export(&self) -> DynamicExport {
        let settings = ColorFxSettings {
            color_uniform: self.color_uniform,
            io_uniform: self.io_uniform,
        };

        DynamicExport {
            tag: RegisterColorFx.tag().to_string(),
            data: serde_json::to_value(settings).expect("Can't unwrap color processing"),
        }
    }

    fn enabled(&self) -> bool {
        self.enabled
    }
}

impl ColorFx {
    pub fn compute_tonemap<'a>(
        &'a self,
        fx_state: &'a FxState,
        gfx_state: &mut GfxState,
        c_pass: &mut wgpu::ComputePass<'a>,
    ) {
        let (count_x, count_y) = fx_state.count_out(&self.io_uniform);

        gfx_state.begin_scope("Tonemapping", c_pass);

        c_pass.set_pipeline(&self.tonemap_pipeline);
        c_pass.set_bind_group(0, &fx_state.bg, &[]);
        c_pass.set_bind_group(1, &self.io_ctx.bg, &[]);
        c_pass.set_bind_group(2, &self.color_bg, &[]);
        c_pass.dispatch_workgroups(count_x, count_y, 1);

        gfx_state.end_scope(c_pass);
    }

    pub fn new(options: &FxOptions, settings: ColorFxSettings) -> Self {
        let FxOptions {
            gfx: gfx_state,
            fx_state,
        } = options;

        let device = &gfx_state.device;

        let io_ctx = UniformContext::from_uniform(&settings.io_uniform, device, "IO");
        let col_ctx = UniformContext::from_uniform(&settings.color_uniform, device, "Color Fx");

        let shader = gfx_state.create_shader_builtin(ShaderOptions {
            if_directives: &[],
            files: &[SDR_TONEMAPPING, "fx/color_processing.wgsl"],
            label: "Color processing",
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Color pipeline layout"),
            bind_group_layouts: &[&fx_state.bg_layout, &io_ctx.bg_layout, &col_ctx.bg_layout],
            push_constant_ranges: &[],
        });

        let create_pipeline = |entry_point: &str| -> wgpu::ComputePipeline {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Color fx pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point,
            })
        };

        let general_pipeline = create_pipeline("cs_general");
        let tonemap_pipeline = create_pipeline("cs_tonemap");

        Self {
            color_uniform: settings.color_uniform,
            color_buffer: col_ctx.buf,
            io_uniform: settings.io_uniform,
            io_ctx,
            color_bg: col_ctx.bg,
            general_pipeline,
            tonemap_pipeline,
            enabled: true,
            update_event: None,
            selected_action: ListAction::None,
        }
    }
}
