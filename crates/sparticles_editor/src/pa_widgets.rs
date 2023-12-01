use crate::EditorData;
use sparticles_app::{
    animations::{ColorAnimation, ForceAnimation, GravityAnimation, StrayAnimation},
    gui::egui::{
        color_picker::{color_edit_button_rgba, Alpha},
        DragValue, Rgba, Slider, Ui,
    },
    traits::ParticleAnimation,
};

#[derive(Clone, Copy, Debug)]
pub struct EditorWidgets;

impl EditorWidgets {
    pub fn color_anim(editor: &mut EditorData, anim: &mut Box<dyn ParticleAnimation>, ui: &mut Ui) {
        let downcast = anim.as_any().downcast_mut::<ColorAnimation>();

        if let Some(anim) = downcast {
            anim.selected_action = editor.create_li_header(ui, "Color animation");

            let mut gui = anim.uniform;

            ui.horizontal(|ui| {
                ui.label("Animate from sec");
                ui.add(DragValue::new(&mut gui.from_sec).speed(0.1));
            });

            ui.horizontal(|ui| {
                ui.label("Animate until sec");
                ui.add(DragValue::new(&mut gui.until_sec).speed(0.1));
            });

            let f_col = gui.from_color;
            let t_col = gui.to_color;
            let mut from_color = Rgba::from_rgba_premultiplied(f_col.x, f_col.y, f_col.z, f_col.w);
            let mut to_color = Rgba::from_rgba_premultiplied(t_col.x, t_col.y, t_col.z, t_col.w);

            ui.horizontal(|ui| {
                ui.label("From color: ");
                color_edit_button_rgba(ui, &mut from_color, Alpha::Opaque);
            });

            ui.horizontal(|ui| {
                ui.label("To color: ");
                color_edit_button_rgba(ui, &mut to_color, Alpha::Opaque);
            });

            ui.checkbox(&mut anim.enabled, "Enabled");

            gui.from_color = from_color.to_array().into();
            gui.to_color = to_color.to_array().into();

            if anim.uniform != gui {
                anim.update_uniform = true;
                anim.uniform = gui;
            }
        }
    }

    pub fn gravity_anim(
        editor: &mut EditorData,
        anim: &mut Box<dyn ParticleAnimation>,
        ui: &mut Ui,
    ) {
        let downcast = anim.as_any().downcast_mut::<GravityAnimation>();

        if let Some(anim) = downcast {
            anim.selected_action = editor.create_li_header(ui, "Gravity animation");
            let mut gui = anim.uniform;

            ui.horizontal(|ui| {
                ui.label("Animate from sec");
                ui.add(DragValue::new(&mut gui.life_cycle.from_sec).speed(0.1));
            });

            ui.horizontal(|ui| {
                ui.label("Animate until sec");
                ui.add(DragValue::new(&mut gui.life_cycle.until_sec).speed(0.1));
            });

            ui.horizontal(|ui| {
                ui.label("Lifetime sec");
                ui.add(DragValue::new(&mut gui.life_cycle.lifetime_sec).speed(0.1));
            });

            ui.horizontal(|ui| {
                ui.label("Start position > ");
                ui.label("x:");
                ui.add(DragValue::new(&mut gui.start_pos.x).speed(0.1));
                ui.label("y:");
                ui.add(DragValue::new(&mut gui.start_pos.y).speed(0.1));
                ui.label("z:");
                ui.add(DragValue::new(&mut gui.start_pos.z).speed(0.1));
            });

            ui.horizontal(|ui| {
                ui.label("End position > ");
                ui.label("x:");
                ui.add(DragValue::new(&mut gui.end_pos.x).speed(0.1));
                ui.label("y:");
                ui.add(DragValue::new(&mut gui.end_pos.y).speed(0.1));
                ui.label("z:");
                ui.add(DragValue::new(&mut gui.end_pos.z).speed(0.1));
            });

            ui.horizontal(|ui| {
                ui.label("Dead zone");
                ui.add(DragValue::new(&mut gui.dead_zone).speed(0.1));
            });

            ui.horizontal(|ui| {
                ui.label("Gravitational force");
                ui.add(
                    DragValue::new(&mut gui.gravitational_force)
                        .speed(0.001)
                        .clamp_range(-0.02..=0.02),
                );
            });

            ui.checkbox(&mut anim.enabled, "Enabled");

            if anim.uniform != gui {
                anim.uniform = gui;
            }
        }
    }

    pub fn stray_anim(editor: &mut EditorData, anim: &mut Box<dyn ParticleAnimation>, ui: &mut Ui) {
        let downcast = anim.as_any().downcast_mut::<StrayAnimation>();

        if let Some(anim) = downcast {
            anim.selected_action = editor.create_li_header(ui, "Stray animation");

            let mut gui = anim.uniform;
            let mut stray_degrees = gui.stray_radians.to_degrees();

            ui.horizontal(|ui| {
                ui.label("Animate from sec");
                ui.add(DragValue::new(&mut gui.from_sec).speed(0.1));
            });

            ui.horizontal(|ui| {
                ui.label("Animate until sec");
                ui.add(DragValue::new(&mut gui.until_sec).speed(0.1));
            });

            ui.spacing_mut().slider_width = 200.0;

            ui.add(
                Slider::new(&mut stray_degrees, 0.0..=45.)
                    .step_by(0.1)
                    .text("Stray degrees"),
            );

            ui.checkbox(&mut anim.enabled, "Enabled");

            gui.stray_radians = stray_degrees.to_radians();

            if anim.uniform != gui {
                anim.update_uniform = true;
                anim.uniform = gui;
            }
        }
    }

    pub fn force_anim(editor: &mut EditorData, anim: &mut Box<dyn ParticleAnimation>, ui: &mut Ui) {
        let downcast = anim.as_any().downcast_mut::<ForceAnimation>();

        if let Some(anim) = downcast {
            anim.selected_action = editor.create_li_header(ui, "Force animation");

            let mut gui = anim.uniform;

            ui.horizontal(|ui| {
                ui.label("Animate from sec");
                ui.add(DragValue::new(&mut gui.life_cycle.from_sec).speed(0.1));
            });

            ui.horizontal(|ui| {
                ui.label("Animate until sec");
                ui.add(DragValue::new(&mut gui.life_cycle.until_sec).speed(0.1));
            });

            ui.horizontal(|ui| {
                ui.label("Lifetime sec");
                ui.add(DragValue::new(&mut gui.life_cycle.lifetime_sec).speed(0.1));
            });

            ui.horizontal(|ui| {
                ui.label("Force velocity > ");
                ui.label("x:");
                ui.add(DragValue::new(&mut gui.velocity.x).speed(0.1));
                ui.label("y:");
                ui.add(DragValue::new(&mut gui.velocity.y).speed(0.1));
                ui.label("z:");
                ui.add(DragValue::new(&mut gui.velocity.z).speed(0.1));
            });

            ui.horizontal(|ui| {
                ui.label("Mass applied per (1) unit length");
                ui.add(DragValue::new(&mut gui.mass_per_unit).speed(0.1));
            });

            ui.checkbox(&mut anim.enabled, "Enabled");

            if anim.uniform != gui {
                anim.update_uniform = true;
                anim.uniform = gui;
            }
        }
    }
}
