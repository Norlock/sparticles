use super::Clock;

#[derive(Debug, Clone, Copy)]
pub struct LifeCycle {
    pub from_sec: f32,
    pub until_sec: f32,
    /// Time until the animmation repeats
    pub lifetime_sec: f32,
}

impl LifeCycle {
    pub fn get_current_sec(&self, clock: &Clock) -> f32 {
        clock.elapsed_sec() % self.lifetime_sec
    }

    pub fn shoud_animate(&self, current_sec: f32) -> bool {
        self.from_sec <= current_sec && current_sec <= self.until_sec
    }

    pub fn get_fraction(&self, current_sec: f32) -> f32 {
        let delta_current = current_sec - self.from_sec;
        let delta_max = self.until_sec - self.from_sec;
        delta_current / delta_max
    }
}
