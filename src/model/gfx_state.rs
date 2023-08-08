use std::iter;

use crate::texture::DepthTexture;

use egui_wgpu::renderer::ScreenDescriptor;
use egui_wgpu::wgpu;
use egui_wgpu::Renderer;
use egui_winit::egui::Context;
use egui_winit::egui::FontDefinitions;
use egui_winit::egui::Style;
use egui_winit::winit;
use egui_winit::winit::event::WindowEvent;
use egui_winit::EventResponse;
use winit::dpi::PhysicalSize;
use winit::window;

use super::app_state::AppState;

use super::GuiState;

pub struct GfxState {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub window: window::Window,
    winit: egui_winit::State,
    surface: wgpu::Surface,
    renderer: Renderer,
    ctx: Context,
}

impl GfxState {
    pub async fn new<'a>(window: window::Window) -> Self {
        let instance = wgpu::Instance::default();

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
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &surface_config);

        let winit = egui_winit::State::new(&window);
        let renderer = Renderer::new(&device, surface_format, Some(DepthTexture::DEPTH_FORMAT), 1);
        let ctx = Context::default();
        ctx.set_fonts(FontDefinitions::default());
        ctx.set_style(Style::default());

        Self {
            surface,
            window,
            device,
            surface_config,
            renderer,
            queue,
            winit,
            ctx,
        }
    }

    pub fn window_id(&self) -> window::WindowId {
        self.window.id()
    }

    pub fn handle_event(&mut self, event: &WindowEvent<'_>) -> EventResponse {
        self.winit.on_event(&self.ctx, event)
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

    pub fn render(&mut self, app_state: &AppState, gui_state: &mut GuiState) {
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

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("encoder"),
            });

        let output_view = output_frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let input = self.winit.take_egui_input(&self.window);

        let full_output = self.ctx.run(input, |ui| {
            gui_state.update(app_state, ui);
        });

        let paint_jobs = self.ctx.tessellate(full_output.shapes);

        self.winit
            .handle_platform_output(&self.window, &self.ctx, full_output.platform_output);

        // Upload all resources for the GPU.
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [self.surface_config.width, self.surface_config.height],
            pixels_per_point: self.window.scale_factor() as f32,
        };

        self.renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            &paint_jobs,
            &screen_descriptor,
        );

        for (tex_id, img_delta) in full_output.textures_delta.set {
            self.renderer
                .update_texture(&self.device, &self.queue, tex_id, &img_delta);
        }

        for tex_id in full_output.textures_delta.free {
            self.renderer.free_texture(&tex_id);
        }

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("transform pipeline"),
            });

            app_state.compute(&mut compute_pass);
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &app_state.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            app_state.render(&mut render_pass);

            self.renderer
                .render(&mut render_pass, &paint_jobs, &screen_descriptor);
        }

        // Submit the commands.
        self.queue.submit(iter::once(encoder.finish()));

        // Redraw egui
        output_frame.present();
    }
}

pub trait StateInitializer {
    //fn create_emitter(&self) -> Emitter;
    fn show_ui(&self) -> bool;
}
