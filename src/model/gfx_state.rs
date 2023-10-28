use egui_wgpu::renderer::ScreenDescriptor;
use egui_wgpu::wgpu;
use egui_wgpu::wgpu::CommandEncoder;
use egui_wgpu::Renderer;
use egui_winit::egui::ClippedPrimitive;
use egui_winit::egui::Context;
use egui_winit::egui::FontData;
use egui_winit::egui::FontDefinitions;
use egui_winit::egui::FontFamily;
use egui_winit::winit;
use egui_winit::winit::event::WindowEvent;
use egui_winit::EventResponse;
use wgpu_profiler::CreationError;
use wgpu_profiler::GpuProfiler;
use wgpu_profiler::GpuProfilerSettings;
use wgpu_profiler::GpuTimerScopeResult;
use winit::dpi::PhysicalSize;
use winit::window;

use crate::fx::PostProcessState;

use super::state::State;
use super::EmitterState;
use super::GuiState;

pub struct GfxState {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub ctx: Context,
    pub window: window::Window,
    pub renderer: Renderer,
    pub screen_descriptor: ScreenDescriptor,
    pub profiler: GpuProfiler,
    winit: egui_winit::State,
    surface: wgpu::Surface,
}

impl GfxState {
    pub async fn new(window: window::Window) -> Self {
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

        // Higher limits for Post FX
        let limits = wgpu::Limits {
            max_sampled_textures_per_shader_stage: 64,
            max_storage_textures_per_shader_stage: 64,
            ..Default::default()
        };

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::TIMESTAMP_QUERY
                        | wgpu::Features::TEXTURE_BINDING_ARRAY
                        | wgpu::Features::STORAGE_RESOURCE_BINDING_ARRAY
                        | GpuProfiler::ALL_WGPU_TIMER_FEATURES,
                    limits,
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
        let renderer = Renderer::new(&device, surface_format, None, 1);
        let ctx = Context::default();

        let mut fonts = FontDefinitions::default();

        fonts.font_data.insert(
            "FiraMono-Medium".to_string(),
            FontData::from_static(include_bytes!("../assets/fonts/FiraMono-Medium.ttf")),
        );

        fonts.families.insert(
            FontFamily::Proportional,
            vec!["FiraMono-Medium".to_string()],
        );

        ctx.set_fonts(fonts);

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [surface_config.width, surface_config.height],
            pixels_per_point: window.scale_factor() as f32,
        };

        let profiler = GpuProfiler::new_with_tracy_client(
            GpuProfilerSettings::default(),
            adapter.get_info().backend,
            &device,
            &queue,
        )
        .unwrap_or_else(|err| match err {
            CreationError::TracyClientNotRunning
            | CreationError::TracyGpuContextCreationError(_) => {
                println!("Failed to connect to Tracy. Continuing without Tracy integration.");
                GpuProfiler::new(GpuProfilerSettings::default()).expect("Failed to create profiler")
            }
            _ => {
                panic!("Failed to create profiler: {}", err);
            }
        });

        Self {
            surface,
            window,
            device,
            surface_config,
            renderer,
            queue,
            winit,
            ctx,
            screen_descriptor,
            profiler,
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

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width > 0 && size.height > 0 {
            self.surface_config.width = size.width;
            self.surface_config.height = size.height;
            self.surface.configure(&self.device, &self.surface_config);

            self.screen_descriptor = ScreenDescriptor {
                size_in_pixels: [size.width, size.height],
                pixels_per_point: self.window.scale_factor() as f32,
            };
        }
    }

    pub fn draw_gui(state: &mut State, encoder: &mut CommandEncoder) -> Vec<ClippedPrimitive> {
        let input = state
            .gfx_state
            .winit
            .take_egui_input(&state.gfx_state.window);

        state.gfx_state.ctx.begin_frame(input);

        GuiState::update_gui(state);

        let State { gfx_state, .. } = state;

        let full_output = gfx_state.ctx.end_frame();

        let clipped_primitives = gfx_state.ctx.tessellate(full_output.shapes);

        gfx_state.winit.handle_platform_output(
            &gfx_state.window,
            &gfx_state.ctx,
            full_output.platform_output,
        );

        for (tex_id, img_delta) in full_output.textures_delta.set {
            gfx_state.renderer.update_texture(
                &gfx_state.device,
                &gfx_state.queue,
                tex_id,
                &img_delta,
            );
        }

        for tex_id in full_output.textures_delta.free {
            gfx_state.renderer.free_texture(&tex_id);
        }

        gfx_state.renderer.update_buffers(
            &gfx_state.device,
            &gfx_state.queue,
            encoder,
            &clipped_primitives,
            &gfx_state.screen_descriptor,
        );

        clipped_primitives
    }

    pub fn render(state: &mut State) {
        let output_frame = match state.gfx_state.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Outdated) => {
                return;
            }
            Err(e) => {
                eprintln!("Dropped frame with error: {}", e);
                return;
            }
        };

        let mut encoder =
            state
                .gfx_state
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("encoder"),
                });

        let output_view = output_frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Computing particles
        EmitterState::compute_particles(state, &mut encoder);

        // Rendering particles
        EmitterState::render_particles(state, &mut encoder);

        let ping_pong_idx = PostProcessState::compute(state, &mut encoder);

        // Post processing render
        PostProcessState::render(state, output_view, &mut encoder, ping_pong_idx);

        let GfxState {
            queue, profiler, ..
        } = &mut state.gfx_state;

        profiler.resolve_queries(&mut encoder);

        // Submit the commands.
        queue.submit(Some(encoder.finish()));

        // Redraw egui
        output_frame.present();

        // Signal to the profiler that the frame is finished.
        profiler.end_frame().unwrap();
    }
}
