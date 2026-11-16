//! Separate OS child-window Image List dialog (`Enter`) with sorting & unsupported dimming.

use egui::Color32;
use iv_core::i18n::{I18n, Language};
use iv_core::natural_sort::NaturalSort;
use iv_core::settings::{SortDirection, SortField};
use iv_media::sources::MediaEntry;

pub fn draw_image_list(
    ctx: &egui::Context,
    lang: Language,
    entries: &[MediaEntry],
    selected_index: usize,
    sort_field: &mut SortField,
    sort_dir: &mut SortDirection,
) -> Option<usize> {
    let mut clicked = None;
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading(I18n::tr(lang, "dialog.image_list"));
            ui.separator();
            ui.selectable_value(sort_field, SortField::Name, "Name");
            ui.selectable_value(sort_field, SortField::ModifiedDate, "Date");
            ui.selectable_value(sort_field, SortField::FileSize, "Size");
            let dir_lbl = match sort_dir {
                SortDirection::Ascending => "↑",
                SortDirection::Descending => "↓",
            };
            if ui.button(dir_lbl).clicked() {
                *sort_dir = match sort_dir {
                    SortDirection::Ascending => SortDirection::Descending,
                    SortDirection::Descending => SortDirection::Ascending,
                };
            }
        });
        ui.separator();

        let mut indexed: Vec<(usize, &MediaEntry)> = entries.iter().enumerate().collect();
        indexed.sort_by(|a, b| {
            let ord = match *sort_field {
                SortField::Name => NaturalSort::cmp(&a.1.name, &b.1.name),
                SortField::ModifiedDate => a.1.modified_utc.cmp(&b.1.modified_utc),
                SortField::FileSize => a.1.byte_size.cmp(&b.1.byte_size),
            };
            match *sort_dir {
                SortDirection::Ascending => ord,
                SortDirection::Descending => ord.reverse(),
            }
        });

        egui::ScrollArea::vertical().show(ui, |ui| {
            for (orig_idx, entry) in indexed {
                let selected = orig_idx == selected_index;
                let label = if entry.is_supported {
                    egui::RichText::new(format!("{} ({} B)", entry.name, entry.byte_size))
                } else {
                    egui::RichText::new(format!("{} (unsupported)", entry.name))
                        .color(Color32::DARK_GRAY)
                };
                if ui.selectable_label(selected, label).clicked() && entry.is_supported {
                    clicked = Some(orig_idx);
                }
            }
        });
    });
    clicked
}
