use egui_winit::egui::{DragValue, Ui};
use glam::Vec2;

use crate::{
    model::{Clock, Emitter, LifeCycle},
    traits::EmitterAnimation,
};

pub struct SwayAnimation {
    pub life_cycle: LifeCycle,
    pub yaw: Vec2,
    pub pitch: Vec2,
    pub roll: Vec2,
}

impl EmitterAnimation for SwayAnimation {
    fn animate(&mut self, emitter: &mut Emitter, clock: &Clock) {
        let current_sec = self.life_cycle.get_current_sec(clock);

        if !self.life_cycle.shoud_animate(current_sec) {
            return;
        }

        let fraction = self.life_cycle.get_fraction(current_sec);

        emitter.box_rotation.x = self.yaw.x + fraction * (self.yaw.y - self.yaw.x);
        emitter.box_rotation.y = self.pitch.x + fraction * (self.pitch.y - self.pitch.x);
        emitter.box_rotation.z = self.roll.x + fraction * (self.roll.y - self.roll.x);
    }

    fn create_gui(&mut self, ui: &mut Ui) {
        let life_cycle = &mut self.life_cycle;

        ui.label("Sway animation");

        ui.horizontal(|ui| {
            ui.label("Animate from sec");
            ui.add(DragValue::new(&mut life_cycle.from_sec).speed(0.1));
        });

        ui.horizontal(|ui| {
            ui.label("Animate until sec");
            ui.add(
                DragValue::new(&mut life_cycle.until_sec)
                    .speed(0.1)
                    .clamp_range(life_cycle.from_sec..=life_cycle.lifetime_sec),
            );
        });

        ui.horizontal(|ui| {
            ui.label("Until restart animation");
            ui.add(DragValue::new(&mut life_cycle.lifetime_sec).speed(0.1));
        });
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
