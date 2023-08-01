use glam::Vec4;

use crate::traits::FromRGB;

impl FromRGB for Vec4 {
    fn from_rgb(ru: u8, gu: u8, bu: u8) -> Self {
        let r = ru as f32 / 255.;
        let g = gu as f32 / 255.;
        let b = bu as f32 / 255.;

        Self::new(r, g, b, 1.0)
    }

    fn from_rgba(ru: u8, gu: u8, bu: u8, au: u8) -> Self {
        let r = ru as f32 / 255.;
        let g = gu as f32 / 255.;
        let b = bu as f32 / 255.;
        let a = au as f32 / 255.;

        Self::new(r, g, b, a)
    }
}
