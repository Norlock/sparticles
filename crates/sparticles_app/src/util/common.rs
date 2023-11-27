use crate::traits::{HandleAction, OtherIterMut, Splitting};
use egui_wgpu::wgpu::{self, util::DeviceExt};
use encase::{private::WriteInto, ShaderType, UniformBuffer};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, Formatter, Result},
    num::NonZeroU64,
    rc::Rc,
};

pub type ID = String;
pub type Tag = Rc<str>;

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

impl ListAction {
    pub fn update_list<T: HandleAction + ?Sized>(list: &mut Vec<Box<T>>) {
        let mut i = 0;

        while i < list.len() {
            if list[i].selected_action() == &mut ListAction::Delete {
                list.remove(i);
                continue;
            } else if 0 < i && list[i].selected_action() == &mut ListAction::MoveUp {
                *list[i].selected_action() = ListAction::None;
                list.swap(i, i - 1);
            } else if 0 < i && list[i - 1].selected_action() == &mut ListAction::MoveDown {
                *list[i - 1].selected_action() = ListAction::None;
                list.swap(i, i - 1);
            }

            i += 1;
        }
    }
}

pub struct UniformContext {
    pub buf: wgpu::Buffer,
    pub bg: wgpu::BindGroup,
    pub bg_layout: wgpu::BindGroupLayout,
}

impl UniformContext {
    pub fn from_content(buffer_content: &[u8], device: &wgpu::Device, label: &str) -> Self {
        let mut layout_entries = Vec::new();
        let mut entries = Vec::new();

        layout_entries.push(wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: NonZeroU64::new(buffer_content.len() as u64),
            },
            count: None,
        });

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            label: Some(label),
            contents: &buffer_content,
        });

        entries.push(wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&format!("{} uniform layout", label)),
            entries: &layout_entries,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{} uniform bindgroup", label)),
            layout: &bind_group_layout,
            entries: &entries,
        });

        Self {
            buf: buffer,
            bg: bind_group,
            bg_layout: bind_group_layout,
        }
    }

    pub fn from_uniform(
        uniform: &(impl ShaderType + WriteInto),
        device: &wgpu::Device,
        label: &str,
    ) -> Self {
        let buffer_contents = CommonBuffer::uniform_content(uniform);
        Self::from_content(&buffer_contents, device, label)
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

impl<T: std::fmt::Debug> Splitting<T> for Vec<T> {
    fn split_item_mut(&mut self, idx: usize) -> (&mut T, OtherIterMut<T>) {
        assert!(idx < self.len());

        let (head, rest) = self.split_at_mut(idx);
        let (item, tail) = rest.split_first_mut().unwrap();
        let others = head.iter_mut().chain(tail.iter_mut());

        (item, others)
    }
}
