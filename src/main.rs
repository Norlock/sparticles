use crate::state::run;

mod camera;
mod clock;
mod examples;
mod instance;
mod render;
mod state;

fn main() {
    pollster::block_on(run());
}
