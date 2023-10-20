use std::{
    fmt::{Display, Formatter, Result},
    num::NonZeroU64,
};

use egui_wgpu::wgpu::{self, util::DeviceExt};
use encase::{private::WriteInto, ShaderType, UniformBuffer};
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Serialize, Deserialize, Default)]
pub enum ListAction {
    #[default]
    None,
    Delete,
    MoveUp,
    MoveDown,
    Disable,
}

impl Display for ListAction {
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
    pub buffers: Vec<wgpu::Buffer>,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl UniformCompute {
    pub fn new(
        // TODO array of uniform in case of more meta + global
        buffer_contents: &[&[u8]],
        device: &wgpu::Device,
        label: &str,
    ) -> Self {
        let mut layout_entries = Vec::new();
        let mut entries = Vec::new();
        let mut buffers = Vec::new();

        for (i, contents) in buffer_contents.iter().enumerate() {
            layout_entries.push(wgpu::BindGroupLayoutEntry {
                binding: i as u32,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: NonZeroU64::new(contents.len() as u64),
                },
                count: None,
            });

            buffers.push(
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    label: Some(label),
                    contents,
                }),
            );
        }

        for i in 0..buffers.len() {
            entries.push(wgpu::BindGroupEntry {
                binding: i as u32,
                resource: buffers[i].as_entire_binding(),
            });
        }

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&format!("{} uniform layout", label)),
            entries: &layout_entries,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{} uniform bindgroup", label)),
            layout: &bind_group_layout,
            entries: &entries,
        });

        UniformCompute {
            buffers,
            bind_group,
            bind_group_layout,
        }
    }
}

pub struct CommonBuffer;

impl CommonBuffer {
    pub fn uniform_content(uniform: &(impl ShaderType + WriteInto)) -> Vec<u8> {
        let mut buffer = UniformBuffer::new(Vec::new());
        buffer.write(&uniform).unwrap();
        buffer.into_inner()
    }
}
