//! Status overlays: loading indicator, one-off notices, and error messages.

use eframe::egui::{self, Align2, Color32};

/// A message shown over the canvas until replaced or dismissed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusMessage {
    Info(String),
    Error(String),
}

/// Draws the loading indicator in the middle of the canvas.
pub fn loading(ctx: &egui::Context, label: &str) {
    egui::Area::new(egui::Id::unique("iv_loading"))
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .interactable(false)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(label);
                });
            });
        });
}

/// Draws a message near the bottom of the canvas. Returns `true` when the user dismissed it.
pub fn message(ctx: &egui::Context, msg: &StatusMessage) -> bool {
    let mut dismissed = false;
    egui::Area::new(egui::Id::unique("iv_status"))
        .anchor(Align2::CENTER_BOTTOM, [0.0, -48.0])
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.horizontal(|ui| {
                    match msg {
                        StatusMessage::Info(text) => {
                            ui.label(text.as_str());
                        }
                        StatusMessage::Error(text) => {
                            ui.colored_label(Color32::LIGHT_RED, text.as_str());
                        }
                    }
                    if ui.small_button("✕").clicked() {
                        dismissed = true;
                    }
                });
            });
        });
    dismissed
}

/// Draws a centred hint on an empty viewer.
pub fn empty_hint(ctx: &egui::Context, label: &str) {
    egui::Area::new(egui::Id::unique("iv_empty"))
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .interactable(false)
        .show(ctx, |ui| {
            ui.label(egui::RichText::new(label).size(18.0).color(Color32::GRAY));
        });
}
