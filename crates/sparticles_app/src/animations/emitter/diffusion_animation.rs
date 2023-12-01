use std::any::Any;

use crate::{
    model::{Clock, EmitterUniform, LifeCycle},
    traits::{EmitterAnimation, HandleAction, HandleAngles, RegisterEmitterAnimation},
    util::persistence::DynamicExport,
    util::ListAction,
};
use glam::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Gui {
    pub diff_width: Vec2,
    pub diff_depth: Vec2,
}

#[derive(Serialize, Deserialize)]
pub struct DiffusionAnimation {
    pub life_cycle: LifeCycle,
    pub diff_width: Vec2,
    pub diff_depth: Vec2,
    pub gui: Gui,

    #[serde(skip_serializing, skip_deserializing)]
    pub selected_action: ListAction,

    pub enabled: bool,
}

#[derive(Clone, Copy)]
pub struct RegisterDiffusionAnimation;

impl RegisterEmitterAnimation for RegisterDiffusionAnimation {
    fn tag(&self) -> &'static str {
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
    pub fn new(life_cycle: LifeCycle, diff_width_deg: Vec2, diff_depth_deg: Vec2) -> Self {
        let gui = Gui {
            diff_width: diff_width_deg,
            diff_depth: diff_depth_deg,
        };

        Self {
            life_cycle,
            diff_width: diff_width_deg.to_radians(),
            diff_depth: diff_depth_deg.to_radians(),
            selected_action: ListAction::None,
            gui,
            enabled: true,
        }
    }
}

impl HandleAction for DiffusionAnimation {
    fn selected_action(&mut self) -> &mut ListAction {
        &mut self.selected_action
    }

    fn export(&self) -> DynamicExport {
        DynamicExport {
            tag: RegisterDiffusionAnimation.tag().to_string(),
            data: serde_json::to_value(self).unwrap(),
        }
    }

    fn enabled(&self) -> bool {
        self.enabled
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

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}
