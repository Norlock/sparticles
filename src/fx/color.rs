use super::{
    post_process::{CreateFxOptions, FxIOUniform, PingPongState},
    FxState,
};
use crate::{
    model::{GfxState, GuiState},
    traits::{CustomShader, HandleAction, PostFx, RegisterPostFx},
    util::{CommonBuffer, DynamicExport, ListAction, UniformContext},
};
use egui_wgpu::wgpu;
use egui_winit::egui::{self, Slider};
use encase::ShaderType;
use serde::{Deserialize, Serialize};

#[allow(unused)]
pub struct ColorFx {
    color_uniform: ColorFxUniform,
    color_buffer: wgpu::Buffer,
    color_bg: wgpu::BindGroup,
    io_uniform: FxIOUniform,
    io_ctx: UniformContext,
    general_pipeline: wgpu::ComputePipeline,
    tonemap_pipeline: wgpu::ComputePipeline,
    selected_action: ListAction,
    update_uniform: bool,
    enabled: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct ColorFxSettings {
    pub color_uniform: ColorFxUniform,
    pub io_uniform: FxIOUniform,
}

pub struct RegisterColorFx;

impl RegisterPostFx for RegisterColorFx {
    fn tag(&self) -> &str {
        "color-processing"
    }

    fn create_default(&self, options: &CreateFxOptions) -> Box<dyn PostFx> {
        let settings = ColorFxSettings {
            color_uniform: ColorFxUniform::default_rgb(),
            io_uniform: FxIOUniform::zero(&options.fx_state),
        };

        Box::new(ColorFx::new(options, settings))
    }

    fn import(&self, options: &CreateFxOptions, value: serde_json::Value) -> Box<dyn PostFx> {
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
        // TODO find neutral settings
        Self {
            gamma: 1.0,
            contrast: 2.5,
            brightness: 0.3,
        }
    }
}

impl PostFx for ColorFx {
    fn resize(&mut self, options: &CreateFxOptions) {
        self.io_uniform.resize(&self.io_ctx, options);
    }

    fn compute<'a>(
        &'a self,
        ping_pong: &mut PingPongState,
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

        ping_pong.swap();
    }

    fn update(&mut self, gfx_state: &GfxState) {
        if self.update_uniform {
            let queue = &gfx_state.queue;

            let color_content = CommonBuffer::uniform_content(&self.color_uniform);
            queue.write_buffer(&self.color_buffer, 0, &color_content);
            self.update_uniform = false;
        }
    }

    fn create_ui(&mut self, ui: &mut egui::Ui, ui_state: &GuiState) {
        let mut uniform = self.color_uniform;

        self.selected_action = ui_state.create_li_header(ui, "Color correction");
        ui.add(Slider::new(&mut uniform.gamma, 0.1..=4.0).text("Gamma"));
        ui.add(Slider::new(&mut uniform.contrast, 0.1..=4.0).text("Contrast"));
        ui.add(Slider::new(&mut uniform.brightness, 0.01..=1.0).text("Brightness"));
        ui.checkbox(&mut self.enabled, "Enabled");

        if self.color_uniform != uniform {
            self.update_uniform = true;
            self.color_uniform = uniform;
        }
    }
}

impl HandleAction for ColorFx {
    fn selected_action(&mut self) -> &mut ListAction {
        &mut self.selected_action
    }

    fn reset_action(&mut self) {
        self.selected_action = ListAction::None;
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
        ping_pong: &mut PingPongState,
        fx_state: &'a FxState,
        c_pass: &mut wgpu::ComputePass<'a>,
    ) {
        let (count_x, count_y) = fx_state.count_out(&self.io_uniform);

        c_pass.set_pipeline(&self.tonemap_pipeline);
        c_pass.set_bind_group(0, &fx_state.bg, &[]);
        c_pass.set_bind_group(1, &self.io_ctx.bg, &[]);
        c_pass.set_bind_group(2, &self.color_bg, &[]);
        c_pass.dispatch_workgroups(count_x, count_y, 1);

        ping_pong.swap();
    }

    pub fn new(options: &CreateFxOptions, settings: ColorFxSettings) -> Self {
        let CreateFxOptions {
            gfx_state,
            fx_state,
        } = options;

        let device = &gfx_state.device;

        let io_ctx = UniformContext::from_uniform(&settings.io_uniform, device, "IO");
        let col_ctx = UniformContext::from_uniform(&settings.color_uniform, device, "Color Fx");

        let shader = device.create_shader("fx/color_processing.wgsl", "Color correction");

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

        let general_pipeline = create_pipeline("general");
        let tonemap_pipeline = create_pipeline("tonemap");

        Self {
            color_uniform: settings.color_uniform,
            color_buffer: col_ctx.buf,
            io_uniform: settings.io_uniform,
            io_ctx,
            color_bg: col_ctx.bg,
            general_pipeline,
            tonemap_pipeline,
            enabled: true,
            update_uniform: false,
            selected_action: ListAction::None,
        }
    }
}
