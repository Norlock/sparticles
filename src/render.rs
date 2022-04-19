use crate::{camera::Camera, instance::particle::Particle};
use winit::{event_loop::EventLoop, window::WindowBuilder};

pub fn create_window(event_loop: &EventLoop<()>) -> winit::window::Window {
    WindowBuilder::new()
        .with_title("Sparticles")
        .build(&event_loop)
        .unwrap()
}

pub fn create_pipeline_layout(device: &wgpu::Device, camera: &Camera) -> wgpu::PipelineLayout {
    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[&camera.bind_group_layout],
        push_constant_ranges: &[],
    })
}

pub struct PipelineProperties<'a> {
    pub config: &'a wgpu::SurfaceConfiguration,
    pub device: &'a wgpu::Device,
    pub render_pipeline_layout: &'a wgpu::PipelineLayout,
}

pub fn create_pipeline(pipeline_properties: PipelineProperties) -> wgpu::RenderPipeline {
    let PipelineProperties {
        config,
        device,
        render_pipeline_layout,
    } = pipeline_properties;

    let draw_shader = device.create_shader_module(&wgpu::include_wgsl!("./instance/draw.wgsl"));

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &draw_shader,
            entry_point: "vs_main",
            buffers: &[Particle::descriptor()],
        },
        fragment: Some(wgpu::FragmentState {
            module: &draw_shader,
            entry_point: "fs_main",
            targets: &[wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            }],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleStrip,
            strip_index_format: Some(wgpu::IndexFormat::Uint32),
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Front),
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    })
}
