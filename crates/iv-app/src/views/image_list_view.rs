//! Separate OS child-window Image List dialog (`Enter`).

use iv_core::i18n::{I18n, Language};
use iv_media::sources::MediaEntry;

pub fn draw_image_list(
    ctx: &egui::Context,
    lang: Language,
    entries: &[MediaEntry],
    selected_index: usize,
) -> Option<usize> {
    let mut clicked = None;
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading(I18n::tr(lang, "dialog.image_list"));
        ui.separator();
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (idx, entry) in entries.iter().enumerate() {
                let selected = idx == selected_index;
                if ui.selectable_label(selected, &entry.name).clicked() {
                    clicked = Some(idx);
                }
            }
        });
    });
    clicked
}
