use std::time::{Duration, Instant};

pub struct Clock {
    instant: Instant,
    last_update: Duration,
    current_delta: Duration,
}

impl Clock {
    pub fn new() -> Self {
        Self {
            instant: Instant::now(),
            last_update: Duration::ZERO,
            current_delta: Duration::ZERO,
        }
    }

    pub fn update(&mut self) {
        let now = self.instant.elapsed();
        self.current_delta = now - self.last_update;
        self.last_update = now;
    }

    pub fn delta(&self) -> Duration {
        self.current_delta
    }

    pub fn delta_sec(&self) -> f32 {
        self.current_delta.as_secs_f32()
    }

    pub fn elapsed(&self) -> Duration {
        self.instant.elapsed()
    }

    pub fn elapsed_ms(&self) -> u128 {
        self.instant.elapsed().as_millis()
    }
}
