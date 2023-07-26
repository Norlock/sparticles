#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    /// values from 0 - 255
    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: (r as f32 / 255.),
            g: (g as f32 / 255.),
            b: (b as f32 / 255.),
            a: (a as f32 / 255.),
        }
    }

    /// values from 0 - 255
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self {
            r: (r as f32 / 255.),
            g: (g as f32 / 255.),
            b: (b as f32 / 255.),
            a: 1.,
        }
    }

    pub fn transparent() -> Self {
        Self {
            r: 0.,
            g: 0.,
            b: 0.,
            a: 0.,
        }
    }
}
