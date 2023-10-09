use egui_winit::egui::{DragValue, Ui};
use glam::Vec2;
use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    animations::ItemAction,
    math::SparVec2,
    model::{Clock, EmitterUniform, GuiState, LifeCycle},
    traits::{EmitterAnimation, HandleAngles, RegisterEmitterAnimation},
    util::persistence::ExportAnimation,
};

#[derive(Serialize, Deserialize)]
struct Gui {
    yaw: SparVec2,
    pitch: SparVec2,
    roll: SparVec2,
}

#[derive(Serialize, Deserialize)]
pub struct SwayAnimation {
    life_cycle: LifeCycle,
    yaw: SparVec2,
    pitch: SparVec2,
    roll: SparVec2,
    gui: Gui,

    #[serde(skip_serializing, skip_deserializing)]
    selected_action: ItemAction,
}

#[derive(Clone, Copy)]
pub struct RegisterSwayAnimation;

impl RegisterEmitterAnimation for RegisterSwayAnimation {
    fn tag(&self) -> &str {
        "sway-animation"
    }

    fn import(&self, value: serde_json::Value) -> Box<dyn EmitterAnimation> {
        let anim: SwayAnimation = serde_json::from_value(value).unwrap();
        Box::new(anim)
    }

    fn create_default(&self) -> Box<dyn EmitterAnimation> {
        let sway_animation = SwayAnimation::new(
            LifeCycle {
                from_sec: 0.,
                until_sec: 4.,
                lifetime_sec: 4.,
            },
            glam::Vec2::ZERO,
            Vec2::new(30., 120.),
            glam::Vec2::ZERO,
        );

        Box::new(sway_animation)
    }
}

impl SwayAnimation {
    pub fn new(life_cycle: LifeCycle, yaw_deg: Vec2, pitch_deg: Vec2, roll_deg: Vec2) -> Self {
        let gui = Gui {
            yaw: yaw_deg.into(),
            pitch: pitch_deg.into(),
            roll: roll_deg.into(),
        };

        Self {
            life_cycle,
            yaw: yaw_deg.to_radians().into(),
            pitch: pitch_deg.to_radians().into(),
            roll: roll_deg.to_radians().into(),
            selected_action: ItemAction::None,
            gui,
        }
    }
}

impl EmitterAnimation for SwayAnimation {
    fn export(&self) -> ExportAnimation {
        ExportAnimation {
            animation_tag: RegisterSwayAnimation.tag().to_string(),
            animation: serde_json::to_value(self).unwrap(),
        }
    }

    fn animate(&mut self, emitter: &mut EmitterUniform, clock: &Clock) {
        let current_sec = self.life_cycle.get_current_sec(clock);

        if !self.life_cycle.shoud_animate(current_sec) {
            return;
        }

        let fraction = self.life_cycle.get_fraction(current_sec);

        emitter.box_rotation.x = self.yaw.x + fraction * (self.yaw.y - self.yaw.x);
        emitter.box_rotation.y = self.pitch.x + fraction * (self.pitch.y - self.pitch.x);
        emitter.box_rotation.z = self.roll.x + fraction * (self.roll.y - self.roll.x);
    }

    fn reset_action(&mut self) {
        self.selected_action = ItemAction::None;
    }

    fn selected_action(&mut self) -> &mut ItemAction {
        &mut self.selected_action
    }

    fn create_gui(&mut self, ui: &mut Ui, gui_state: &GuiState) {
        let life_cycle = &mut self.life_cycle;
        let gui = &mut self.gui;

        gui_state.create_anim_header(ui, &mut self.selected_action, "Sway animation");

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
