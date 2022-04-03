use crate::state::run;

mod camera;
mod camera_controller;
mod camera_uniform;
mod instance;
mod state;
mod texture;

fn main() {
    pollster::block_on(run());
}
