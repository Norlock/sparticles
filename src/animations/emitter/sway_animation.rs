use crate::{
    model::{Clock, Emitter, LifeCycle},
    traits::EmitterAnimation,
};

pub struct SwayAnimation {
    pub life_cycle: LifeCycle,
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
}

impl EmitterAnimation for SwayAnimation {
    fn animate(&mut self, emitter: &mut Emitter, clock: &Clock) {
        let current_sec = self.life_cycle.get_current_sec(clock);

        if !self.life_cycle.shoud_animate(current_sec) {
            return;
        }

        let fraction = self.life_cycle.get_fraction(current_sec);

        emitter.box_rotation.x += fraction * self.pitch * clock.delta_sec();
        emitter.box_rotation.y += fraction * self.roll * clock.delta_sec();
        emitter.box_rotation.z += fraction * self.yaw * clock.delta_sec();
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
