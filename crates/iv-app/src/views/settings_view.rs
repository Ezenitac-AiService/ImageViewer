//! Separate OS child-window Settings dialog (`T042`).

use iv_core::i18n::I18n;
use iv_core::settings::{MouseWheelAction, ViewerBackground, ViewerSettings};

pub enum SettingsDialogAction {
    None,
    SaveAndClose,
    Cancel,
}

pub fn draw_settings_window(
    ctx: &egui::Context,
    settings: &mut ViewerSettings,
) -> SettingsDialogAction {
    let mut action = SettingsDialogAction::None;
    let lang = settings.language;
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading(I18n::tr(lang, "dialog.settings"));
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Background:");
            egui::ComboBox::from_id_salt("bg_combo")
                .selected_text(format!("{:?}", settings.display.background))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut settings.display.background,
                        ViewerBackground::Black,
                        "Black",
                    );
                    ui.selectable_value(
                        &mut settings.display.background,
                        ViewerBackground::DarkGray,
                        "DarkGray",
                    );
                    ui.selectable_value(
                        &mut settings.display.background,
                        ViewerBackground::White,
                        "White",
                    );
                    ui.selectable_value(
                        &mut settings.display.background,
                        ViewerBackground::Checkerboard,
                        "Checkerboard",
                    );
                });
        });

        ui.horizontal(|ui| {
            ui.label("Mouse Wheel:");
            ui.radio_value(
                &mut settings.display.mouse_wheel,
                MouseWheelAction::Navigate,
                "Navigate",
            );
            ui.radio_value(
                &mut settings.display.mouse_wheel,
                MouseWheelAction::Zoom,
                "Zoom",
            );
        });

        ui.checkbox(&mut settings.navigation.wrap_around, "Wrap Around");
        ui.checkbox(&mut settings.navigation.watch_folder, "Watch Folder");
        ui.checkbox(&mut settings.slideshow.repeat, "Slideshow Repeat");
        ui.checkbox(&mut settings.thumbnail.visible, "Show Thumbnails");
        ui.checkbox(
            &mut settings.thumbnail.show_scroll_bar,
            "Thumbnail Scrollbar",
        );
        ui.add(egui::Slider::new(&mut settings.thumbnail.size, 48..=256).text("Thumbnail Size"));
        ui.add(
            egui::Slider::new(&mut settings.filter.smooth_sigma, 0.1..=2.0).text("Smooth Sigma"),
        );
        ui.add(
            egui::Slider::new(&mut settings.filter.sharpen_sigma, 0.1..=3.0).text("Sharpen Sigma"),
        );
        ui.add(
            egui::Slider::new(&mut settings.filter.sharpen_amount, 0.1..=3.0)
                .text("Sharpen Amount"),
        );
        ui.add(
            egui::Slider::new(&mut settings.limits.max_image_megapixels, 10..=500)
                .text("Max Megapixels"),
        );
        ui.checkbox(&mut settings.update.auto_check, "Auto Check Updates");

        ui.separator();
        ui.horizontal(|ui| {
            if ui.button(I18n::tr(lang, "settings.ok")).clicked() {
                action = SettingsDialogAction::SaveAndClose;
            }
            if ui.button(I18n::tr(lang, "settings.cancel")).clicked() {
                action = SettingsDialogAction::Cancel;
            }
        });
    });
    action
}
