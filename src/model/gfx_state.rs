use std::iter;

use crate::CustomEvent;
use egui::FontDefinitions;
use egui_wgpu_backend::{wgpu, RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use winit::dpi::PhysicalSize;
use winit::event::Event;
use winit::event_loop::EventLoop;
use winit::window;

use super::app_state;

pub struct Options<'a> {
    pub instance: &'a wgpu::Instance,
    pub event_loop: &'a EventLoop<CustomEvent>,
}

pub struct GfxState {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface_config: wgpu::SurfaceConfiguration,
    platform: Platform,
    window: window::Window,
    surface: wgpu::Surface,
    render_pass: RenderPass,
}

impl GfxState {
    pub async fn new<'a>(options: Options<'a>) -> Self {
        let Options {
            instance,
            event_loop,
        } = options;

        let window = window::WindowBuilder::new()
            .with_decorations(true)
            .with_resizable(true)
            .with_transparent(false)
            .build(&event_loop)
            .unwrap();

        let surface = unsafe {
            instance
                .create_surface(&window)
                .expect("Can't load surface")
        };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::default(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let size = window.inner_size();
        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &surface_config);

        let platform = Platform::new(PlatformDescriptor {
            physical_width: size.width as u32,
            physical_height: size.height as u32,
            scale_factor: window.scale_factor(),
            font_definitions: FontDefinitions::default(),
            style: Default::default(),
        });

        let render_pass = RenderPass::new(&device, surface_format, 1);

        Self {
            platform,
            surface,
            window,
            device,
            surface_config,
            render_pass,
            queue,
        }
    }

    pub fn window_id(&self) -> window::WindowId {
        self.window.id()
    }

    pub fn handle_event(&mut self, event: &Event<CustomEvent>) {
        self.platform.handle_event(event);
    }

    // TODO naar kijken of dit misschien gelijk in render kan
    pub fn update(&mut self, app_state: &app_state::AppState) {
        let clock = app_state.clock;
        self.platform.update_time(clock.elapsed_sec_f64());
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    pub fn window_resize(&mut self, size: PhysicalSize<u32>) {
        if size.width > 0 && size.height > 0 {
            self.surface_config.width = size.width;
            self.surface_config.height = size.height;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }

    pub fn render(&mut self, app_state: &app_state::AppState) {
        let output_frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Outdated) => {
                return;
            }
            Err(e) => {
                eprintln!("Dropped frame with error: {}", e);
                return;
            }
        };

        let output_view = output_frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Begin to draw the UI frame.
        self.platform.begin_frame();

        // End the UI frame. We could now handle the output and draw the UI with the backend.
        let full_output = self.platform.end_frame(Some(&self.window));

        let ctx = &self.platform.context();
        let paint_jobs = ctx.tessellate(full_output.shapes);

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("encoder"),
            });

        // Upload all resources for the GPU.
        let screen_descriptor = ScreenDescriptor {
            physical_width: self.surface_config.width,
            physical_height: self.surface_config.height,
            scale_factor: self.window.scale_factor() as f32,
        };

        let tex_delta: egui::TexturesDelta = full_output.textures_delta;

        self.render_pass
            .add_textures(&self.device, &self.queue, &tex_delta)
            .expect("add texture ok");

        self.render_pass
            .update_buffers(&self.device, &self.queue, &paint_jobs, &screen_descriptor);

        // Draw the demo application.
        egui::Window::new("My Window").show(ctx, |ui| {
            ui.label("Hello World!");
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&app_state.render_pipeline);
            render_pass.set_bind_group(0, &app_state.diffuse_bind_group, &[]);
            render_pass.draw(0..4, 0..1);

            let result = self.render_pass.execute_with_renderpass(
                &mut render_pass,
                &paint_jobs,
                &screen_descriptor,
            );

            match result {
                Ok(..) => {}
                Err(err) => {
                    println!("{}", err);
                }
            }
        }

        // Submit the commands.
        self.queue.submit(iter::once(encoder.finish()));

        // Redraw egui
        output_frame.present();

        self.render_pass
            .remove_textures(tex_delta)
            .expect("remove texture ok");
    }
}

pub trait StateInitializer {
    //fn create_emitter(&self) -> Emitter;
    fn show_ui(&self) -> bool;
}
