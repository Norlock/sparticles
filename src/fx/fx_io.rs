use super::FxState;
use crate::util::CommonBuffer;
use crate::{model::GfxState, util::UniformContext};
use egui_wgpu::wgpu::{self, util::DeviceExt};
use encase::ShaderType;
use serde::{Deserialize, Serialize};
use std::num::NonZeroU64;

#[derive(ShaderType, Clone, Copy, Serialize, Deserialize, Debug)]
pub struct FxIOUniform {
    pub in_idx: u32,
    pub out_idx: u32,

    pub in_downscale: u32,
    pub out_downscale: u32,

    pub in_size_x: u32,
    pub in_size_y: u32,

    pub out_size_x: u32,
    pub out_size_y: u32,
}

pub struct FxIO {
    pub in_idx: u32,
    pub out_idx: u32,

    pub in_downscale: u32,
    pub out_downscale: u32,

    pub in_size: glam::Vec2,
    pub out_size: glam::Vec2,
}

pub struct FxIOUniformOptions {
    pub in_idx: u32,
    pub out_idx: u32,
    pub in_downscale: f32,
    pub out_downscale: f32,
}

pub struct FxOptions<'a> {
    pub gfx_state: &'a GfxState,
    pub fx_state: &'a FxState,
}

pub struct FxIOSwapCtx {
    pub uniforms: [FxIOUniform; 2],
    pub buffers: Vec<wgpu::Buffer>,
    pub bgs: Vec<wgpu::BindGroup>,
    pub bg_layout: wgpu::BindGroupLayout,
}

impl FxIOUniform {
    pub fn create_downscale_list(
        list: &mut Vec<FxIO>,
        fx_size: &glam::Vec2,
        downscale_count: i32,
        in_downscale: u32,
        io_idx: u32,
    ) -> Vec<Self> {
        let out_downscale = in_downscale * 2;
        let out_size;

        if let Some(last) = list.iter().last() {
            let in_size = last.out_size;
            out_size = (in_size / 2.).ceil();

            list.push(FxIO {
                in_idx: io_idx,
                out_idx: io_idx + 1,
                in_size,
                out_size,
                in_downscale,
                out_downscale,
            });
        } else {
            let in_size = *fx_size;
            out_size = (in_size / 2.).ceil();

            list.push(FxIO {
                in_idx: io_idx,
                out_idx: io_idx + 1,
                in_size,
                out_size,
                in_downscale,
                out_downscale,
            });
        }

        if 0 <= downscale_count - 1 {
            Self::create_downscale_list(
                list,
                fx_size,
                downscale_count - 1,
                out_downscale,
                io_idx + 1,
            )
        } else {
            list.iter()
                .map(|io| Self {
                    in_idx: io.in_idx,
                    out_idx: io.out_idx,
                    in_downscale: io.in_downscale,
                    out_downscale: io.out_downscale,
                    in_size_x: io.in_size.x as u32,
                    in_size_y: io.in_size.y as u32,
                    out_size_x: io.out_size.x as u32,
                    out_size_y: io.out_size.y as u32,
                })
                .collect()
        }
    }

    pub fn symetric_unscaled(fx_state: &FxState, io_idx: u32) -> Self {
        Self::create(
            fx_state,
            FxIOUniformOptions {
                in_idx: io_idx,
                out_idx: io_idx,
                in_downscale: 1.,
                out_downscale: 1.,
            },
        )
    }

    pub fn asymetric_unscaled(fx_state: &FxState, in_idx: u32, out_idx: u32) -> Self {
        Self::create(
            fx_state,
            FxIOUniformOptions {
                in_idx,
                out_idx,
                in_downscale: 1.,
                out_downscale: 1.,
            },
        )
    }

    pub fn reverse_list(list: &Vec<Self>) -> Vec<Self> {
        let mut result = Vec::new();

        for last in list.iter().rev() {
            result.push(Self {
                in_idx: last.out_idx,
                out_idx: last.in_idx,
                in_size_x: last.out_size_x,
                in_size_y: last.out_size_y,
                out_size_x: last.in_size_x,
                out_size_y: last.in_size_y,
                in_downscale: last.out_downscale,
                out_downscale: last.in_downscale,
            });
        }

        result
    }

    pub fn create(fx_state: &FxState, options: FxIOUniformOptions) -> Self {
        let FxIOUniformOptions {
            in_idx,
            out_idx,
            in_downscale,
            out_downscale,
        } = options;

        let in_size = (fx_state.tex_size / in_downscale).ceil();
        let out_size = (fx_state.tex_size / out_downscale).ceil();

        Self {
            in_idx,
            out_idx,
            in_size_x: in_size.x as u32,
            in_size_y: in_size.y as u32,
            out_size_x: out_size.x as u32,
            out_size_y: out_size.y as u32,
            in_downscale: 1,
            out_downscale: 1,
        }
    }

    pub fn create_content(&self) -> Vec<u8> {
        CommonBuffer::uniform_content(self)
    }

    pub fn zero(fx_state: &FxState) -> Self {
        let tex_size = &fx_state.tex_size;
        let size_x = tex_size.x as u32;
        let size_y = tex_size.y as u32;

        Self {
            in_idx: 0,
            out_idx: 0,
            in_size_x: size_x,
            in_size_y: size_y,
            out_size_x: size_x,
            out_size_y: size_y,
            in_downscale: 1,
            out_downscale: 1,
        }
    }

    pub fn resize(&mut self, buf: &wgpu::Buffer, options: &FxOptions) {
        let fx_state = &options.fx_state;

        let in_size = (fx_state.tex_size / self.in_downscale as f32).ceil();
        let out_size = (fx_state.tex_size / self.out_downscale as f32).ceil();

        *self = Self {
            in_idx: self.in_idx,
            out_idx: self.out_idx,
            in_size_x: in_size.x as u32,
            in_size_y: in_size.y as u32,
            out_size_x: out_size.x as u32,
            out_size_y: out_size.y as u32,
            in_downscale: self.in_downscale,
            out_downscale: self.out_downscale,
        };

        let queue = &options.gfx_state.queue;
        let contents = CommonBuffer::uniform_content(self);

        queue.write_buffer(buf, 0, &contents);
    }
}

impl FxIOSwapCtx {
    pub fn resize(&mut self, options: &FxOptions) {
        for (io, buf) in self.uniforms.iter_mut().zip(self.buffers.iter()) {
            io.resize(buf, options);
        }
    }

    pub fn new(uniforms: [FxIOUniform; 2], device: &wgpu::Device, label: &str) -> Self {
        let mut layout_entries = Vec::new();
        let mut buffers = Vec::new();
        let mut bgs = Vec::new();

        for (i, uniform) in uniforms.iter().enumerate() {
            let contents = CommonBuffer::uniform_content(uniform);

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
                    contents: &contents,
                }),
            );
        }

        let bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&format!("{} uniform layout", label)),
            entries: &layout_entries,
        });

        for i in 0..2 {
            bgs.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("{} uniform bindgroup", label)),
                layout: &bg_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffers[i % 2].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: buffers[(i + 1) % 2].as_entire_binding(),
                    },
                ],
            }));
        }

        Self {
            buffers,
            bgs,
            bg_layout,
            uniforms,
        }
    }
}
