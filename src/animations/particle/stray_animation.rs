use crate::{
    model::{Clock, EmitterState, GfxState, GuiState},
    traits::{CalculateBufferSize, CreateAnimation, CustomShader, ParticleAnimation},
};
use egui_wgpu::wgpu;
use egui_winit::egui::{DragValue, Ui};
use wgpu::util::DeviceExt;

#[derive(Clone, Copy, PartialEq)]
pub struct StrayUniform {
    pub stray_radians: f32,
    pub from_sec: f32,
    pub until_sec: f32,
}

impl StrayUniform {
    fn create_buffer_content(&self) -> [f32; 4] {
        [self.stray_radians, self.from_sec, self.until_sec, 0.]
    }
}

impl CreateAnimation for StrayUniform {
    fn into_animation(
        self: Box<Self>,
        gfx_state: &GfxState,
        particle: &EmitterState,
    ) -> Box<dyn ParticleAnimation> {
        Box::new(StrayAnimation::new(*self, particle, &gfx_state.device))
    }
}

struct StrayAnimation {
    pipeline: wgpu::ComputePipeline,
    uniform: StrayUniform,
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    update_uniform: bool,
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
        spawner: &'a EmitterState,
        clock: &Clock,
        compute_pass: &mut wgpu::ComputePass<'a>,
    ) {
        let nr = clock.get_bindgroup_nr();

        compute_pass.set_pipeline(&self.pipeline);
        compute_pass.set_bind_group(0, &spawner.bind_groups[nr], &[]);
        compute_pass.set_bind_group(1, &self.bind_group, &[]);
        compute_pass.dispatch_workgroups(spawner.dispatch_x_count, 1, 1);
    }

    fn recreate(
        self: Box<Self>,
        gfx_state: &GfxState,
        spawner: &EmitterState,
    ) -> Box<dyn ParticleAnimation> {
        Box::new(Self::new(self.uniform, spawner, &gfx_state.device))
    }

    fn create_gui(&mut self, ui: &mut Ui) {
        GuiState::create_title(ui, "Stray animation");

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

        GuiState::create_degree_slider(ui, &mut stray_degrees, "Stray degrees");

        gui.stray_radians = stray_degrees.to_radians();

        if self.uniform != gui {
            self.update_uniform = true;
            self.uniform = gui;
        }
    }
}

impl StrayAnimation {
    fn new(uniform: StrayUniform, spawner: &EmitterState, device: &wgpu::Device) -> Self {
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
            bind_group_layouts: &[&spawner.bind_group_layout, &animation_layout],
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
        }
    }
}
