use crate::{
    model::{Clock, EmitterState, GfxState, GuiState, LifeCycle},
    shaders::ShaderOptions,
    traits::{HandleAction, ParticleAnimation, RegisterParticleAnimation},
    util::persistence::DynamicExport,
    util::ListAction,
};
use egui_wgpu::wgpu::{self, util::DeviceExt};
use egui_winit::egui::{DragValue, Ui};
use glam::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ForceUniform {
    pub life_cycle: LifeCycle,
    pub velocity: Vec3,
    /// Applied on a 1.0 particle size unit
    pub mass_per_unit: f32,
}

impl Default for ForceUniform {
    fn default() -> Self {
        Self {
            life_cycle: LifeCycle {
                from_sec: 0.,
                until_sec: 5.,
                lifetime_sec: 10.,
            },
            velocity: [-15., -15., 0.].into(),
            mass_per_unit: 0.5,
        }
    }
}

impl ForceUniform {
    fn create_buffer_content(&self) -> [f32; 4] {
        [
            self.velocity.x,
            self.velocity.y,
            self.velocity.z,
            self.mass_per_unit,
        ]
    }
}

#[derive(Clone, Copy)]
pub struct RegisterForceAnimation;

impl RegisterForceAnimation {
    /// Will append animation to emitter
    pub fn append(uniform: ForceUniform, emitter: &mut EmitterState, gfx_state: &GfxState) {
        let anim = Box::new(ForceAnimation::new(uniform, emitter, gfx_state));

        emitter.push_particle_animation(anim);
    }
}

impl RegisterParticleAnimation for RegisterForceAnimation {
    fn tag(&self) -> &'static str {
        "force"
    }

    fn create_default(
        &self,
        gfx_state: &GfxState,
        emitter: &EmitterState,
    ) -> Box<dyn ParticleAnimation> {
        Box::new(ForceAnimation::new(
            ForceUniform::default(),
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
        Box::new(ForceAnimation::new(uniform, emitter, gfx_state))
    }
}

pub struct ForceAnimation {
    pipeline: wgpu::ComputePipeline,
    uniform: ForceUniform,
    bind_group: wgpu::BindGroup,
    buffer: wgpu::Buffer,
    update_uniform: bool,
    should_animate: bool,
    selected_action: ListAction,
    enabled: bool,
}

impl HandleAction for ForceAnimation {
    fn export(&self) -> DynamicExport {
        let animation = serde_json::to_value(self.uniform).unwrap();

        DynamicExport {
            tag: RegisterForceAnimation.tag().to_owned(),
            data: animation,
        }
    }

    fn selected_action(&mut self) -> &mut ListAction {
        &mut self.selected_action
    }

    fn enabled(&self) -> bool {
        self.enabled
    }
}

impl ParticleAnimation for ForceAnimation {
    fn update(&mut self, clock: &Clock, gfx_state: &GfxState) {
        let queue = &gfx_state.queue;
        let uniform = &self.uniform;
        let current_sec = uniform.life_cycle.get_current_sec(clock);
        self.should_animate = uniform.life_cycle.shoud_animate(current_sec);

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
        if !self.should_animate {
            return;
        }

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
        self.selected_action = ui_state.create_li_header(ui, "Force animation");

        let mut gui = self.uniform;

        ui.horizontal(|ui| {
            ui.label("Animate from sec");
            ui.add(DragValue::new(&mut gui.life_cycle.from_sec).speed(0.1));
        });

        ui.horizontal(|ui| {
            ui.label("Animate until sec");
            ui.add(DragValue::new(&mut gui.life_cycle.until_sec).speed(0.1));
        });

        ui.horizontal(|ui| {
            ui.label("Lifetime sec");
            ui.add(DragValue::new(&mut gui.life_cycle.lifetime_sec).speed(0.1));
        });

        ui.horizontal(|ui| {
            ui.label("Force velocity > ");
            ui.label("x:");
            ui.add(DragValue::new(&mut gui.velocity.x).speed(0.1));
            ui.label("y:");
            ui.add(DragValue::new(&mut gui.velocity.y).speed(0.1));
            ui.label("z:");
            ui.add(DragValue::new(&mut gui.velocity.z).speed(0.1));
        });

        ui.horizontal(|ui| {
            ui.label("Mass applied per (1) unit length");
            ui.add(DragValue::new(&mut gui.mass_per_unit).speed(0.1));
        });

        ui.checkbox(&mut self.enabled, "Enabled");

        if self.uniform != gui {
            self.update_uniform = true;
            self.uniform = gui;
        }
    }
}

impl ForceAnimation {
    fn new(uniform: ForceUniform, emitter: &EmitterState, gfx_state: &GfxState) -> Self {
        let device = &gfx_state.device;
        let shader = gfx_state.create_shader_builtin(ShaderOptions {
            if_directives: &[],
            files: &["force_anim.wgsl"],
            label: "Force animation",
        });

        let buffer_content = uniform.create_buffer_content();

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Force buffer"),
            contents: bytemuck::cast_slice(&buffer_content),
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
                        min_binding_size: wgpu::BufferSize::new(buffer_content.len() as u64 * 4),
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
            label: Some("Force animation bind group"),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Force animation layout"),
            bind_group_layouts: &[&emitter.bg_layout, &animation_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Force animation pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "main",
        });

        Self {
            pipeline,
            uniform,
            bind_group,
            buffer,
            update_uniform: false,
            should_animate: false,
            selected_action: ListAction::None,
            enabled: true,
        }
    }
}
