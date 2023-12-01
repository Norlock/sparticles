use sparticles_app::{
    fx::{
        bloom::UIAction,
        blur::{BlurEvent, BlurFx, BlurType},
        color::UpdateAction,
        BloomFx, ColorFx,
    },
    gui::egui::{Slider, Ui},
    traits::PostFx,
};

use crate::{EditorData, EditorWidgets};

impl EditorWidgets {
    pub fn bloom_fx(editor: &mut EditorData, anim: &mut Box<dyn PostFx>, ui: &mut Ui) {
        let downcast = anim.as_any().downcast_mut::<BloomFx>();

        if let Some(anim) = downcast {
            anim.selected_action = editor.create_li_header(ui, "Bloom settings");
            ui.add_space(5.0);

            ui.add(Slider::new(&mut anim.bloom_treshold, 0.0..=10.0).text("Brightness treshold"));

            for (i, up) in anim.upscale_passes.iter_mut().enumerate() {
                let io_uniform = up.blend.io();
                let text = format!(
                    "IO mix from downscale {} to {}",
                    io_uniform.in_downscale, io_uniform.out_downscale
                );

                if ui
                    .add(Slider::new(&mut up.blend_uniform.io_mix, 0.0..=1.0).text(&text))
                    .changed()
                {
                    anim.update_event = Some(UIAction::UpdateBuffer(i));
                }
            }

            editor.create_title(ui, "Blend");
            if ui
                .add(
                    Slider::new(&mut anim.blend_uniform.io_mix, 0.0..=1.0)
                        .text("IO mix bloom to frame"),
                )
                .changed()
            {
                anim.update_event = Some(UIAction::UpdateBuffer(anim.upscale_passes.len()));
            }

            editor.create_title(ui, "Color correction");
            // TODO refactor
            ui.add(Slider::new(&mut anim.color.color_uniform.gamma, 0.1..=4.0).text("Gamma"))
                .changed()
                .then(|| anim.color.update_event = Some(UpdateAction::UpdateBuffer));

            ui.checkbox(&mut anim.enabled, "Enabled");
        }
    }

    pub fn blur_fx(editor: &mut EditorData, anim: &mut Box<dyn PostFx>, ui: &mut Ui) {
        let downcast = anim.as_any().downcast_mut::<BlurFx>();

        if let Some(anim) = downcast {
            anim.selected_action = editor.create_li_header(ui, "Gaussian blur");

            if anim.blur_type == BlurType::Gaussian {
                let a =
                    ui.add(Slider::new(&mut anim.blur_uniform.sigma, 0.1..=3.0).text("Blur sigma"));
                let b =
                    ui.add(Slider::new(&mut anim.blur_uniform.radius, 2..=6).text("Blur radius"));
                let c = ui.add(
                    Slider::new(&mut anim.blur_uniform.intensity, 0.9..=1.1).text("Blur intensity"),
                );

                if a.changed() || b.changed() || c.changed() {
                    anim.update_uniform = Some(BlurEvent::UpdateUniform);
                }
            } else {
                let a = ui.add(
                    Slider::new(&mut anim.blur_uniform.brightness_threshold, 0.0..=1.0)
                        .text("Brightness threshold"),
                );
                let b =
                    ui.add(Slider::new(&mut anim.blur_uniform.sigma, 0.1..=3.0).text("Blur sigma"));
                let c =
                    ui.add(Slider::new(&mut anim.blur_uniform.radius, 2..=8).text("Blur radius"));
                let d = ui.add(
                    Slider::new(&mut anim.blur_uniform.intensity, 0.9..=1.1).text("Blur intensity"),
                );

                if a.changed() || b.changed() || c.changed() || d.changed() {
                    anim.update_uniform = Some(BlurEvent::UpdateUniform);
                }
            }

            ui.checkbox(&mut anim.enabled, "Enabled");
        }
    }

    pub fn color_fx(editor: &mut EditorData, anim: &mut Box<dyn PostFx>, ui: &mut Ui) {
        let downcast = anim.as_any().downcast_mut::<ColorFx>();

        if let Some(anim) = downcast {
            anim.selected_action = editor.create_li_header(ui, "Color correction");

            ui.add(Slider::new(&mut anim.color_uniform.gamma, 0.1..=4.0).text("Gamma"))
                .changed()
                .then(|| anim.update_event = Some(UpdateAction::UpdateBuffer));

            ui.add(Slider::new(&mut anim.color_uniform.contrast, 0.1..=4.0).text("Contrast"))
                .changed()
                .then(|| anim.update_event = Some(UpdateAction::UpdateBuffer));

            ui.add(Slider::new(&mut anim.color_uniform.brightness, 0.01..=1.0).text("Brightness"))
                .changed()
                .then(|| anim.update_event = Some(UpdateAction::UpdateBuffer));

            ui.checkbox(&mut anim.enabled, "Enabled");
        }
    }
}
