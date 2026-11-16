//! About dialog (an in-window `egui::Window`) with license, origin and framework credits.

use eframe::egui;
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
Core & Third-Party Frameworks: eframe, egui, winit, wgpu, glow, zune-jpeg, image, fast_image_resize, moxcms, image-webp, zip, rfd, sys-locale",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH
    )
}

/// Draws the About dialog. Returns `false` when the user closed it.
pub fn draw_about_window(ctx: &egui::Context, lang: Language) -> bool {
    let info_text = about_info_text();
    let mut open = true;
    egui::Window::new(I18n::tr(lang, "dialog.about"))
        .id(egui::Id::unique("iv_about"))
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label(&info_text);
            ui.separator();
            if ui.button(I18n::tr(lang, "about.copy")).clicked() {
                ctx.copy_text(info_text.clone());
            }
        });
    open
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
        assert!(text.contains("eframe"));
        assert!(text.contains("egui"));
        assert!(text.contains("winit"));
        assert!(text.contains("wgpu"));
        assert!(text.contains("zune-jpeg"));
    }
}
