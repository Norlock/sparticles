use egui_wgpu::wgpu;
use egui_winit::egui::{DragValue, Ui};
use glam::Vec4;
use serde::{Deserialize, Serialize};
use wgpu::util::DeviceExt;

use crate::{
    animations::ItemAction,
    math::SparVec4,
    model::{Clock, EmitterState, GfxState, GuiState},
    traits::*,
    util::persistence::ExportAnimation,
};

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ColorUniform {
    pub from_color: SparVec4,
    pub to_color: SparVec4,
    pub from_sec: f32,
    pub until_sec: f32,
}

impl Default for ColorUniform {
    fn default() -> Self {
        Self {
            from_sec: 0.,
            until_sec: 0.5,
            from_color: Vec4::from_rgb(0, 255, 0).into(),
            to_color: Vec4::from_rgb(0, 0, 255).into(),
        }
    }
}

impl ColorUniform {
    fn create_buffer_content(&self) -> Vec<f32> {
        vec![
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
        ]
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
    fn tag(&self) -> &str {
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
    animation_bind_group: wgpu::BindGroup,
    uniform: ColorUniform,
    buffer: wgpu::Buffer,
    update_uniform: bool,
    selected_action: ItemAction,
}

impl ParticleAnimation for ColorAnimation {
    fn export(&self) -> ExportAnimation {
        let animation = serde_json::to_value(self.uniform).unwrap();
        let animation_type = RegisterColorAnimation.tag().to_owned();

        ExportAnimation {
            animation_tag: animation_type,
            animation,
        }
    }

    fn update(&mut self, _clock: &Clock, gfx_state: &GfxState) {
        if self.update_uniform {
            let buf_content_raw = self.uniform.create_buffer_content();
            let buf_content = bytemuck::cast_slice(&buf_content_raw);
            gfx_state.queue.write_buffer(&self.buffer, 0, buf_content);
            self.update_uniform = false;
        }
    }

    fn compute<'a>(
        &'a self,
        spawner: &'a EmitterState,
        clock: &Clock,
        compute_pass: &mut wgpu::ComputePass<'a>,
    ) {
        let nr = clock.get_bindgroup_nr();

        compute_pass.set_pipeline(&self.pipeline);
        compute_pass.set_bind_group(0, &spawner.bind_groups[nr], &[]);
        compute_pass.set_bind_group(1, &self.animation_bind_group, &[]);
        compute_pass.dispatch_workgroups(spawner.dispatch_x_count, 1, 1);
    }

    fn recreate(
        self: Box<Self>,
        gfx_state: &GfxState,
        spawner: &EmitterState,
    ) -> Box<dyn ParticleAnimation> {
        Box::new(Self::new(self.uniform, spawner, gfx_state))
    }

    fn selected_action(&mut self) -> &mut ItemAction {
        &mut self.selected_action
    }

    fn reset_action(&mut self) {
        self.selected_action = ItemAction::None;
    }

    fn create_ui(&mut self, ui: &mut Ui, gui_state: &GuiState) {
        gui_state.create_anim_header(ui, &mut self.selected_action, "Color animation");

        let mut gui = self.uniform;

        ui.horizontal(|ui| {
            ui.label("Animate from sec");
            ui.add(DragValue::new(&mut gui.from_sec).speed(0.1));
        });

        ui.horizontal(|ui| {
            ui.label("Animate until sec");
            ui.add(DragValue::new(&mut gui.until_sec).speed(0.1));
        });

        fn color_drag_value(val: &mut f32) -> DragValue<'_> {
            DragValue::new(val).clamp_range(0f32..=1f32).speed(0.01)
        }

        ui.horizontal(|ui| {
            ui.label("From color >");
            ui.label("r:");
            ui.add(color_drag_value(&mut gui.from_color.x));
            ui.label("g:");
            ui.add(color_drag_value(&mut gui.from_color.y));
            ui.label("b:");
            ui.add(color_drag_value(&mut gui.from_color.z));
            ui.label("a:");
            ui.add(color_drag_value(&mut gui.from_color.w));
        });

        ui.horizontal(|ui| {
            ui.label("To color >");
            ui.label("r:");
            ui.add(color_drag_value(&mut gui.to_color.x));
            ui.label("g:");
            ui.add(color_drag_value(&mut gui.to_color.y));
            ui.label("b:");
            ui.add(color_drag_value(&mut gui.to_color.z));
            ui.label("a:");
            ui.add(color_drag_value(&mut gui.to_color.w));
        });

        if self.uniform != gui {
            self.update_uniform = true;
            self.uniform = gui;
        }
    }
}

impl ColorAnimation {
    fn new(uniform: ColorUniform, spawner: &EmitterState, gfx_state: &GfxState) -> Self {
        let device = &gfx_state.device;
        let shader = device.create_shader("color_anim.wgsl", "Color animation");

        let animation_uniform = uniform.create_buffer_content();

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Color buffer"),
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

        let animation_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &animation_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("Color animation"),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute layout"),
            bind_group_layouts: &[&spawner.bind_group_layout, &animation_layout],
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
            animation_bind_group,
            buffer,
            uniform,
            update_uniform: false,
            selected_action: ItemAction::None,
        }
    }
}
