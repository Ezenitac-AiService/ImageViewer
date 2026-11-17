//! OS locale → UI language (FR-018).

use iv_core::i18n::Language;

/// Korean when the first (most preferred) tag is Korean, English otherwise.
pub fn language_from_tags(tags: &[&str]) -> Language {
    tags.first()
        .map_or(Language::En, |tag| Language::from_locale(tag))
}

/// The user's preferred locale tags, most preferred first (empty when unknown).
pub fn system_locale_tags() -> Vec<String> {
    sys_locale::get_locales().collect()
}
