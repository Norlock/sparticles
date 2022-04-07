use crate::render::run;

mod camera;
mod instance;
mod render;
mod texture;

fn main() {
    pollster::block_on(run());
}
