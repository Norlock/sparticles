pub mod emitter;
pub mod particle;

pub use emitter::*;
use encase::{private::WriteInto, ShaderType, UniformBuffer};
pub use particle::*;

pub fn create_buffer_content(to_buffer: &(impl ShaderType + WriteInto)) -> Vec<u8> {
    let mut buffer = UniformBuffer::new(Vec::new());
    buffer.write(&to_buffer).unwrap();
    buffer.into_inner()
}
