use crate::state::run;

mod camera;
mod clock;
mod examples;
mod forces;
mod instance;
mod render;
mod state;
mod time;

fn main() {
    pollster::block_on(run());
}
