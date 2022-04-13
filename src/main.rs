use crate::sparticles_state::run;

mod camera;
mod clock;
mod instance;
mod sparticles_state;
mod texture;

fn main() {
    pollster::block_on(run());
}
