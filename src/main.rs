use crate::state::run;

mod camera;
mod clock;
mod examples;
mod forces;
mod instance;
mod render;
mod state;

fn main() {
    pollster::block_on(run());
}
