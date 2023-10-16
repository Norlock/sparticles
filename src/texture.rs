use std::{fs, path::PathBuf};

use crate::{
    fx::PostProcessState,
    model::gfx_state::GfxState,
    traits::{CreateFxView, FxDimensions},
};
use egui_wgpu::wgpu::{self, util::align_to};
use image::GenericImageView;
use rand::{rngs::ThreadRng, Rng};

pub struct DiffuseTexture {
    pub sampler: wgpu::Sampler,
    pub view: wgpu::TextureView,
}

pub struct CustomTexture;

impl CustomTexture {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_path: &str,
    ) -> wgpu::TextureView {
        let bytes = fs::read(texture_path).expect("Can't read texture image");
        let diffuse_image = image::load_from_memory(&bytes).unwrap();
        let diffuse_rgba = diffuse_image.to_rgba8();

        let dimensions = diffuse_image.dimensions();

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let diffuse_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("diffuse_texture"),
            view_formats: &[],
        });

        queue.write_texture(
            diffuse_texture.as_image_copy(),
            &diffuse_rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_size,
        );

        diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default())
    }
}

impl GfxState {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn create_depth_view(&self) -> wgpu::TextureView {
        let device = &self.device;
        let config = &self.surface_config;
        let dimensions = config.fx_dimensions();

        let desc = wgpu::TextureDescriptor {
            label: Some("Depth texture"),
            size: wgpu::Extent3d {
                width: dimensions[0],
                height: dimensions[1],
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };

        device.create_texture(&desc).default_view()
    }

    pub fn create_frame_view(&self) -> wgpu::TextureView {
        let config = &self.surface_config;
        let dimensions = config.fx_dimensions();

        self.device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Frame view"),
                size: wgpu::Extent3d {
                    width: dimensions[0],
                    height: dimensions[1],
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                view_formats: &[],
                dimension: wgpu::TextureDimension::D2,
                format: config.format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
            })
            .default_view()
    }

    pub fn create_fx_view(&self) -> wgpu::TextureView {
        let format = PostProcessState::TEXTURE_FORMAT;

        self.device
            .create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: self.surface_config.width,
                    height: self.surface_config.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                view_formats: &[],
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
            })
            .default_view()
    }

    pub fn create_diffuse_texture(&self, texture_path: &str) -> DiffuseTexture {
        let device = &self.device;

        let bytes = fs::read(texture_path).expect("Can't read texture image");
        let diffuse_image = image::load_from_memory(&bytes).unwrap();
        let diffuse_rgba = diffuse_image.to_rgba8();

        let dimensions = diffuse_image.dimensions();

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let diffuse_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("diffuse_texture"),
            view_formats: &[],
        });

        self.queue.write_texture(
            diffuse_texture.as_image_copy(),
            &diffuse_rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_size,
        );

        let view = diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        DiffuseTexture { sampler, view }
    }

    pub fn create_noise_view(&self) -> wgpu::TextureView {
        let device = &self.device;
        let queue = &self.queue;

        let size = wgpu::Extent3d {
            width: self.surface_config.width,
            height: self.surface_config.height,
            depth_or_array_layers: 1,
        };

        let mut noise_data = Vec::new();

        let bytes_per_row = align_to(
            size.width * std::mem::size_of::<f32>() as u32,
            wgpu::COPY_BYTES_PER_ROW_ALIGNMENT,
        );

        let mut rand = ThreadRng::default();

        for _ in 0..size.height {
            for _ in 0..size.width {
                noise_data.push(rand.gen_range(-1.0..=1.0));
            }
        }

        let tex_content = bytemuck::cast_slice(&noise_data);

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            view_formats: &[],
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            tex_content,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(size.height),
            },
            size,
        );

        texture.default_view()
    }
}

impl CreateFxView for wgpu::Texture {
    fn default_view(&self) -> wgpu::TextureView {
        self.create_view(&wgpu::TextureViewDescriptor::default())
    }
}
