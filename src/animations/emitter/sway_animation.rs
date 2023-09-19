use egui_winit::egui::{DragValue, Ui};
use glam::Vec2;

use crate::{
    model::{Clock, Emitter, LifeCycle},
    traits::{EmitterAnimation, HandleAngles},
};

struct Gui {
    yaw: Vec2,
    pitch: Vec2,
    roll: Vec2,
}

pub struct SwayAnimation {
    life_cycle: LifeCycle,
    yaw: Vec2,
    pitch: Vec2,
    roll: Vec2,
    gui: Gui,
}

impl SwayAnimation {
    pub fn new(life_cycle: LifeCycle, yaw_deg: Vec2, pitch_deg: Vec2, roll_deg: Vec2) -> Self {
        let gui = Gui {
            yaw: yaw_deg,
            pitch: pitch_deg,
            roll: roll_deg,
        };

        Self {
            life_cycle,
            yaw: yaw_deg.to_radians(),
            pitch: pitch_deg.to_radians(),
            roll: roll_deg.to_radians(),
            gui,
        }
    }
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
        let gui = &mut self.gui;

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

        ui.horizontal(|ui| {
            ui.label("Pitch (from - until)");
            ui.add(DragValue::new(&mut gui.pitch.x).speed(0.1));
            ui.add(DragValue::new(&mut gui.pitch.y).speed(0.1));
        });

        ui.horizontal(|ui| {
            ui.label("Yaw (from - until)");
            ui.add(DragValue::new(&mut gui.yaw.x).speed(0.1));
            ui.add(DragValue::new(&mut gui.yaw.y).speed(0.1));
        });

        ui.horizontal(|ui| {
            ui.label("Roll (from - until)");
            ui.add(DragValue::new(&mut gui.roll.x).speed(0.1));
            ui.add(DragValue::new(&mut gui.roll.y).speed(0.1));
        });

        self.yaw = gui.yaw.to_radians();
        self.pitch = gui.pitch.to_radians();
        self.roll = gui.pitch.to_radians();
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
