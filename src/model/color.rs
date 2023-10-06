use glam::{Vec3, Vec4};

use crate::{
    math::SparVec4,
    traits::{FromRGB, FromRGBA},
};

impl FromRGB for Vec4 {
    fn from_rgb(ru: u8, gu: u8, bu: u8) -> Self {
        let r = ru as f32 / 255.;
        let g = gu as f32 / 255.;
        let b = bu as f32 / 255.;

        Self::new(r, g, b, 1.0)
    }
}

impl FromRGB for SparVec4 {
    fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        let r = r as f32 / 255.;
        let g = g as f32 / 255.;
        let b = b as f32 / 255.;

        [r, g, b, 1.0].into()
    }
}

impl FromRGBA for SparVec4 {
    fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        let r = r as f32 / 255.;
        let g = g as f32 / 255.;
        let b = b as f32 / 255.;
        let a = a as f32 / 255.;

        [r, g, b, a].into()
    }
}

impl FromRGBA for Vec4 {
    fn from_rgba(ru: u8, gu: u8, bu: u8, au: u8) -> Self {
        let r = ru as f32 / 255.;
        let g = gu as f32 / 255.;
        let b = bu as f32 / 255.;
        let a = au as f32 / 255.;

        Self::new(r, g, b, a)
    }
}

impl FromRGB for Vec3 {
    fn from_rgb(ru: u8, gu: u8, bu: u8) -> Self {
        let r = ru as f32 / 255.;
        let g = gu as f32 / 255.;
        let b = bu as f32 / 255.;

        Self::new(r, g, b)
    }
}
