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
    pub fn bloom_fx(editor: &mut EditorData, post_fx: &mut Box<dyn PostFx>, ui: &mut Ui) {
        let downcast = post_fx.as_any().downcast_mut::<BloomFx>();

        if let Some(post_fx) = downcast {
            post_fx.selected_action = editor.create_li_header(ui, "Bloom settings");
            ui.add_space(5.0);

            ui.add(
                Slider::new(&mut post_fx.bloom_treshold, 0.0..=10.0).text("Brightness treshold"),
            );

            for (i, up) in post_fx.upscale_passes.iter_mut().enumerate() {
                let io_uniform = up.blend.io();
                let text = format!(
                    "IO mix from downscale {} to {}",
                    io_uniform.in_downscale, io_uniform.out_downscale
                );

                if ui
                    .add(Slider::new(&mut up.blend_uniform.io_mix, 0.0..=1.0).text(&text))
                    .changed()
                {
                    post_fx.update_event = Some(UIAction::UpdateBuffer(i));
                }
            }

            editor.create_title(ui, "Blend");

            if ui
                .add(
                    Slider::new(&mut post_fx.blend_uniform.io_mix, 0.0..=1.0)
                        .text("IO mix bloom to frame"),
                )
                .changed()
            {
                post_fx.update_event = Some(UIAction::UpdateBuffer(post_fx.upscale_passes.len()));
            }

            editor.create_title(ui, "Color correction");

            Self::gamma_widget(&mut post_fx.color, ui);

            ui.checkbox(&mut post_fx.enabled, "Enabled");
        }
    }

    pub fn gamma_widget(post_fx: &mut ColorFx, ui: &mut Ui) {
        if ui
            .add(Slider::new(&mut post_fx.color_uniform.gamma, 0.1..=4.0).text("Gamma"))
            .changed()
        {
            post_fx.update_event = Some(UpdateAction::UpdateBuffer)
        }
    }

    pub fn blur_fx(editor: &mut EditorData, post_fx: &mut Box<dyn PostFx>, ui: &mut Ui) {
        let downcast = post_fx.as_any().downcast_mut::<BlurFx>();

        if let Some(post_fx) = downcast {
            post_fx.selected_action = editor.create_li_header(ui, "Gaussian blur");

            if post_fx.blur_type == BlurType::Gaussian {
                let a = ui.add(
                    Slider::new(&mut post_fx.blur_uniform.sigma, 0.1..=3.0).text("Blur sigma"),
                );
                let b = ui
                    .add(Slider::new(&mut post_fx.blur_uniform.radius, 2..=6).text("Blur radius"));
                let c = ui.add(
                    Slider::new(&mut post_fx.blur_uniform.intensity, 0.9..=1.1)
                        .text("Blur intensity"),
                );

                if a.changed() || b.changed() || c.changed() {
                    post_fx.update_uniform = Some(BlurEvent::UpdateUniform);
                }
            } else {
                let a = ui.add(
                    Slider::new(&mut post_fx.blur_uniform.brightness_threshold, 0.0..=1.0)
                        .text("Brightness threshold"),
                );
                let b = ui.add(
                    Slider::new(&mut post_fx.blur_uniform.sigma, 0.1..=3.0).text("Blur sigma"),
                );
                let c = ui
                    .add(Slider::new(&mut post_fx.blur_uniform.radius, 2..=8).text("Blur radius"));
                let d = ui.add(
                    Slider::new(&mut post_fx.blur_uniform.intensity, 0.9..=1.1)
                        .text("Blur intensity"),
                );

                if a.changed() || b.changed() || c.changed() || d.changed() {
                    post_fx.update_uniform = Some(BlurEvent::UpdateUniform);
                }
            }

            ui.checkbox(&mut post_fx.enabled, "Enabled");
        }
    }

    pub fn color_fx(editor: &mut EditorData, post_fx: &mut Box<dyn PostFx>, ui: &mut Ui) {
        let downcast = post_fx.as_any().downcast_mut::<ColorFx>();

        if let Some(post_fx) = downcast {
            post_fx.selected_action = editor.create_li_header(ui, "Color correction");

            Self::gamma_widget(post_fx, ui);

            ui.add(Slider::new(&mut post_fx.color_uniform.contrast, 0.1..=4.0).text("Contrast"))
                .changed()
                .then(|| post_fx.update_event = Some(UpdateAction::UpdateBuffer));

            ui.add(
                Slider::new(&mut post_fx.color_uniform.brightness, 0.01..=1.0).text("Brightness"),
            )
            .changed()
            .then(|| post_fx.update_event = Some(UpdateAction::UpdateBuffer));

            ui.checkbox(&mut post_fx.enabled, "Enabled");
        }
    }
}
