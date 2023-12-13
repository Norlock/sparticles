use crate::{
    model::{Clock, EmitterState, GfxState},
    shaders::ShaderOptions,
    traits::*,
    util::{persistence::DynamicExport, ListAction, UniformContext},
};
use egui_wgpu::wgpu;
use encase::ShaderType;
use glam::Vec4;
use serde::{Deserialize, Serialize};
use std::any::Any;

#[derive(ShaderType, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ColorUniform {
    pub from_color: Vec4,
    pub to_color: Vec4,
    pub from_sec: f32,
    pub until_sec: f32,
}

pub struct ColorAnimation {
    pub pipeline: wgpu::ComputePipeline,
    pub bind_group: wgpu::BindGroup,
    pub uniform: ColorUniform,
    pub buffer: wgpu::Buffer,
    pub update_uniform: bool,
    pub selected_action: ListAction,
    pub enabled: bool,
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

#[derive(Clone, Copy)]
pub struct RegisterColorAnimation;

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
            let buf_content = self.uniform.buffer_content();
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

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl ColorAnimation {
    pub fn new(uniform: ColorUniform, emitter: &EmitterState, gfx_state: &GfxState) -> Self {
        let device = &gfx_state.device;

        let shader = gfx_state.create_shader_builtin(ShaderOptions {
            if_directives: &[],
            files: &["color_anim.wgsl"],
            label: "Color animation",
        });

        let buffer_content = uniform.buffer_content();

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
