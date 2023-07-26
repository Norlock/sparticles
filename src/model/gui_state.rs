use std::iter;

use crate::model::Clock;
use crate::CustomEvent;
use egui::FontDefinitions;
use egui_wgpu_backend::{wgpu, RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use winit::dpi::{self, PhysicalSize};
use winit::event::Event;
use winit::event_loop::EventLoop;
use winit::window;

pub struct Properties<'a> {
    pub width: u32,
    pub height: u32,
    pub instance: &'a wgpu::Instance,
    pub event_loop: &'a EventLoop<CustomEvent>,
}

pub struct GuiState {
    pub platform: Platform,
    pub window: window::Window,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,
    render_pass: RenderPass,
}

impl GuiState {
    pub fn new(props: Properties) -> Self {
        let Properties {
            width,
            height,
            instance,
            event_loop,
        } = props;

        let window = window::WindowBuilder::new()
            .with_decorations(true)
            .with_resizable(true)
            .with_transparent(false)
            .with_title("Sparticles")
            .with_inner_size(dpi::PhysicalSize { width, height })
            .build(&event_loop)
            .unwrap();

        let surface = unsafe {
            instance
                .create_surface(&window)
                .expect("Can't load surface")
        };

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .unwrap();

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::default(),
                limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        ))
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

        // We use the egui_winit_platform crate as the platform.
        let platform = Platform::new(PlatformDescriptor {
            physical_width: size.width as u32,
            physical_height: size.height as u32,
            scale_factor: window.scale_factor(),
            font_definitions: FontDefinitions::default(),
            style: Default::default(),
        });

        // We use the egui_wgpu_backend crate as the render backend.
        let render_pass = RenderPass::new(&device, surface_format, 1);

        // Display the demo application that ships with egui.
        //let mut demo_app = egui_demo_lib::DemoWindows::default();

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

    pub fn handle_event(&mut self, event: &Event<CustomEvent>) {
        self.platform.handle_event(event);
    }

    pub fn update(&mut self, clock: &Clock) {
        self.platform.update_time(clock.elapsed_sec_f64());
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw(); // TODO maak functie in state
    }

    pub fn window_resize(&mut self, size: PhysicalSize<u32>) {
        // Resize with 0 width and height is used by winit to signal a minimize event on Windows.
        // See: https://github.com/rust-windowing/winit/issues/208
        // This solves an issue where the app would panic when minimizing on Windows.
        if size.width > 0 && size.height > 0 {
            self.surface_config.width = size.width;
            self.surface_config.height = size.height;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }

    pub fn render(&mut self) {
        let output_frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Outdated) => {
                // This error occurs when the app is minimized on Windows.
                // Silently return here to prevent spamming the console with:
                // "The underlying surface has changed, and therefore the swap chain must be updated"
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
        let paint_jobs = self.platform.context().tessellate(full_output.shapes);

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
        egui::CentralPanel::default().show(&self.platform.context(), |ui| {
            ui.heading("My egui Application");
        });

        // Record all render passes.
        self.render_pass
            .execute(
                &mut encoder,
                &output_view,
                &paint_jobs,
                &screen_descriptor,
                Some(wgpu::Color::BLACK),
            )
            .unwrap();

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
