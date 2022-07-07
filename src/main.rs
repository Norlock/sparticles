use crate::state::run;

mod animations;
mod camera;
mod clock;
mod examples;
mod forces;
mod instance;
mod life_cycle;
mod render;
mod state;

fn main() {
    pollster::block_on(run());
}
