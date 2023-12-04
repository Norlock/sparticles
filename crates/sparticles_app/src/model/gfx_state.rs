use super::state::SparState;
use super::EmitterState;
use super::SparEvents;
use crate::fx::PostProcessState;
use crate::init::AppVisitor;
use egui_wgpu::renderer::ScreenDescriptor;
use egui_wgpu::wgpu;
use egui_wgpu::wgpu::CommandEncoder;
use egui_wgpu::Renderer;
use egui_winit::egui::ClippedPrimitive;
use egui_winit::egui::Context;
use egui_winit::egui::FontData;
use egui_winit::egui::FontDefinitions;
use egui_winit::egui::FontFamily;
use egui_winit::egui::RawInput;
use egui_winit::winit;
use egui_winit::winit::event::WindowEvent;
use egui_winit::EventResponse;
use wgpu_profiler::GpuProfiler;
use wgpu_profiler::GpuProfilerSettings;
use wgpu_profiler::ProfilerCommandRecorder;
use winit::dpi::PhysicalSize;
use winit::window;

pub struct GfxState {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub ctx: Context,
    pub window: window::Window,
    pub renderer: Renderer,
    pub screen_descriptor: ScreenDescriptor,
    pub profiler: GpuProfiler,
    pub winit: egui_winit::State,
    pub surface: wgpu::Surface,
}

pub struct DrawGuiResult {
    pub primitives: Vec<ClippedPrimitive>,
    pub events: SparEvents,
}

impl GfxState {
    pub fn begin_scope(&mut self, label: &str, pass: &mut impl ProfilerCommandRecorder) {
        self.profiler.begin_scope(label, pass, &self.device);
    }

    pub fn end_scope(&mut self, pass: &mut impl ProfilerCommandRecorder) {
        self.profiler.end_scope(pass).unwrap();
    }

    pub fn finish_frame(
        &mut self,
        mut encoder: CommandEncoder,
        output_frame: wgpu::SurfaceTexture,
    ) {
        self.profiler.resolve_queries(&mut encoder);

        // Submit the commands.
        self.queue.submit(Some(encoder.finish()));

        // Redraw egui
        output_frame.present();

        // Signal to the profiler that the frame is finished.
        self.profiler.end_frame().unwrap();
    }

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
            max_sampled_textures_per_shader_stage: 32,
            max_storage_textures_per_shader_stage: 32,
            ..Default::default()
        };

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::TEXTURE_BINDING_ARRAY
                        | wgpu::Features::STORAGE_RESOURCE_BINDING_ARRAY
                        | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
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

        let raw_input = RawInput::default();
        let vp = raw_input.viewport();

        let ctx = Context::default();

        let winit = egui_winit::State::new(
            raw_input.viewport_id,
            &window,
            vp.native_pixels_per_point,
            raw_input.max_texture_side,
        );

        let renderer = Renderer::new(&device, surface_format, None, 1);

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

        let profiler =
            GpuProfiler::new(GpuProfilerSettings::default()).expect("Failed to create profiler");

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
        self.winit.on_window_event(&self.ctx, event)
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

    pub fn draw_ui(
        state: &mut SparState,
        encoder: &mut wgpu::CommandEncoder,
        app_visitor: &mut impl AppVisitor,
    ) -> DrawGuiResult {
        let input = state.gfx.winit.take_egui_input(&state.gfx.window);

        state.gfx.ctx.begin_frame(input);

        let events = app_visitor.draw_ui(state, encoder);
        let gfx = &mut state.gfx;

        let full_output = gfx.ctx.end_frame();

        let primitives = gfx
            .ctx
            .tessellate(full_output.shapes, gfx.winit.pixels_per_point());

        gfx.winit
            .handle_platform_output(&gfx.window, &gfx.ctx, full_output.platform_output);

        for (tex_id, img_delta) in full_output.textures_delta.set {
            gfx.renderer
                .update_texture(&gfx.device, &gfx.queue, tex_id, &img_delta);
        }

        for tex_id in full_output.textures_delta.free {
            gfx.renderer.free_texture(&tex_id);
        }

        gfx.renderer.update_buffers(
            &gfx.device,
            &gfx.queue,
            encoder,
            &primitives,
            &gfx.screen_descriptor,
        );

        DrawGuiResult { events, primitives }
    }

    pub fn render(state: &mut SparState, app_visitor: &mut impl AppVisitor) -> SparEvents {
        let output_frame = match state.gfx.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Outdated) => {
                return SparEvents::default();
            }
            Err(e) => {
                eprintln!("Dropped frame with error: {}", e);
                return SparEvents::default();
            }
        };

        let mut encoder =
            state
                .gfx
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("encoder"),
                });

        let output_view = output_frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        if state.play {
            EmitterState::compute_particles(state, &mut encoder);
        }

        EmitterState::render_particles(state, &mut encoder);
        PostProcessState::compute(state, &mut encoder);
        let events = PostProcessState::render(state, output_view, &mut encoder, app_visitor);

        state.gfx.finish_frame(encoder, output_frame);

        events
    }
}
