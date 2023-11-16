use crate::{
    model::{Clock, EmitterState, GfxState, GuiState},
    traits::{
        CalculateBufferSize, CustomShader, HandleAction, ParticleAnimation,
        RegisterParticleAnimation,
    },
    util::persistence::DynamicExport,
    util::ListAction,
};
use egui_wgpu::wgpu;
use egui_winit::egui::{DragValue, Slider, Ui};
use serde::{Deserialize, Serialize};
use wgpu::util::DeviceExt;

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
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

impl StrayUniform {
    fn create_buffer_content(&self) -> [f32; 4] {
        [self.stray_radians, self.from_sec, self.until_sec, 0.]
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

    fn tag(&self) -> &str {
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

struct StrayAnimation {
    pipeline: wgpu::ComputePipeline,
    uniform: StrayUniform,
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    selected_action: ListAction,
    update_uniform: bool,
    enabled: bool,
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
            let buf_content_raw = self.uniform.create_buffer_content();
            let buf_content = bytemuck::cast_slice(&buf_content_raw);
            queue.write_buffer(&self.buffer, 0, buf_content);
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

    fn create_ui(&mut self, ui: &mut Ui, ui_state: &GuiState) {
        self.selected_action = ui_state.create_li_header(ui, "Stray animation");

        let mut gui = self.uniform;
        let mut stray_degrees = gui.stray_radians.to_degrees();

        ui.horizontal(|ui| {
            ui.label("Animate from sec");
            ui.add(DragValue::new(&mut gui.from_sec).speed(0.1));
        });

        ui.horizontal(|ui| {
            ui.label("Animate until sec");
            ui.add(DragValue::new(&mut gui.until_sec).speed(0.1));
        });

        ui.spacing_mut().slider_width = 200.0;

        ui.add(
            Slider::new(&mut stray_degrees, 0.0..=45.)
                .step_by(0.1)
                .text("Stray degrees"),
        );

        ui.checkbox(&mut self.enabled, "Enabled");

        gui.stray_radians = stray_degrees.to_radians();

        if self.uniform != gui {
            self.update_uniform = true;
            self.uniform = gui;
        }
    }
}

impl StrayAnimation {
    fn new(uniform: StrayUniform, emitter: &EmitterState, gfx_state: &GfxState) -> Self {
        let device = &gfx_state.device;
        let shader = device.create_shader("stray_anim.wgsl", "Stray animation");

        let animation_uniform = uniform.create_buffer_content();

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Stray buffer"),
            contents: bytemuck::cast_slice(&animation_uniform),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let animation_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // Uniform data
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: animation_uniform.cal_buffer_size(),
                    },
                    count: None,
                },
            ],
            label: None,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &animation_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("Stray animation"),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Stray layout"),
            bind_group_layouts: &[&emitter.bg_layout, &animation_layout],
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
            bind_group,
            uniform,
            buffer,
            update_uniform: false,
            selected_action: ListAction::None,
            enabled: true,
        }
    }
}
