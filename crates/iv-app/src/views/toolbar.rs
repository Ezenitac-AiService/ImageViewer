//! Core-only toolbars (`contracts/interaction-contract.md` §5).

use eframe::egui;
use iv_core::i18n::{I18n, Language};
use iv_core::interaction::{DialogKind, ExtendedFeatures, ViewerCommand};

/// State shown by the toolbars.
pub struct ToolbarState<'a> {
    pub lang: Language,
    pub zoom_percent: f64,
    pub position: Option<usize>,
    pub total: usize,
    pub title: &'a str,
    pub extended: ExtendedFeatures,
    pub slideshow_playing: bool,
    pub info_panel: bool,
}

/// Draws the top bar (file actions and dialogs). Returns the commands the user triggered.
pub fn top_bar(ui: &mut egui::Ui, state: &ToolbarState<'_>) -> Vec<ViewerCommand> {
    let tr = |key| I18n::tr(state.lang, key);
    let mut out = Vec::new();
    egui::Panel::top("iv_top_bar").show(ui, |ui| {
        ui.horizontal(|ui| {
            if ui.button(tr("menu.open_file")).clicked() {
                out.push(ViewerCommand::RequestOpenFile);
            }
            if ui.button(tr("menu.open_folder")).clicked() {
                out.push(ViewerCommand::RequestOpenFolder);
            }
            ui.separator();
            if state.extended.image_list && ui.button(tr("dialog.image_list")).clicked() {
                out.push(ViewerCommand::OpenDialog(DialogKind::ImageList));
            }
            if state.extended.info_panel
                && ui
                    .selectable_label(state.info_panel, tr("menu.exif"))
                    .clicked()
            {
                out.push(ViewerCommand::ToggleInfoPanel);
            }
            if ui.button(tr("dialog.settings")).clicked() {
                out.push(ViewerCommand::OpenDialog(DialogKind::Settings));
            }
            if ui.button(tr("dialog.about")).clicked() {
                out.push(ViewerCommand::OpenDialog(DialogKind::About));
            }
            ui.separator();
            ui.label(state.title);
        });
    });
    out
}

/// Draws the bottom bar (zoom and navigation). Returns the commands the user triggered.
pub fn bottom_bar(
    ui: &mut egui::Ui,
    state: &ToolbarState<'_>,
    viewport_center: (f64, f64),
) -> Vec<ViewerCommand> {
    let tr = |key| I18n::tr(state.lang, key);
    let mut out = Vec::new();
    egui::Panel::bottom("iv_bottom_bar").show(ui, |ui| {
        ui.horizontal(|ui| {
            if ui
                .button("－")
                .on_hover_text(tr("toolbar.zoom_out"))
                .clicked()
            {
                out.push(ViewerCommand::ZoomBy {
                    factor: 0.83,
                    anchor: viewport_center,
                });
            }
            if ui
                .button(format!("{:.0}%", state.zoom_percent))
                .on_hover_text(tr("toolbar.reset_view"))
                .clicked()
            {
                out.push(ViewerCommand::ResetView);
            }
            if ui
                .button("＋")
                .on_hover_text(tr("toolbar.zoom_in"))
                .clicked()
            {
                out.push(ViewerCommand::ZoomBy {
                    factor: 1.2,
                    anchor: viewport_center,
                });
            }
            ui.separator();
            if ui.button("◀").on_hover_text(tr("toolbar.prev")).clicked() {
                out.push(ViewerCommand::Prev);
            }
            let position = match state.position {
                Some(i) if state.total > 0 => format!("{} / {}", i + 1, state.total),
                _ => "0 / 0".to_string(),
            };
            ui.label(position);
            if ui.button("▶").on_hover_text(tr("toolbar.next")).clicked() {
                out.push(ViewerCommand::Next);
            }
            if state.extended.slideshow {
                ui.separator();
                let label = if state.slideshow_playing {
                    format!("⏸ {}", tr("menu.slideshow"))
                } else {
                    format!("▶ {}", tr("menu.slideshow"))
                };
                if ui
                    .selectable_label(state.slideshow_playing, label)
                    .clicked()
                {
                    out.push(ViewerCommand::ToggleSlideshow);
                }
            }
        });
    });
    out
}
