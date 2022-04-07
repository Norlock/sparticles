use crate::state::run;

mod camera;
mod instance;
mod state;
mod texture;

fn main() {
    pollster::block_on(run());
}
