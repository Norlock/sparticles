use model::GfxState;
use std::sync::Mutex;
use traits::CreateAnimation;
use winit::event::Event::*;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoopProxy;
use winit::window;
use winit::window::WindowId;

pub mod animations;
pub mod debug;
pub mod model;
pub mod shaders;
pub mod texture;
pub mod traits;

pub struct InitialiseApp {
    pub show_gui: bool,
    pub particle_animations: Vec<Box<dyn CreateAnimation>>,
}

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

pub fn start(init_app: InitialiseApp) {
    env_logger::init();

    let event_loop = winit::event_loop::EventLoopBuilder::<CustomEvent>::with_user_event().build();

    let window = window::WindowBuilder::new()
        .with_decorations(true)
        .with_transparent(false)
        .with_resizable(true)
        .with_title("Sparticles")
        .build(&event_loop)
        .unwrap();

    let mut gfx_state = pollster::block_on(GfxState::new(window, &init_app));
    let mut app_state = gfx_state.create_app_state(init_app);

    event_loop.run(move |event, _, control_flow| {
        // Pass the winit events to the platform integration.
        gfx_state.handle_event(&event);
        let do_exec = |window_id: WindowId| window_id == gfx_state.window_id();

        match event {
            RedrawRequested(window_id) if do_exec(window_id) => {
                app_state.update(&gfx_state);
                gfx_state.render(&mut app_state);
            }
            MainEventsCleared | UserEvent(CustomEvent::RequestRedraw) => {
                gfx_state.request_redraw();
            }
            WindowEvent { event, window_id } if do_exec(window_id) => match event {
                winit::event::WindowEvent::Resized(size) => {
                    gfx_state.window_resize(size);
                    app_state.window_resize(&gfx_state);
                }
                winit::event::WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    gfx_state.window_resize(*new_inner_size);
                    app_state.window_resize(&gfx_state);
                }
                winit::event::WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                winit::event::WindowEvent::KeyboardInput { input, .. } => {
                    app_state.process_events(input);
                }
                _ => {}
            },
            _ => (),
        }
    });
}
