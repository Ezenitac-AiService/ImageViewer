//! Native error dialog for failures that end a launch
//! (`specs/004-launch-reliability-refactor/contracts/launch-and-diagnostics-contract.md` §4).

use iv_core::diagnostics::FatalCategory;
use iv_core::i18n::{I18n, Language};

/// The log folder as shown to the user: environment-variable / `~` form, never the real path
/// (Constitution VII, CWE-532).
pub fn redacted_log_folder(os: &str) -> &'static str {
    match os {
        "macos" => "~/Library/Application Support/com.Ezenitac-AiService.ImageViewer/logs",
        _ => "~/.config/imageviewer/logs",
    }
}

/// The dialog body for `category`: the cause sentence, then where details were saved.
pub fn fatal_message(lang: Language, category: FatalCategory) -> String {
    let cause = match category {
        FatalCategory::NoRenderer => "fatal.no_renderer",
        FatalCategory::WindowFailed => "fatal.window_failed",
        FatalCategory::SettingsUnwritable => "settings.unreadable_notice",
        FatalCategory::Panic => "fatal.panic",
    };
    format!(
        "{}\n\n{}\n{}",
        I18n::tr(lang, cause),
        I18n::tr(lang, "fatal.log_location"),
        redacted_log_folder(std::env::consts::OS)
    )
}

/// Shows a blocking native error dialog (title "ImageViewer", OK only).
pub fn show_fatal(lang: Language, category: FatalCategory) {
    rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Error)
        .set_title("ImageViewer")
        .set_description(fatal_message(lang, category))
        .set_buttons(rfd::MessageButtons::Ok)
        .show();
}
