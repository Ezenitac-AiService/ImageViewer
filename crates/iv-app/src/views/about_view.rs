//! Separate OS child-window About dialog (`T042`).

use iv_core::i18n::{I18n, Language};

pub fn draw_about_window(ctx: &egui::Context, lang: Language) -> bool {
    let mut check_updates_clicked = false;
    let info_text = format!(
        "ImageViewer v{}\nOS: {} ({})\nCore: iv-core, iv-media (zune-jpeg, image, fast_image_resize, moxcms, jxl-oxide), iv-platform, iv-app (winit, wgpu, egui)",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading(I18n::tr(lang, "dialog.about"));
        ui.separator();
        ui.label(&info_text);
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button(I18n::tr(lang, "about.copy")).clicked()
                && let Ok(mut cb) = arboard::Clipboard::new()
            {
                let _ = cb.set_text(info_text.clone());
            }
            if ui.button(I18n::tr(lang, "about.check_updates")).clicked() {
                check_updates_clicked = true;
            }
        });
    });
    check_updates_clicked
}
