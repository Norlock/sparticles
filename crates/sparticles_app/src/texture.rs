use crate::{fx::PostProcessState, model::gfx_state::GfxState, traits::CreateFxView};
use async_std::sync::RwLock;
use egui_wgpu::wgpu::{self, util::align_to};
use glam::Vec4;
use image::GenericImageView;
use rand::{rngs::ThreadRng, Rng};
use std::{fs, sync::Arc};

pub struct DiffuseCtx {
    pub sampler: wgpu::Sampler,
    pub view: wgpu::TextureView,
    pub bg: wgpu::BindGroup,
    pub bg_layout: wgpu::BindGroupLayout,
}

pub struct IconTexture;

const MAX_FX_WIDTH: f32 = 2048.;
const MAX_FX_HEIGHT: f32 = 1024.;

pub enum TexType {
    White,
    Black,
    Normal,
    Custom { srgb: bool, value: Vec4 },
}

impl IconTexture {
    pub fn create_view(
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
    pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

    pub fn aspect(&self) -> f32 {
        self.surface_config.width as f32 / self.surface_config.height as f32
    }

    pub fn dimensions(&self) -> (f32, f32) {
        let width = self.surface_config.width as f32;
        let height = self.surface_config.height as f32;

        let ratio_x = width / MAX_FX_WIDTH;
        let ratio_y = height / MAX_FX_HEIGHT;

        if 1.0 < ratio_x || 1.0 < ratio_y {
            if ratio_y < ratio_x {
                (MAX_FX_WIDTH, height / ratio_x)
            } else {
                (width / ratio_y, MAX_FX_HEIGHT)
            }
        } else {
            (width, height)
        }
    }

    fn tex_size(&self) -> wgpu::Extent3d {
        let (width, height) = self.dimensions();

        wgpu::Extent3d {
            width: width as u32,
            height: height as u32,
            depth_or_array_layers: 1,
        }
    }

    pub fn create_depth_view(&self) -> wgpu::TextureView {
        let device = &self.device;

        let desc = wgpu::TextureDescriptor {
            label: Some("Depth texture"),
            size: self.tex_size(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };

        device.create_texture(&desc).default_view()
    }

    pub fn create_fx_view(&self) -> wgpu::TextureView {
        self.device
            .create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: self.tex_size(),
                mip_level_count: 1,
                sample_count: 1,
                view_formats: &[],
                dimension: wgpu::TextureDimension::D2,
                format: Self::TEXTURE_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::STORAGE_BINDING,
            })
            .default_view()
    }

    pub fn create_mip_fx_view(&self, mip_level: u32) -> wgpu::TextureView {
        self.device
            .create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: self.tex_size(),
                mip_level_count: 5,
                sample_count: 1,
                view_formats: &[],
                dimension: wgpu::TextureDimension::D2,
                format: Self::TEXTURE_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::STORAGE_BINDING,
            })
            .create_view(&wgpu::TextureViewDescriptor {
                label: Some("mip"),
                format: None,
                dimension: None,
                aspect: wgpu::TextureAspect::All,
                base_mip_level: mip_level,
                mip_level_count: Some(1),
                base_array_layer: 0,
                array_layer_count: None,
            })
    }

    pub fn create_sampler(&self) -> wgpu::Sampler {
        self.device
            .create_sampler(&wgpu::SamplerDescriptor::default())
    }

    pub fn create_builtin_tex(&self, tex_type: TexType) -> wgpu::Texture {
        let (format, bytes) = match tex_type {
            TexType::White => (wgpu::TextureFormat::Rgba8UnormSrgb, [255, 255, 255, 255]),
            TexType::Black => (wgpu::TextureFormat::Rgba8UnormSrgb, [0, 0, 0, 0]),
            TexType::Normal => (wgpu::TextureFormat::Rgba8Unorm, [127, 127, 255, 255]),
            TexType::Custom { srgb, value: v } => {
                let bytes = (v * 255.).round();
                let rgba8 = [bytes.x as u8, bytes.y as u8, bytes.z as u8, bytes.w as u8];

                if srgb {
                    (wgpu::TextureFormat::Rgba8UnormSrgb, rgba8)
                } else {
                    (wgpu::TextureFormat::Rgba8Unorm, rgba8)
                }
            }
        };

        let tex = self.device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d::default(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("diffuse_texture"),
            view_formats: &[],
        });

        self.queue.write_texture(
            tex.as_image_copy(),
            &bytes,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: None,
            },
            wgpu::Extent3d::default(),
        );

        tex
    }

    pub fn create_noise_view(&self) -> wgpu::TextureView {
        let device = &self.device;
        let queue = &self.queue;

        let size = self.tex_size();
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

pub struct TextureHandler;

impl TextureHandler {
    pub async fn tex_from_string(
        gfx_arc: &Arc<RwLock<GfxState>>,
        path: &str,
        std_rgb: bool,
    ) -> wgpu::Texture {
        let bytes = fs::read(path).expect("Can't read texture image");
        Self::tex_from_bytes(gfx_arc, &bytes, std_rgb).await
    }

    pub async fn tex_from_bytes(
        gfx_arc: &Arc<RwLock<GfxState>>,
        bytes: &[u8],
        std_rgb: bool,
    ) -> wgpu::Texture {
        let diffuse_image = image::load_from_memory(bytes).unwrap();
        let diffuse_rgba = diffuse_image.to_rgba8();
        let dimensions = diffuse_image.dimensions();

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let format = if std_rgb {
            wgpu::TextureFormat::Rgba8UnormSrgb
        } else {
            wgpu::TextureFormat::Rgba8Unorm
        };

        let gfx = gfx_arc.read().await;
        let device = &gfx.device;

        let diffuse_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("diffuse_texture"),
            view_formats: &[],
        });

        gfx.queue.write_texture(
            diffuse_texture.as_image_copy(),
            &diffuse_rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_size,
        );

        diffuse_texture
    }
}
