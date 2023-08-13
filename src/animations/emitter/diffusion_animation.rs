use crate::{
    model::{Clock, Emitter, LifeCycle},
    traits::EmitterAnimation,
};

pub struct DiffusionAnimation {
    pub life_cycle: LifeCycle,
    pub start_diff_width: f32,
    pub end_diff_width: f32,
    pub start_diff_depth: f32,
    pub end_diff_depth: f32,
}

impl EmitterAnimation for DiffusionAnimation {
    fn animate(&mut self, emitter: &mut Emitter, clock: &Clock) {
        let current_sec = self.life_cycle.get_current_sec(clock);

        if !self.life_cycle.shoud_animate(current_sec) {
            return;
        }

        let fraction = self.life_cycle.get_fraction(current_sec);

        emitter.diff_width =
            self.start_diff_width + fraction * (self.end_diff_width - self.start_diff_depth);
        emitter.diff_depth =
            self.start_diff_depth + fraction * (self.end_diff_depth - self.start_diff_depth);
    }
}
