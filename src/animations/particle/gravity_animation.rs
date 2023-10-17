use crate::math::SparVec3;
use crate::model::clock::Clock;
use crate::model::{EmitterState, GfxState, GuiState, LifeCycle};
use crate::traits::*;
use crate::util::persistence::DynamicExport;
use crate::util::ItemAction;
use egui_wgpu::wgpu;
use egui_winit::egui::{DragValue, Ui};
use glam::Vec3;
use serde::{Deserialize, Serialize};
use wgpu::util::DeviceExt;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GravityUniform {
    life_cycle: LifeCycle,
    gravitational_force: f32,
    dead_zone: f32,
    mass: f32,
    should_animate: bool,
    start_pos: SparVec3,
    end_pos: SparVec3,
    current_pos: SparVec3,
}

impl Default for GravityUniform {
    fn default() -> Self {
        Self {
            life_cycle: LifeCycle {
                from_sec: 0.,
                until_sec: 6.,
                lifetime_sec: 12.,
            },
            gravitational_force: 0.01,
            dead_zone: 4.,
            mass: 1_000_000.,
            start_pos: [-25., 8., 0.].into(),
            current_pos: [-25., 8., 0.].into(),
            end_pos: [25., 8., 0.].into(),
            should_animate: false,
        }
    }
}

pub struct GravityUniformOptions {
    /// In newton
    pub gravitational_force: f32,
    /// Use to exclude extreme gravitational pulls, e.g. 20.
    pub dead_zone: f32,
    pub mass: f32,
    pub life_cycle: LifeCycle,
    pub start_pos: Vec3,
    pub end_pos: Vec3,
}

impl GravityUniform {
    pub fn new(props: GravityUniformOptions) -> Self {
        Self {
            gravitational_force: props.gravitational_force,
            dead_zone: props.dead_zone,
            mass: props.mass,
            life_cycle: props.life_cycle,
            start_pos: props.start_pos.into(),
            end_pos: props.end_pos.into(),
            current_pos: props.start_pos.into(),
            should_animate: false,
        }
    }

    fn create_buffer_content(&self) -> [f32; 6] {
        [
            self.gravitational_force,
            self.dead_zone,
            self.mass,
            self.current_pos.x,
            self.current_pos.y,
            self.current_pos.z,
        ]
    }
}

#[derive(Clone, Copy)]
pub struct RegisterGravityAnimation;

impl RegisterGravityAnimation {
    /// Will append animation to emitter
    pub fn append(uniform: GravityUniform, emitter: &mut EmitterState, gfx_state: &GfxState) {
        let anim = Box::new(GravityAnimation::new(uniform, emitter, gfx_state));

        emitter.push_particle_animation(anim);
    }
}

impl RegisterParticleAnimation for RegisterGravityAnimation {
    fn create_default(
        &self,
        gfx_state: &GfxState,
        emitter: &EmitterState,
    ) -> Box<dyn ParticleAnimation> {
        Box::new(GravityAnimation::new(
            GravityUniform::default(),
            emitter,
            gfx_state,
        ))
    }

    fn tag(&self) -> &str {
        "gravity"
    }

    fn import(
        &self,
        gfx_state: &GfxState,
        emitter: &EmitterState,
        value: serde_json::Value,
    ) -> Box<dyn ParticleAnimation> {
        let uniform = serde_json::from_value(value).unwrap();
        Box::new(GravityAnimation::new(uniform, emitter, gfx_state))
    }
}

pub struct GravityAnimation {
    pipeline: wgpu::ComputePipeline,
    uniform: GravityUniform,
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    selected_action: ItemAction,
}

impl HandleAction for GravityAnimation {
    fn reset_action(&mut self) {
        self.selected_action = ItemAction::None;
    }

    fn selected_action(&mut self) -> &mut ItemAction {
        &mut self.selected_action
    }

