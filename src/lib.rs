use egui_winit::winit;
use init::AppSettings;
use model::State;
use winit::event::Event::*;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{self, WindowId};

pub mod animations;
pub mod fx;
pub mod init;
pub mod math;
pub mod model;
pub mod shaders;
pub mod texture;
pub mod traits;
mod util;

pub fn start(init_app: impl AppSettings) {
    env_logger::init();

    let event_loop = EventLoop::new();

    let window = window::WindowBuilder::new()
        .with_decorations(true)
        .with_transparent(false)
        //.with_resizable(false)
        //.with_max_inner_size(PhysicalSize::new(1920., 1080.))
        .with_title("Sparticles")
        .build(&event_loop)
        .unwrap();

    let mut state = State::new(init_app, window);
    let mut shift_pressed = false;

    event_loop.run(move |event, _, control_flow| {
        let gfx_state = &mut state.gfx_state;
        let do_exec = |window_id: WindowId| window_id == gfx_state.window_id();

        match event {
            RedrawRequested(window_id) if do_exec(window_id) => {
                state.update();
                state.render();
            }
            MainEventsCleared => {
                gfx_state.request_redraw();
            }
            WindowEvent { event, window_id } if do_exec(window_id) => {
                let response = gfx_state.handle_event(&event);

                match event {
                    winit::event::WindowEvent::Resized(size) => {
                        state.resize(size);
                    }
                    winit::event::WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(*new_inner_size);
                    }
                    winit::event::WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    winit::event::WindowEvent::KeyboardInput { input, .. } => {
                        if !response.consumed {
                            state.process_events(input, shift_pressed);
                        }
                    }
                    winit::event::WindowEvent::ModifiersChanged(modifier) => {
                        shift_pressed = modifier.shift()
                    }
                    _ => {}
                }
            }
            _ => (),
        }
    });
}
