use std::{
    fmt::{Display, Formatter, Result},
    num::NonZeroU64,
};

use egui_wgpu::wgpu::{self, util::DeviceExt};
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

pub struct UniformCompute {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

pub struct CommonBuffer;

impl UniformCompute {
    pub fn new(
        // TODO array of uniform in case of more meta + global
        uniform: &(impl ShaderType + WriteInto),
        device: &wgpu::Device,
        label: &str,
    ) -> Self {
        let contents = CommonBuffer::uniform_content(&uniform);

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{} uniform", label)),
            contents: &contents,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&format!("{} uniform layout", label)),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: NonZeroU64::new(contents.len() as u64),
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{} uniform bindgroup", label)),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        UniformCompute {
            buffer,
            bind_group,
            bind_group_layout,
        }
    }
}

impl CommonBuffer {
    pub fn uniform_content(uniform: &(impl ShaderType + WriteInto)) -> Vec<u8> {
        let mut buffer = UniformBuffer::new(Vec::new());
        buffer.write(&uniform).unwrap();
        buffer.into_inner()
    }
}
