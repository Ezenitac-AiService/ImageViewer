//! Korean-capable fallback fonts from the host OS (egui's bundled fonts have no Hangul glyphs).

use eframe::egui;
use std::sync::Arc;

/// System fonts with Hangul coverage, tried in order; the first readable one is used.
const CJK_CANDIDATES: &[&str] = &[
    "/System/Library/Fonts/AppleSDGothicNeo.ttc",
    "C:\\Windows\\Fonts\\malgun.ttf",
    "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
    "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
];

/// Appends the first available CJK system font as a fallback for proportional and monospace text.
/// Returns the path that was installed, if any.
pub fn install_cjk_fallback(ctx: &egui::Context) -> Option<&'static str> {
    for path in CJK_CANDIDATES {
        let Ok(bytes) = std::fs::read(path) else {
            continue;
        };
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "iv_cjk".to_owned(),
            Arc::new(egui::FontData::from_owned(bytes)),
        );
        for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
            if let Some(list) = fonts.families.get_mut(&family) {
                list.push("iv_cjk".to_owned());
            }
        }
        ctx.set_fonts(fonts);
        return Some(path);
    }
    None
}
