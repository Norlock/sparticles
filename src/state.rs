use crate::camera::*;
use crate::examples::simple_emitter;
use crate::instance::emitter::Emitter;
use crate::instance::instance::Instance;
use crate::render::{create_pipeline, create_pipeline_layout, create_window, PipelineProperties};

use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::window::Window;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
};

pub struct State {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub render_pipeline: wgpu::RenderPipeline,
    pub mouse_pressed: bool,
    pub instances: Instance,
    pub camera: Camera,
}

const VERTICES_LEN: u32 = 4;

impl State {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        surface.configure(&device, &config);

        let camera = Camera::new(&device, &config);
        let instances = Instance::new(&device);

        let render_pipeline_layout = create_pipeline_layout(&device, &camera);

        let pipeline_properties = PipelineProperties {
            device: &device,
            render_pipeline_layout: &render_pipeline_layout,
            config: &config,
        };

        let render_pipeline = create_pipeline(pipeline_properties);

        Self {
            config,
            device,
            surface,
            size,
            queue,
            render_pipeline,
            camera,
            instances,
            mouse_pressed: false,
        }
    }

    pub fn add_emitter(&mut self, emitter: Emitter) {
        self.instances.emitters.push(emitter);
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.camera.resize(new_size);
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(key),
                        state,
                        ..
                    },
                ..
            } => self.camera.process_keyboard(&key, &state),
            WindowEvent::MouseWheel { delta, .. } => {
                self.camera.process_scroll(delta);
                true
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                self.mouse_pressed = *state == ElementState::Pressed;
                true
            }
            _ => false,
        }
    }

    pub fn update(&mut self) {
        let delta = self.instances.clock.delta();

        self.camera.update(delta, &self.queue);
        self.instances.update(&self.device);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        // Not clearing here in order to test wgpu's zero texture initialization on a surface texture.
                        // Users should avoid loading uninitialized memory since this can cause additional overhead.
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                    //ops: wgpu::Operations {
                    //load: wgpu::LoadOp::Clear(wgpu::Color {
                    //r: 0.0,
                    //g: 0.0,
                    //b: 0.0,
                    //a: 1.0,
                    //}),
                    //store: true,
                    //},
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_vertex_buffer(0, self.instances.buffer.slice(..));
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera.bind_group, &[]);

            let num_particles = self.instances.particles.len() as u32;
            render_pass.draw(0..VERTICES_LEN, 0..num_particles);
        }

        self.queue.submit(Some(encoder.finish()));
        output.present();

        Ok(())
    }
}

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new();

    let window = create_window(&event_loop);
    let mut state = State::new(&window).await;

    state.add_emitter(simple_emitter());

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion{ delta, },
                .. // We're not using device_id currently
            } if state.mouse_pressed => {
                state.camera.process_mouse(delta.0, delta.1)
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() && !state.input(event) => {
                match event {
                    #[cfg(not(target_arch="wasm32"))]
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                state.update();

                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                window.request_redraw();
            }
            _ => {}
        }
    });
}
