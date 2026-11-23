//! Settings dialog (an in-window `egui::Window`). Every label is localized (FR-018).

use eframe::egui;
use iv_core::i18n::{I18n, Language};
use iv_core::settings::{MouseWheelAction, ViewerBackground, ViewerSettings};

pub enum SettingsDialogAction {
    None,
    SaveAndClose,
    Cancel,
}

/// Draws the settings dialog over `draft`. The caller owns the draft so Cancel discards it.
pub fn draw_settings_window(
    ctx: &egui::Context,
    draft: &mut ViewerSettings,
) -> SettingsDialogAction {
    let lang = draft.language;
    let tr = |key| I18n::tr(lang, key);
    let mut action = SettingsDialogAction::None;
    let mut open = true;
    egui::Window::new(tr("dialog.settings"))
        .id(egui::Id::unique("iv_settings"))
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            egui::Grid::new("iv_settings_grid")
                .num_columns(2)
                .spacing([16.0, 8.0])
                .show(ui, |ui| {
                    ui.label(tr("settings.language"));
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut draft.language, Language::Ko, "한국어");
                        ui.radio_value(&mut draft.language, Language::En, "English");
                    });
                    ui.end_row();

                    ui.label(tr("settings.background"));
                    egui::ComboBox::from_id_salt("iv_bg_combo")
                        .selected_text(background_label(lang, draft.display.background))
                        .show_ui(ui, |ui| {
                            for bg in [
                                ViewerBackground::Black,
                                ViewerBackground::DarkGray,
                                ViewerBackground::White,
                                ViewerBackground::Checkerboard,
                            ] {
                                ui.selectable_value(
                                    &mut draft.display.background,
                                    bg,
                                    background_label(lang, bg),
                                );
                            }
                        });
                    ui.end_row();

                    ui.label(tr("settings.mouse_wheel"));
                    ui.horizontal(|ui| {
                        ui.radio_value(
                            &mut draft.display.mouse_wheel,
                            MouseWheelAction::Navigate,
                            tr("settings.wheel_navigate"),
                        );
                        ui.radio_value(
                            &mut draft.display.mouse_wheel,
                            MouseWheelAction::Zoom,
                            tr("settings.wheel_zoom"),
                        );
                    });
                    ui.end_row();

                    ui.label(tr("settings.slideshow_interval"));
                    ui.add(egui::Slider::new(
                        &mut draft.slideshow.interval_seconds,
                        1..=60,
                    ));
                    ui.end_row();

                    ui.label(tr("settings.max_megapixels"));
                    ui.add(egui::Slider::new(
                        &mut draft.limits.max_image_megapixels,
                        10..=1000,
                    ));
                    ui.end_row();
                });
            ui.checkbox(
                &mut draft.navigation.wrap_around,
                tr("settings.wrap_around"),
            );
            ui.checkbox(
                &mut draft.navigation.watch_folder,
                tr("settings.watch_folder"),
            );
            ui.checkbox(
                &mut draft.display.play_animations,
                tr("settings.play_animations"),
            );
            ui.checkbox(&mut draft.slideshow.repeat, tr("settings.slideshow_repeat"));

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button(tr("settings.ok")).clicked() {
                    action = SettingsDialogAction::SaveAndClose;
                }
                if ui.button(tr("settings.cancel")).clicked() {
                    action = SettingsDialogAction::Cancel;
                }
            });
        });
    if !open {
        action = SettingsDialogAction::Cancel;
    }
    action
}

fn background_label(lang: Language, bg: ViewerBackground) -> &'static str {
    let key = match bg {
        ViewerBackground::Black => "settings.bg_black",
        ViewerBackground::DarkGray => "settings.bg_dark_gray",
        ViewerBackground::White => "settings.bg_white",
        ViewerBackground::Checkerboard => "settings.bg_checkerboard",
    };
    I18n::tr(lang, key)
}
