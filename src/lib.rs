use egui_winit::winit;
use model::emitter::Emitter;
use model::GfxState;
use traits::CreateAnimation;
use winit::event::Event::*;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use winit::window;
use winit::window::WindowId;

pub mod animations;
pub mod debug;
pub mod math;
pub mod model;
pub mod shaders;
pub mod texture;
pub mod traits;

pub struct InitialiseApp {
    pub show_gui: bool,
    pub particle_animations: Vec<Box<dyn CreateAnimation>>,
    pub emitter: Emitter,
}

pub fn start(init_app: InitialiseApp) {
    env_logger::init();

    let event_loop = EventLoop::new();

    let window = window::WindowBuilder::new()
        .with_decorations(true)
        .with_transparent(false)
        .with_resizable(true)
        .with_title("Sparticles")
        .build(&event_loop)
        .unwrap();

    let show_gui = init_app.show_gui;

    let mut gfx_state = pollster::block_on(GfxState::new(window));
    let mut app_state = gfx_state.create_app_state(init_app);
    let mut gui_state = app_state.create_gui_state(show_gui);

    event_loop.run(move |event, _, control_flow| {
        // Pass the winit events to the platform integration.
        let do_exec = |window_id: WindowId| window_id == gfx_state.window_id();

        match event {
            RedrawRequested(window_id) if do_exec(window_id) => {
                app_state.update(&gfx_state, &gui_state);
                gfx_state.render(&app_state, &mut gui_state);
            }
            MainEventsCleared => {
                gfx_state.request_redraw();
            }
            WindowEvent { event, window_id } if do_exec(window_id) => {
                let response = gfx_state.handle_event(&event);

                if response.consumed {
                    return;
                }

                match event {
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
                }
            }
            _ => (),
        }
    });
}
