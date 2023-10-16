use egui_winit::egui::{DragValue, Ui};
use serde::{Deserialize, Serialize};

use crate::{
    animations::ItemAction,
    math::SparVec2,
    model::{Clock, EmitterUniform, GuiState, LifeCycle},
    traits::{EmitterAnimation, HandleAction, HandleAngles, RegisterEmitterAnimation},
    util::persistence::DynamicExport,
};

#[derive(Serialize, Deserialize)]
struct Gui {
    diff_width: SparVec2,
    diff_depth: SparVec2,
}

#[derive(Serialize, Deserialize)]
pub struct DiffusionAnimation {
    life_cycle: LifeCycle,
    diff_width: SparVec2,
    diff_depth: SparVec2,
    gui: Gui,

    #[serde(skip_serializing, skip_deserializing)]
    selected_action: ItemAction,
}

#[derive(Clone, Copy)]
pub struct RegisterDiffusionAnimation;

impl RegisterEmitterAnimation for RegisterDiffusionAnimation {
    fn tag(&self) -> &str {
        "diffusion"
    }

    fn import(&self, value: serde_json::Value) -> Box<dyn EmitterAnimation> {
        let anim: DiffusionAnimation = serde_json::from_value(value).unwrap();
        Box::new(anim)
    }

    fn create_default(&self) -> Box<dyn EmitterAnimation> {
        let diff_anim = DiffusionAnimation::new(
            LifeCycle {
                from_sec: 0.,
                until_sec: 5.,
                lifetime_sec: 5.,
            },
            [0., 45.].into(),
            [0., 15.].into(),
        );

        Box::new(diff_anim)
    }
}

impl DiffusionAnimation {
    pub fn new(life_cycle: LifeCycle, diff_width_deg: SparVec2, diff_depth_deg: SparVec2) -> Self {
        let gui = Gui {
            diff_width: diff_width_deg,
            diff_depth: diff_depth_deg,
        };

        Self {
            life_cycle,
            diff_width: diff_width_deg.to_radians(),
            diff_depth: diff_depth_deg.to_radians(),
            selected_action: ItemAction::None,
            gui,
        }
    }
}

impl HandleAction for DiffusionAnimation {
    fn reset_action(&mut self) {
        self.selected_action = ItemAction::None;
    }

    fn selected_action(&mut self) -> &mut ItemAction {
        &mut self.selected_action
    }

    fn export(&self) -> DynamicExport {
        DynamicExport {
            tag: RegisterDiffusionAnimation.tag().to_string(),
            data: serde_json::to_value(self).unwrap(),
        }
    }

    fn enabled(&self) -> bool {
        todo!()
    }
}

impl EmitterAnimation for DiffusionAnimation {
    fn animate(&mut self, emitter: &mut EmitterUniform, clock: &Clock) {
        let current_sec = self.life_cycle.get_current_sec(clock);

        if !self.life_cycle.shoud_animate(current_sec) {
            return;
        }

        let fraction = self.life_cycle.get_fraction(current_sec);

        emitter.diff_width = self.diff_width.x + fraction * (self.diff_width.y - self.diff_width.x);
        emitter.diff_depth = self.diff_depth.x + fraction * (self.diff_depth.y - self.diff_depth.x);
    }

    fn create_ui(&mut self, ui: &mut Ui, ui_state: &GuiState) {
        self.selected_action = ui_state.create_anim_header(ui, "Diffusion animation");

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
            ui.label("Diffusion width (from - until)");
            ui.add(DragValue::new(&mut gui.diff_width.x).speed(0.1));
            ui.add(DragValue::new(&mut gui.diff_width.y).speed(0.1));
        });

        ui.horizontal(|ui| {
            ui.label("Diffusion depth (from - until)");
            ui.add(DragValue::new(&mut gui.diff_depth.x).speed(0.1));
            ui.add(DragValue::new(&mut gui.diff_depth.y).speed(0.1));
        });

        self.diff_width = gui.diff_width.to_radians();
        self.diff_depth = gui.diff_depth.to_radians();
    }
}
