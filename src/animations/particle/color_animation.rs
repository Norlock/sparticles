use crate::{
    model::{Clock, EmitterState, GfxState, GuiState},
    shaders::ShaderOptions,
    traits::*,
    util::{persistence::DynamicExport, ListAction, UniformContext},
};
use egui_wgpu::wgpu;
use egui_winit::egui::{
    color_picker::{color_edit_button_rgba, Alpha},
    DragValue, Rgba, Ui,
};
use glam::Vec4;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ColorUniform {
    pub from_color: Vec4,
    pub to_color: Vec4,
    pub from_sec: f32,
    pub until_sec: f32,
}

impl Default for ColorUniform {
    fn default() -> Self {
        Self {
            from_color: Vec4::from_rgb(0, 255, 0),
            to_color: Vec4::from_rgb(0, 0, 255),
            from_sec: 0.,
            until_sec: 0.5,
        }
    }
}

impl ColorUniform {
    fn create_buffer_content(&self) -> Vec<u8> {
        let raw = [
            self.from_color.x,
            self.from_color.y,
            self.from_color.z,
            self.from_color.w,
            self.to_color.x,
            self.to_color.y,
            self.to_color.z,
            self.to_color.w,
            self.from_sec,
            self.until_sec,
            0., // Padding
            0., // Padding
        ];

        bytemuck::cast_slice(&raw).to_vec()
    }
}

#[derive(Clone, Copy)]
pub struct RegisterColorAnimation;

impl RegisterColorAnimation {
    /// Will append animation to emitter
    pub fn append(uniform: ColorUniform, emitter: &mut EmitterState, gfx_state: &GfxState) {
        let anim = Box::new(ColorAnimation::new(uniform, emitter, gfx_state));

        emitter.push_particle_animation(anim);
    }
}

impl RegisterParticleAnimation for RegisterColorAnimation {
    fn tag(&self) -> &'static str {
        "color"
    }

    fn create_default(
        &self,
        gfx_state: &GfxState,
        emitter: &EmitterState,
    ) -> Box<dyn ParticleAnimation> {
        Box::new(ColorAnimation::new(
            ColorUniform::default(),
            emitter,
            gfx_state,
        ))
    }

    fn import(
        &self,
        gfx_state: &GfxState,
        emitter: &EmitterState,
        value: serde_json::Value,
    ) -> Box<dyn ParticleAnimation> {
        let uniform = serde_json::from_value(value).unwrap();
        Box::new(ColorAnimation::new(uniform, emitter, gfx_state))
    }
}

struct ColorAnimation {
    pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
    uniform: ColorUniform,
    buffer: wgpu::Buffer,
    update_uniform: bool,
    selected_action: ListAction,
    enabled: bool,
}

impl HandleAction for ColorAnimation {
    fn selected_action(&mut self) -> &mut ListAction {
        &mut self.selected_action
    }

    fn export(&self) -> DynamicExport {
        let data = serde_json::to_value(self.uniform).unwrap();
        let tag = RegisterColorAnimation.tag().to_owned();

        DynamicExport { tag, data }
    }

    fn enabled(&self) -> bool {
        self.enabled
    }
}

impl ParticleAnimation for ColorAnimation {
    fn update(&mut self, _clock: &Clock, gfx_state: &GfxState) {
        if self.update_uniform {
            let buf_content = self.uniform.create_buffer_content();
            gfx_state.queue.write_buffer(&self.buffer, 0, &buf_content);
            self.update_uniform = false;
        }
    }

    fn compute<'a>(
        &'a self,
        emitter: &'a EmitterState,
        clock: &Clock,
        compute_pass: &mut wgpu::ComputePass<'a>,
    ) {
        let nr = clock.get_bindgroup_nr();

        compute_pass.set_pipeline(&self.pipeline);
        compute_pass.set_bind_group(0, &emitter.bgs[nr], &[]);
        compute_pass.set_bind_group(1, &self.bind_group, &[]);
        compute_pass.dispatch_workgroups(emitter.dispatch_x_count, 1, 1);
    }

    fn recreate(&self, gfx_state: &GfxState, emitter: &EmitterState) -> Box<dyn ParticleAnimation> {
        Box::new(Self::new(self.uniform, emitter, gfx_state))
    }

    fn create_ui(&mut self, ui: &mut Ui, gui_state: &GuiState) {
        self.selected_action = gui_state.create_li_header(ui, "Color animation");

        let mut gui = self.uniform;

        ui.horizontal(|ui| {
            ui.label("Animate from sec");
            ui.add(DragValue::new(&mut gui.from_sec).speed(0.1));
        });

        ui.horizontal(|ui| {
            ui.label("Animate until sec");
            ui.add(DragValue::new(&mut gui.until_sec).speed(0.1));
        });

        let f_col = gui.from_color;
        let t_col = gui.to_color;
        let mut from_color = Rgba::from_rgba_premultiplied(f_col.x, f_col.y, f_col.z, f_col.w);
        let mut to_color = Rgba::from_rgba_premultiplied(t_col.x, t_col.y, t_col.z, t_col.w);

        ui.horizontal(|ui| {
            ui.label("From color: ");
            color_edit_button_rgba(ui, &mut from_color, Alpha::Opaque);
        });

        ui.horizontal(|ui| {
            ui.label("To color: ");
            color_edit_button_rgba(ui, &mut to_color, Alpha::Opaque);
        });

        ui.checkbox(&mut self.enabled, "Enabled");

        gui.from_color = from_color.to_array().into();
        gui.to_color = to_color.to_array().into();

        if self.uniform != gui {
            self.update_uniform = true;
            self.uniform = gui;
        }
    }
}

impl ColorAnimation {
    fn new(uniform: ColorUniform, emitter: &EmitterState, gfx_state: &GfxState) -> Self {
        let device = &gfx_state.device;

        let shader = gfx_state.create_shader_builtin(ShaderOptions {
            if_directives: &[],
            files: &["color_anim.wgsl"],
            label: "Color animation",
        });

        let buffer_content = uniform.create_buffer_content();

        let color_ctx = UniformContext::from_content(&buffer_content, device, "Color animation");

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute layout"),
            bind_group_layouts: &[&emitter.bg_layout, &color_ctx.bg_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Color animation pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "main",
        });

        Self {
            pipeline,
            bind_group: color_ctx.bg,
            buffer: color_ctx.buf,
            uniform,
            update_uniform: false,
            selected_action: ListAction::None,
            enabled: true,
        }
    }
}
