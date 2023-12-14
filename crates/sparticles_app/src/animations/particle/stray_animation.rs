use std::{any::Any, num::NonZeroU64};

use crate::{
    model::{Clock, EmitterState, GfxState},
    shaders::ShaderOptions,
    traits::{BufferContent, HandleAction, ParticleAnimation, RegisterParticleAnimation},
    util::ListAction,
    util::{persistence::DynamicExport, UniformContext},
};
use egui_wgpu::wgpu;
use encase::ShaderType;
use serde::{Deserialize, Serialize};

#[derive(ShaderType, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StrayUniform {
    pub stray_radians: f32,
    pub from_sec: f32,
    pub until_sec: f32,
}

impl Default for StrayUniform {
    fn default() -> Self {
        Self {
            stray_radians: 5f32.to_radians(),
            from_sec: 1.,
            until_sec: 3.,
        }
    }
}

#[derive(Clone, Copy)]
pub struct RegisterStrayAnimation;

impl RegisterStrayAnimation {
    /// Will append animation to emitter
    pub fn append(uniform: StrayUniform, emitter: &mut EmitterState, gfx_state: &GfxState) {
        let anim = Box::new(StrayAnimation::new(uniform, emitter, gfx_state));

        emitter.push_particle_animation(anim);
    }
}

impl RegisterParticleAnimation for RegisterStrayAnimation {
    fn create_default(
        &self,
        gfx_state: &GfxState,
        emitter: &EmitterState,
    ) -> Box<dyn ParticleAnimation> {
        Box::new(StrayAnimation::new(
            StrayUniform::default(),
            emitter,
            gfx_state,
        ))
    }

    fn tag(&self) -> &'static str {
        "stray"
    }

    fn import(
        &self,
        gfx_state: &GfxState,
        emitter: &EmitterState,
        value: serde_json::Value,
    ) -> Box<dyn ParticleAnimation> {
        let uniform = serde_json::from_value(value).unwrap();
        Box::new(StrayAnimation::new(uniform, emitter, gfx_state))
    }
}

pub struct StrayAnimation {
    pub pipeline: wgpu::ComputePipeline,
    pub uniform: StrayUniform,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub selected_action: ListAction,
    pub update_uniform: bool,
    pub enabled: bool,
}

impl HandleAction for StrayAnimation {
    fn selected_action(&mut self) -> &mut ListAction {
        &mut self.selected_action
    }

    fn export(&self) -> DynamicExport {
        let animation = serde_json::to_value(self.uniform).unwrap();
        let animation_type = RegisterStrayAnimation.tag().to_owned();

        DynamicExport {
            tag: animation_type,
            data: animation,
        }
    }
    fn enabled(&self) -> bool {
        self.enabled
    }
}

impl ParticleAnimation for StrayAnimation {
    fn update(&mut self, _: &Clock, gfx_state: &GfxState) {
        let queue = &gfx_state.queue;

        if self.update_uniform {
            let buf_content = self.uniform.buffer_content();
            queue.write_buffer(&self.buffer, 0, &buf_content);
            self.update_uniform = false;
        }
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
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
}

impl StrayAnimation {
    fn new(uniform: StrayUniform, emitter: &EmitterState, gfx_state: &GfxState) -> Self {
        let device = &gfx_state.device;

        let shader = gfx_state.create_shader_builtin(ShaderOptions {
            if_directives: &[],
            files: &["stray_anim.wgsl"],
            label: "Stray animation",
        });

        let stray_ctx = UniformContext::from_uniform(&uniform, device, "Stray uniform");

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Stray layout"),
            bind_group_layouts: &[&emitter.bg_layout, &stray_ctx.bg_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Stray animation pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "main",
        });

        Self {
            pipeline,
            bind_group: stray_ctx.bg,
            uniform,
            buffer: stray_ctx.buf,
            update_uniform: false,
            selected_action: ListAction::None,
            enabled: true,
        }
    }
}
