//! Image List dialog (an in-window `egui::Window`) with display sorting and unsupported dimming.

use eframe::egui::{self, Color32};
use iv_core::i18n::{I18n, Language};
use iv_core::natural_sort::NaturalSort;
use iv_core::settings::{SortDirection, SortField};
use iv_media::sources::MediaEntry;

pub enum ImageListAction {
    None,
    Select(usize),
    Close,
}

pub fn draw_image_list(
    ctx: &egui::Context,
    lang: Language,
    entries: &[MediaEntry],
    selected_index: Option<usize>,
    sort_field: &mut SortField,
    sort_dir: &mut SortDirection,
) -> ImageListAction {
    let tr = |key| I18n::tr(lang, key);
    let mut action = ImageListAction::None;
    let mut open = true;
    egui::Window::new(tr("dialog.image_list"))
        .id(egui::Id::unique("iv_image_list"))
        .open(&mut open)
        .collapsible(false)
        .default_size([420.0, 480.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(sort_field, SortField::Name, tr("image_list.name"));
                ui.selectable_value(sort_field, SortField::ModifiedDate, tr("image_list.date"));
                ui.selectable_value(sort_field, SortField::FileSize, tr("image_list.size"));
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
                    let selected = Some(orig_idx) == selected_index;
                    let label = if entry.is_supported {
                        egui::RichText::new(entry.name.as_str())
                    } else {
                        egui::RichText::new(format!(
                            "{} ({})",
                            entry.name,
                            tr("image_list.unsupported")
                        ))
                        .color(Color32::DARK_GRAY)
                    };
                    if ui.selectable_label(selected, label).clicked() && entry.is_supported {
                        action = ImageListAction::Select(orig_idx);
                    }
                }
            });
        });
    if !open {
        action = ImageListAction::Close;
    }
    action
}
