use std::time::{Duration, Instant};

pub struct Clock {
    instant: Instant,
    last_update: Duration,
    current_delta: Duration,
    frame: usize,
}

impl Default for Clock {
    fn default() -> Self {
        Self::new()
    }
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

    pub fn elapsed_sec(&self) -> f32 {
        self.instant.elapsed().as_secs_f32()
    }

    pub fn elapsed_sec_f64(&self) -> f64 {
        self.instant.elapsed().as_secs_f64()
    }

    pub fn frame(&self) -> usize {
        self.frame
    }

    pub fn get_bindgroup_nr(&self) -> usize {
        (self.frame) % 2
    }

    pub fn get_alt_bindgroup_nr(&self) -> usize {
        (self.frame + 1) % 2
    }

    pub fn fps_text(&self) -> String {
        format!("FPS: {:.0}", 1. / self.delta_sec())
    }

    pub fn elapsed_text(&self) -> String {
        format!("Time running: {:.2}", self.elapsed_sec())
    }

    pub fn frame_time_text(&self) -> String {
        let cpu_time = self.delta_sec();
        format!("Frame time ms: {:.0}", cpu_time * 1000.)
    }
}
