use super::GfxState;
use crate::{
    loader::CIRCLE_MAT_ID,
    texture::TexType,
    traits::{BufferContent, CreateFxView},
    util::ID,
};
use egui_wgpu::wgpu::{self, util::DeviceExt};
use encase::ShaderType;
use std::{collections::HashMap, num::NonZeroU64};

pub struct Material {
    pub ctx: MaterialCtx,
    pub bg: wgpu::BindGroup,
    pub bg_layout: wgpu::BindGroupLayout,
    pub uniform: MaterialUniform,
    pub buf: wgpu::Buffer,
}

#[derive(ShaderType, Clone, Copy)]
pub struct MaterialUniform {
    pub emissive_strength: f32,
    pub emissive_factor: glam::Vec3,
}

pub struct MaterialCtx {
    pub albedo_tex: wgpu::Texture,
    pub albedo_s: wgpu::Sampler,
    pub metallic_roughness_tex: wgpu::Texture,
    pub metallic_roughness_s: wgpu::Sampler,
    pub normal_tex: wgpu::Texture,
    pub normal_s: wgpu::Sampler,
    pub emissive_tex: wgpu::Texture,
    pub emissive_s: wgpu::Sampler,
    pub emissive_strength: f32,
    pub emissive_factor: glam::Vec3,
    pub ao_tex: wgpu::Texture,
    pub ao_s: wgpu::Sampler,
    pub cull_mode: Option<wgpu::Face>,
}

impl Material {
    pub fn create_builtin(gfx: &GfxState) -> HashMap<ID, Material> {
        let mut materials = HashMap::new();

        // White
        let albedo_tex = gfx.create_builtin_tex(TexType::White);
        let albedo_s = gfx.create_sampler();
        let metallic_roughness_tex = gfx.create_builtin_tex(TexType::Black);
        let metallic_roughness_s = gfx.create_sampler();

        let normal_tex = gfx.create_builtin_tex(TexType::Normal);
        let normal_s = gfx.create_sampler();
        let emissive_tex = gfx.create_builtin_tex(TexType::Black);
        let emissive_s = gfx.create_sampler();
        let ao_tex = gfx.create_builtin_tex(TexType::White);
        let ao_s = gfx.create_sampler();

        materials.insert(
            CIRCLE_MAT_ID.to_string(),
            Self::new(
                MaterialCtx {
                    albedo_tex,
                    albedo_s,
                    metallic_roughness_tex,
                    metallic_roughness_s,
                    normal_tex,
                    normal_s,
                    emissive_tex,
                    emissive_s,
                    emissive_factor: glam::Vec3::ONE,
                    emissive_strength: 1.0,
                    ao_tex,
                    ao_s,
                    cull_mode: Some(wgpu::Face::Back),
                },
                gfx,
            ),
        );

        materials
    }

    pub fn new(ctx: MaterialCtx, gfx: &GfxState) -> Self {
        let device = &gfx.device;

        let mut entries = vec![];

        for i in 0..5 {
            entries.push(wgpu::BindGroupLayoutEntry {
                binding: i * 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            });

            entries.push(wgpu::BindGroupLayoutEntry {
                binding: i * 2 + 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            });
        }

        // Material Uniform
        let uniform = MaterialUniform {
            emissive_strength: ctx.emissive_strength,
            emissive_factor: ctx.emissive_factor,
        };

        let buffer_content = uniform.buffer_content();

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Material uniform"),
            contents: &buffer_content,
            usage: wgpu::BufferUsages::UNIFORM,
        });

        entries.push(wgpu::BindGroupLayoutEntry {
            binding: 10,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: NonZeroU64::new(uniform_buffer.size()),
            },
            count: None,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &entries,
            label: Some("Material layout"),
        });

        let albedo_view = ctx.albedo_tex.default_view();
        let normal_view = ctx.normal_tex.default_view();
        let metal_roughness_view = ctx.metallic_roughness_tex.default_view();
        let emissive_view = ctx.emissive_tex.default_view();
        let ao_view = ctx.ao_tex.default_view();

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&albedo_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&ctx.albedo_s),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&normal_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&ctx.normal_s),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&metal_roughness_view),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(&ctx.metallic_roughness_s),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::TextureView(&emissive_view),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::Sampler(&ctx.emissive_s),
                },
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: wgpu::BindingResource::TextureView(&ao_view),
                },
                wgpu::BindGroupEntry {
                    binding: 9,
                    resource: wgpu::BindingResource::Sampler(&ctx.ao_s),
                },
                wgpu::BindGroupEntry {
                    binding: 10,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
            label: None,
        });

        Self {
            ctx,
            bg: bind_group,
            bg_layout: bind_group_layout,
            uniform,
            buf: uniform_buffer,
        }
    }
}
