use std::fmt::{Display, Formatter, Result};

use encase::{private::WriteInto, ShaderType, UniformBuffer};
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Serialize, Deserialize, Default)]
pub enum ItemAction {
    #[default]
    None,
    Delete,
    MoveUp,
    MoveDown,
    Disable,
}

impl Display for ItemAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Disable => f.write_str("Disable"),
            Self::MoveUp => f.write_str("Move up"),
            Self::MoveDown => f.write_str("Move down"),
            Self::Delete => f.write_str("Delete"),
            Self::None => f.write_str("None"),
        }
    }
}

pub fn create_buffer_content(to_buffer: &(impl ShaderType + WriteInto)) -> Vec<u8> {
    let mut buffer = UniformBuffer::new(Vec::new());
    buffer.write(&to_buffer).unwrap();
    buffer.into_inner()
}
