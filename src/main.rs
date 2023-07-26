use ::egui::FontDefinitions;
use egui_wgpu_backend::wgpu;
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use model::{gui_state, Clock};
use model::GuiState;
use std::iter;
use std::sync::Mutex;
use std::time::Instant;
use winit::event::Event::*;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoopProxy;
const INITIAL_WIDTH: u32 = 1920;
const INITIAL_HEIGHT: u32 = 1080;

pub mod model;
pub mod shaders;

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

/// A simple egui + wgpu + winit based example.
fn main() {
    let event_loop = winit::event_loop::EventLoopBuilder::<CustomEvent>::with_user_event().build();
    //let window = winit::window::WindowBuilder::new()
    //.with_decorations(true)
    //.with_resizable(true)
    //.with_transparent(false)
    //.with_title("Sparticles")
    //.with_inner_size(winit::dpi::PhysicalSize {
    //width: INITIAL_WIDTH,
    //height: INITIAL_HEIGHT,
    //})
    //.build(&event_loop)
    //.unwrap();

    let instance = wgpu::Instance::default();

    let mut gui_state = GuiState::new(gui_state::Properties {
        width: 500,
        height: 500,
        instance: &instance,
        event_loop: &event_loop,
    });

    let clock = Clock::new();

    event_loop.run(move |event, _, control_flow| {
        // Pass the winit events to the platform integration.
        gui_state.handle_event(&event);

        match event {
            RedrawRequested(..) => {
                gui_state.update(&clock);
                gui_state.render();
            }
            MainEventsCleared | UserEvent(CustomEvent::RequestRedraw) => {
                gui_state.request_redraw();
            }
            WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::Resized(size) => {
                    gui_state.window_resize(size);
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

pub fn seconds_since_midnight() -> f64 {
    0.4f64
}
