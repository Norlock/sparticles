use crate::ui::GuiState;
use crate::{
    model::{Clock, EmitterUniform, LifeCycle},
    traits::{EmitterAnimation, HandleAction, HandleAngles, RegisterEmitterAnimation},
    util::persistence::DynamicExport,
    util::ListAction,
};
use egui_winit::egui::{DragValue, Ui};
use glam::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Gui {
    yaw: Vec2,
    pitch: Vec2,
    roll: Vec2,
}

#[derive(Serialize, Deserialize)]
pub struct SwayAnimation {
    life_cycle: LifeCycle,
    yaw: Vec2,
    pitch: Vec2,
    roll: Vec2,
    gui: Gui,

    #[serde(skip_serializing, skip_deserializing)]
    selected_action: ListAction,

    enabled: bool,
}

#[derive(Clone, Copy)]
pub struct RegisterSwayAnimation;

impl RegisterEmitterAnimation for RegisterSwayAnimation {
    fn tag(&self) -> &'static str {
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
            yaw: yaw_deg,
            pitch: pitch_deg,
            roll: roll_deg,
        };

        Self {
            life_cycle,
            yaw: yaw_deg.to_radians(),
            pitch: pitch_deg.to_radians(),
            roll: roll_deg.to_radians(),
            selected_action: ListAction::None,
            enabled: true,
            gui,
        }
    }
}

impl HandleAction for SwayAnimation {
    fn selected_action(&mut self) -> &mut ListAction {
        &mut self.selected_action
    }

    fn export(&self) -> DynamicExport {
        DynamicExport {
            tag: RegisterSwayAnimation.tag().to_string(),
            data: serde_json::to_value(self).unwrap(),
        }
    }

    fn enabled(&self) -> bool {
        self.enabled
    }
}

impl EmitterAnimation for SwayAnimation {
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

    fn create_ui(&mut self, ui: &mut Ui, ui_state: &GuiState) {
        self.selected_action = ui_state.create_li_header(ui, "Sway animation");
        let life_cycle = &mut self.life_cycle;
        let gui = &mut self.gui;

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

        ui.checkbox(&mut self.enabled, "Enabled");

        self.yaw = gui.yaw.to_radians();
        self.pitch = gui.pitch.to_radians();
        self.roll = gui.pitch.to_radians();
    }
}
