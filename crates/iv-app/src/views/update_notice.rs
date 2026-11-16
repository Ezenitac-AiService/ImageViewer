//! Top-right floating UpdateNotice overlay (`T042`).

use egui::Align2;
use iv_core::i18n::{I18n, Language};
use iv_platform::update::ReleaseInfo;

#[allow(dead_code)]
pub enum UpdateNoticeAction {
    None,
    SkipVersion(String),
    Dismiss,
    Install,
}

#[allow(dead_code)]
pub fn draw_update_notice(
    ctx: &egui::Context,
    lang: Language,
    release: &ReleaseInfo,
    progress: Option<f32>,
) -> UpdateNoticeAction {
    let mut action = UpdateNoticeAction::None;
    egui::Area::new(egui::Id::new("iv_update_notice"))
        .anchor(Align2::RIGHT_TOP, [-16.0, 44.0])
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.strong(format!(
                    "{}: v{}",
                    I18n::tr(lang, "update.available"),
                    release.version
                ));
                if let Some(p) = progress {
                    ui.add(egui::ProgressBar::new(p).show_percentage());
                }
                ui.horizontal(|ui| {
                    if ui.button(I18n::tr(lang, "update.install")).clicked() {
                        action = UpdateNoticeAction::Install;
                    }
                    if ui.button(I18n::tr(lang, "update.skip")).clicked() {
                        action = UpdateNoticeAction::SkipVersion(release.version.clone());
                    }
                    if ui.button(I18n::tr(lang, "update.dismiss")).clicked() {
                        action = UpdateNoticeAction::Dismiss;
                    }
                });
            });
        });
    action
}
