use sparticles_app::{
    animations::{DiffusionAnimation, SwayAnimation},
    gui::egui::{DragValue, Ui},
    traits::{EmitterAnimation, HandleAngles},
};

use crate::{EditorData, EditorWidgets};

impl EditorWidgets {
    pub fn sway_anim(editor: &mut EditorData, anim: &mut Box<dyn EmitterAnimation>, ui: &mut Ui) {
        let downcast = anim.as_any().downcast_mut::<SwayAnimation>();

        if let Some(anim) = downcast {
            anim.selected_action = editor.create_li_header(ui, "Sway animation");
            let life_cycle = &mut anim.life_cycle;
            let gui = &mut anim.gui;

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

            ui.checkbox(&mut anim.enabled, "Enabled");

            anim.yaw = gui.yaw.to_radians();
            anim.pitch = gui.pitch.to_radians();
            anim.roll = gui.pitch.to_radians();
        }
    }

    pub fn diffusion_anim(
        editor: &mut EditorData,
        anim: &mut Box<dyn EmitterAnimation>,
        ui: &mut Ui,
    ) {
        let downcast = anim.as_any().downcast_mut::<DiffusionAnimation>();

        if let Some(anim) = downcast {
            anim.selected_action = editor.create_li_header(ui, "Diffusion animation");

            let life_cycle = &mut anim.life_cycle;
            let gui = &mut anim.gui;

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

            ui.checkbox(&mut anim.enabled, "Enabled");

            anim.diff_width = gui.diff_width.to_radians();
            anim.diff_depth = gui.diff_depth.to_radians();
        }
    }
}
