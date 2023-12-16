use sparticles_app::{
    fx::{
        bloom::UIAction,
        blur::{BlurEvent, BlurFx, BlurType},
        color::UpdateAction,
        BloomFx, ColorFx,
    },
    gui::egui::{self, Slider, Ui},
    model::TonemapType,
    traits::PostFx,
};

use crate::{EditorData, EditorWidgets};

impl EditorWidgets {
    pub fn bloom_fx(editor: &mut EditorData, post_fx: &mut Box<dyn PostFx>, ui: &mut Ui) {
        let downcast = post_fx.as_any().downcast_mut::<BloomFx>();

        if let Some(bloom) = downcast {
            bloom.selected_action = editor.create_li_header(ui, "Bloom settings");
            ui.add_space(5.0);

            ui.add(Slider::new(&mut bloom.bloom_treshold, 0.0..=10.0).text("Brightness treshold"));

            for (i, up) in bloom.upscale_passes.iter_mut().enumerate() {
                let io_uniform = up.blend.io();
                let text = format!(
                    "IO mix from downscale {} to {}",
                    io_uniform.in_downscale, io_uniform.out_downscale
                );

                if ui
                    .add(Slider::new(&mut up.blend_uniform.io_mix, 0.0..=1.0).text(&text))
                    .changed()
                {
                    bloom.update_event = Some(UIAction::UpdateBuffer(i));
                }
            }

            editor.create_title(ui, "Blend");

            if ui
                .add(
                    Slider::new(&mut bloom.blend_uniform.io_mix, 0.0..=1.0)
                        .text("IO mix bloom to frame"),
                )
                .changed()
            {
                bloom.update_event = Some(UIAction::UpdateBuffer(bloom.upscale_passes.len()));
            }

            editor.create_title(ui, "Color correction");

            Self::gamma_widget(&mut bloom.color, ui);

            let color_uniform = &mut bloom.color.color_uniform;

            ui.add_space(6.);

            ui.horizontal_top(|ui| {
                egui::ComboBox::from_label("tonemapping")
                    .selected_text(TonemapType::from(color_uniform.tonemap))
                    .show_ui(ui, |ui| {
                        let mut tonemap_option = |tonemap_type: TonemapType| {
                            if ui
                                .selectable_value(
                                    &mut color_uniform.tonemap,
                                    tonemap_type.into(),
                                    tonemap_type,
                                )
                                .changed()
                            {
                                bloom.color.update_event = Some(UpdateAction::UpdateBuffer);
                            }
                        };

                        tonemap_option(TonemapType::AcesNarkowicz);
                        tonemap_option(TonemapType::AcesHill);
                        tonemap_option(TonemapType::Uchimura);
                        tonemap_option(TonemapType::Lottes);
                    });
            });

            ui.add_space(6.);

            ui.checkbox(&mut bloom.enabled, "Enabled");
        }
    }

    pub fn gamma_widget(color_fx: &mut ColorFx, ui: &mut Ui) {
        if ui
            .add(Slider::new(&mut color_fx.color_uniform.gamma, 0.1..=4.0).text("Gamma"))
            .changed()
        {
            color_fx.update_event = Some(UpdateAction::UpdateBuffer)
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
