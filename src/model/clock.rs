use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
pub struct Clock {
    instant: Instant,
    last_update: Duration,
    current_delta: Duration,
    frame: u32,
}

impl Clock {
    pub fn new() -> Self {
        Self {
            instant: Instant::now(),
            last_update: Duration::ZERO,
            current_delta: Duration::ZERO,
            frame: 0,
        }
    }

    pub fn update(&mut self) {
        let now = self.instant.elapsed();
        self.current_delta = now - self.last_update;
        self.last_update = now;
        self.frame += 1;
    }

    pub fn delta(&self) -> Duration {
        self.current_delta
    }

    pub fn delta_sec(&self) -> f32 {
        self.current_delta.as_secs_f32()
    }

    /// Elapsed last update since start
    pub fn elapsed_sec(&self) -> f32 {
        self.last_update.as_secs_f32()
    }

    pub fn elapsed_sec_f64(&self) -> f64 {
        self.last_update.as_secs_f64()
    }

    /// Elapsed now
    pub fn realtime_elapsed(&self) -> Duration {
        self.instant.elapsed()
    }

    pub fn frame(&self) -> u32 {
        self.frame
    }
}
