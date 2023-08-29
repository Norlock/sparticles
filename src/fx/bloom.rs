use super::blur::Blur;
use super::Blend;
use super::PostProcessState;
use super::Upscale;
use crate::traits::*;
use crate::GfxState;
use egui_wgpu::wgpu::{self};

pub struct Bloom {
    blur: Blur,
    upscale: Upscale,
    blend: Blend,
}

impl PostProcessFx for Bloom {
    fn compute<'a>(&'a self, input: Vec<&'a wgpu::BindGroup>, c_pass: &mut wgpu::ComputePass<'a>) {
        self.blur.compute(input.clone(), c_pass);
        self.upscale.compute(vec![self.blur.output()], c_pass);
        self.blend
            .compute(vec![input[0], self.upscale.output()], c_pass);
    }

    fn resize(&mut self, gfx_state: &GfxState, dispatch_xy: &[u32; 2]) {
        self.blur.resize(gfx_state, dispatch_xy);
        self.upscale.resize(gfx_state, dispatch_xy);
        self.blend.resize(gfx_state, dispatch_xy);
    }

    fn enabled(&self) -> bool {
        true
    }
}

impl Bloom {
    pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

    pub fn new(gfx_state: &GfxState, pp: &PostProcessState) -> Self {
        let blur = Blur::new(gfx_state, pp, "split_bloom");
        let upscale = Upscale::new(gfx_state, &blur.bind_group_layout, &pp.res);
        let blend = Blend::new(gfx_state, pp, &upscale.out_bind_group_layout);

        Self {
            upscale,
            blur,
            blend,
        }
    }
}
