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
pub struct ColorProcessing {
    color_uniform: ColorProcessingUniform,
    color_buffer: wgpu::Buffer,
    io_uniform: FxIOUniform,
    io_bg: wgpu::BindGroup,
    color_bg: wgpu::BindGroup,
    pipeline: wgpu::ComputePipeline,
    enabled: bool,
    selected_action: ListAction,
    update_uniform: bool,
}

impl Default for ColorProcessingUniform {
    fn default() -> Self {
        Self {
            gamma: 1.0,
            contrast: 2.5,
            brightness: 0.3,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct ColorProcessingSettings {
    pub color_uniform: ColorProcessingUniform,
    pub io_uniform: FxIOUniform,
}

impl Default for ColorProcessingSettings {
    fn default() -> Self {
        Self {
            color_uniform: ColorProcessingUniform::default(),
            io_uniform: FxIOUniform::zero(),
        }
    }
}

pub struct RegisterColorProcessingFx;

impl RegisterPostFx for RegisterColorProcessingFx {
    fn tag(&self) -> &str {
        "color-processing"
    }

    fn create_default(&self, options: &CreateFxOptions) -> Box<dyn PostFx> {
        Box::new(ColorProcessing::new(
            options,
            ColorProcessingSettings::default(),
        ))
    }

    fn import(&self, options: &CreateFxOptions, value: serde_json::Value) -> Box<dyn PostFx> {
        let settings = serde_json::from_value(value).expect("Can't parse color processing Fx");

        Box::new(ColorProcessing::new(options, settings))
    }
}

#[derive(ShaderType, Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct ColorProcessingUniform {
    pub gamma: f32,
    pub contrast: f32,
    pub brightness: f32,
}

impl PostFx for ColorProcessing {
    fn compute<'a>(
        &'a self,
        ping_pong: &mut PingPongState,
        fx_state: &'a FxState,
        c_pass: &mut wgpu::ComputePass<'a>,
    ) {
        c_pass.set_pipeline(&self.pipeline);
        c_pass.set_bind_group(0, fx_state.bind_group(ping_pong), &[]);
        c_pass.set_bind_group(1, &self.io_bg, &[]);
        c_pass.dispatch_workgroups(fx_state.count_x, fx_state.count_y, 1);

        ping_pong.swap(&self.io_uniform);
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

impl HandleAction for ColorProcessing {
    fn selected_action(&mut self) -> &mut ListAction {
        &mut self.selected_action
    }

    fn reset_action(&mut self) {
        self.selected_action = ListAction::None;
    }

    fn export(&self) -> DynamicExport {
        let settings = ColorProcessingSettings {
            color_uniform: self.color_uniform,
            io_uniform: self.io_uniform,
        };

        DynamicExport {
            tag: RegisterColorProcessingFx.tag().to_string(),
            data: serde_json::to_value(settings).expect("Can't unwrap color processing"),
        }
    }

    fn enabled(&self) -> bool {
        self.enabled
    }
}

impl ColorProcessing {
    pub fn new(options: &CreateFxOptions, settings: ColorProcessingSettings) -> Self {
        let CreateFxOptions {
            gfx_state,
            fx_state,
        } = options;

        let device = &gfx_state.device;

        let UniformContext {
            bg: io_bg,
            bg_layout: io_bg_layout,
            ..
        } = UniformContext::from_uniform(&settings.io_uniform, device, "IO");

        let UniformContext {
            buf: color_buffer,
            bg: color_bg,
            bg_layout: color_bg_layout,
        } = UniformContext::from_uniform(&settings.color_uniform, device, "Color processing");

        let shader = device.create_shader("fx/color_processing.wgsl", "Color correction");

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Color pipeline layout"),
            bind_group_layouts: &[&fx_state.bind_group_layout, &io_bg_layout, &color_bg_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Color processing pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "main",
        });

        Self {
            color_uniform: settings.color_uniform,
            color_buffer,
            io_uniform: settings.io_uniform,
            io_bg,
            color_bg,
            pipeline,
            enabled: true,
            update_uniform: false,
            selected_action: ListAction::None,
        }
    }
}
