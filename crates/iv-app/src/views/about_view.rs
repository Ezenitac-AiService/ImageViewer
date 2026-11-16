//! Separate OS child-window About dialog (`T042`).

use iv_core::i18n::{I18n, Language};

pub fn about_info_text() -> String {
    format!(
        "ImageViewer v{}\n\
OS: {} ({})\n\
License: GPL-3.0-only (GNU GPLv3)\n\
Copyright (C) edith2580\n\
Copyright (C) 2026 Ezenitac-AiService\n\
Repository: https://github.com/Ezenitac-AiService/ImageViewer\n\
Upstream Origin: edith2580/ImageViewer (v1.0.6)\n\
GPLv3 §5(d) Notice: Independent cross-platform Rust fork modified on 2026-11-16.\n\
Core & Third-Party Frameworks: winit, wgpu, egui, zune-jpeg, image, fast_image_resize, moxcms, jxl-oxide, exr, image-webp, zip, reqwest, rustls, sha2, rfd",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH
    )
}

pub fn draw_about_window(ctx: &egui::Context, lang: Language) -> bool {
    let mut check_updates_clicked = false;
    let info_text = about_info_text();
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

#[cfg(test)]
mod tests {
    use super::about_info_text;

    #[test]
    fn about_info_contains_license_origin_and_credits() {
        let text = about_info_text();
        assert!(text.contains("GPL-3.0-only (GNU GPLv3)"));
        assert!(text.contains("https://github.com/Ezenitac-AiService/ImageViewer"));
        assert!(text.contains("edith2580/ImageViewer"));
        assert!(text.contains("2026-11-16"));
        assert!(text.contains("winit"));
        assert!(text.contains("wgpu"));
        assert!(text.contains("egui"));
        assert!(text.contains("zune-jpeg"));
    }
}