    fn export(&self) -> DynamicExport {
        let animation = serde_json::to_value(self.uniform).unwrap();

        DynamicExport {
            tag: RegisterGravityAnimation.tag().to_owned(),
            data: animation,
        }
    }

    fn enabled(&self) -> bool {
        todo!()
    }
}

impl ParticleAnimation for GravityAnimation {
    fn compute<'a>(
        &'a self,
        spawner: &'a EmitterState,
        clock: &Clock,
        compute_pass: &mut wgpu::ComputePass<'a>,
    ) {
        if !self.uniform.should_animate {
            return;
        }

        let nr = clock.get_bindgroup_nr();

        compute_pass.set_pipeline(&self.pipeline);
        compute_pass.set_bind_group(0, &spawner.bind_groups[nr], &[]);
        compute_pass.set_bind_group(1, &self.bind_group, &[]);
        compute_pass.dispatch_workgroups(spawner.dispatch_x_count, 1, 1);
    }

    fn update(&mut self, clock: &Clock, gfx_state: &GfxState) {
        let queue = &gfx_state.queue;
        let uniform = &mut self.uniform;
        let life_cycle = &mut uniform.life_cycle;
        let current_sec = life_cycle.get_current_sec(clock);

        uniform.should_animate = life_cycle.shoud_animate(current_sec);

        if uniform.should_animate {
            let fraction = life_cycle.get_fraction(current_sec);
            *uniform.current_pos = uniform.start_pos.lerp(*uniform.end_pos, fraction);
            let buffer_content = uniform.create_buffer_content();

            queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&buffer_content));
        }
    }

    fn recreate(
        self: Box<Self>,
        gfx_state: &GfxState,
        spawner: &EmitterState,
    ) -> Box<dyn ParticleAnimation> {
        Box::new(Self::new(self.uniform, spawner, gfx_state))
    }

    fn create_ui(&mut self, ui: &mut Ui, ui_state: &GuiState) {
        self.selected_action = ui_state.create_anim_header(ui, "Gravity animation");
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
            ui.label("Start position > ");
            ui.label("x:");
            ui.add(DragValue::new(&mut gui.start_pos.x).speed(0.1));
            ui.label("y:");
            ui.add(DragValue::new(&mut gui.start_pos.y).speed(0.1));
            ui.label("z:");
            ui.add(DragValue::new(&mut gui.start_pos.z).speed(0.1));
        });

        ui.horizontal(|ui| {
            ui.label("End position > ");
            ui.label("x:");
            ui.add(DragValue::new(&mut gui.end_pos.x).speed(0.1));
            ui.label("y:");
            ui.add(DragValue::new(&mut gui.end_pos.y).speed(0.1));
            ui.label("z:");
            ui.add(DragValue::new(&mut gui.end_pos.z).speed(0.1));
        });

        ui.horizontal(|ui| {
            ui.label("Dead zone");
            ui.add(DragValue::new(&mut gui.dead_zone).speed(0.1));
        });

        ui.horizontal(|ui| {
            ui.label("Gravitational force");
            ui.add(
                DragValue::new(&mut gui.gravitational_force)
                    .speed(0.001)
                    .clamp_range(-0.02..=0.02),
            );
        });

        if self.uniform != gui {
            self.uniform = gui;
        }
    }
}

impl GravityAnimation {
    fn new(uniform: GravityUniform, emitter: &EmitterState, gfx_state: &GfxState) -> Self {
        let device = &gfx_state.device;
        let shader = device.create_shader("gravity_anim.wgsl", "Gravity animation");

        let buffer_content = uniform.create_buffer_content();

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Gravitational buffer"),
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
            label: Some("Gravity animation bind group"),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Gravity animation layout"),
            bind_group_layouts: &[&emitter.bind_group_layout, &animation_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Gravity animation pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "main",
        });

        Self {
            pipeline,
            uniform,
            buffer,
            bind_group,
            selected_action: ItemAction::None,
        }
    }
}
