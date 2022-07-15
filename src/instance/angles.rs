#[derive(Debug)]
pub struct Angles {
    pub elevation: f32,
    pub bearing: f32,
}

impl Angles {
    pub fn new(elevation: f32, bearing: f32) -> Self {
        Self { elevation, bearing }
    }

    pub fn to_radians(&self) -> Self {
        Angles::new(self.elevation.to_radians(), self.bearing.to_radians())
    }

    pub fn to_degrees(&self) -> Self {
        Angles::new(self.elevation.to_degrees(), self.bearing.to_degrees())
    }
}
