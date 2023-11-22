use crate::{loader::CIRCLE_MAT_ID, traits::CreateFxView, util::ID};
use egui_wgpu::wgpu;
use std::{collections::HashMap, path::PathBuf};

use super::GfxState;

pub struct Material {
    pub albedo_tex: wgpu::Texture,
    pub metallic_roughness_tex: wgpu::Texture,
    pub normal_tex: wgpu::Texture,
    pub emissive_tex: wgpu::Texture,
    pub ao_tex: wgpu::Texture,
    pub sampler: wgpu::Sampler,
    pub bg: wgpu::BindGroup,
    pub bg_layout: wgpu::BindGroupLayout,
}

pub struct MaterialCtx<'a> {
    pub albedo_tex: wgpu::Texture,
    pub metallic_roughness_tex: wgpu::Texture,
    pub normal_tex: wgpu::Texture,
    pub emissive_tex: wgpu::Texture,
    pub ao_tex: wgpu::Texture,
    pub gfx_state: &'a GfxState,
}

fn get_path_buf() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

impl Material {
    pub fn create_builtin(gfx: &GfxState) -> HashMap<ID, Material> {
        let mut materials = HashMap::new();
        // TODO create default material
        let mut texture_image = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        texture_image.push("src/assets/textures/1x1.png");

        let tex_str = texture_image.to_str().expect("niet goed");

        // White
        let diffuse_tex = gfx.tex_from_string(tex_str, true);
        let metallic_roughness_tex = gfx.tex_from_string(tex_str, true);

        // TODO flat normal (bump)
        let mut norm_buf = get_path_buf();
        norm_buf.push("src/assets/textures/circle_normal.png");
        let n_tex_str = norm_buf.to_str().expect("niet goed");

        let normal_tex = gfx.tex_from_string(n_tex_str, false);
        let emissive_tex = gfx.tex_from_string(tex_str, true);
        let occlusion_tex = gfx.tex_from_string(tex_str, true);

        materials.insert(
            CIRCLE_MAT_ID.to_string(),
            Self::new(MaterialCtx {
                gfx_state: gfx,
                albedo_tex: diffuse_tex,
                metallic_roughness_tex,
                normal_tex,
                emissive_tex,
                ao_tex: occlusion_tex,
            }),
        );

        materials
    }

    pub fn new(mat: MaterialCtx) -> Self {
        let gfx = mat.gfx_state;
        let device = &gfx.device;

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: None,
        });

        let diff_view = mat.albedo_tex.default_view();
        let normal_view = mat.normal_tex.default_view();
        let metal_view = mat.metallic_roughness_tex.default_view();
        let emiss_view = mat.emissive_tex.default_view();
        let ao_view = mat.ao_tex.default_view();

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diff_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&normal_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&metal_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&emiss_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&ao_view),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: None,
        });

        Self {
            emissive_tex: mat.emissive_tex,
            normal_tex: mat.normal_tex,
            albedo_tex: mat.albedo_tex,
            metallic_roughness_tex: mat.metallic_roughness_tex,
            ao_tex: mat.ao_tex,
            sampler,
            bg: bind_group,
            bg_layout: bind_group_layout,
        }
    }
}
