use egui_wgpu_backend::wgpu;
use model::app_state;
use model::gfx_state;
use std::sync::Mutex;
use winit::event::Event::*;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoopProxy;
use winit::window;

pub mod model;
pub mod shaders;
pub mod texture;

/// A custom event type for the winit app.
pub enum CustomEvent {
    RequestRedraw,
}

/// This is the repaint signal type that egui needs for requesting a repaint from another thread.
/// It sends the custom RequestRedraw event to the winit event loop.
struct ExampleRepaintSignal(Mutex<EventLoopProxy<CustomEvent>>);

impl epi::backend::RepaintSignal for ExampleRepaintSignal {
    fn request_repaint(&self) {
        self.0
            .lock()
            .unwrap()
            .send_event(CustomEvent::RequestRedraw)
            .ok();
    }
}

fn main() {
    env_logger::init();

    let event_loop = winit::event_loop::EventLoopBuilder::<CustomEvent>::with_user_event().build();

    let window = window::WindowBuilder::new()
        .with_decorations(true)
        .with_transparent(false)
        .with_resizable(true)
        .with_title("Sparticles")
        .build(&event_loop)
        .unwrap();

    let mut gfx_state = pollster::block_on(gfx_state::GfxState::new(window));
    let mut app_state = app_state::AppState::new(&gfx_state);

    event_loop.run(move |event, _, control_flow| {
        // Pass the winit events to the platform integration.
        gfx_state.handle_event(&event);

        match event {
            RedrawRequested(window_id) => {
                if window_id == gfx_state.window_id() {
                    app_state.update(&gfx_state);
                    gfx_state.update(&app_state);
                    gfx_state.render(&app_state);
                }
            }
            MainEventsCleared | UserEvent(CustomEvent::RequestRedraw) => {
                gfx_state.request_redraw();
            }
            WindowEvent { event, window_id } => match event {
                winit::event::WindowEvent::Resized(size) => {
                    if window_id == gfx_state.window_id() {
                        gfx_state.window_resize(size);
                        app_state.window_resize(&gfx_state);
                    }
                }
                winit::event::WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    if window_id == gfx_state.window_id() {
                        gfx_state.window_resize(*new_inner_size);
                        app_state.window_resize(&gfx_state);
                    }
                }
                winit::event::WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            },
            _ => (),
        }
    });
}
