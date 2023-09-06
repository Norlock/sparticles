use super::blend::BlendType;
use super::blur::Blur;
use super::post_process::FxChainOutput;
use super::Upscale;
use crate::traits::*;
use crate::GfxState;
use egui_wgpu::wgpu::{self};

pub struct Bloom {
    blur: Blur,
    upscale: Upscale,
}

impl PostFxChain for Bloom {
    fn compute<'a>(
        &'a self,
        input: &'a wgpu::BindGroup,
        c_pass: &mut wgpu::ComputePass<'a>,
    ) -> FxChainOutput {
        self.blur.compute(vec![input], c_pass);
        self.upscale.compute(vec![self.blur.output()], c_pass);

        FxChainOutput {
            blend: BlendType::ADDITIVE,
            bind_group: self.upscale.output(),
        }
    }

    fn resize(&mut self, gfx_state: &GfxState) {
        self.blur.resize(gfx_state);
        self.upscale.resize(gfx_state);
    }

    fn enabled(&self) -> bool {
        true
    }
}

impl Bloom {
    pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

    pub fn new(gfx_state: &GfxState, depth_view: &wgpu::TextureView) -> Self {
        let blur = Blur::new(gfx_state, depth_view, "split_bloom");
        let upscale = Upscale::new(gfx_state);

        Self { blur, upscale }
    }
}
