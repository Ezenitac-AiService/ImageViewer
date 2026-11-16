//! Bilingual Korean (`ko`) and English (`en`) localization catalog.

use serde::{Deserialize, Serialize};

/// Supported UI languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Language {
    #[serde(rename = "en")]
    #[default]
    En,
    #[serde(rename = "ko")]
    Ko,
}

impl Language {
    /// Detects default language from system locale tags (`ko*` -> `Ko`, otherwise `En`).
    pub fn from_locale(tag: &str) -> Self {
        if tag.trim().to_ascii_lowercase().starts_with("ko") {
            Self::Ko
        } else {
            Self::En
        }
    }
}

/// Static bilingual lookup table.
pub struct I18n;

impl I18n {
    /// Returns the localized string for `key`, falling back to English or `key` if unknown.
    pub fn tr(lang: Language, key: &str) -> &'static str {
        match (lang, key) {
            (Language::En, "app.title") | (Language::Ko, "app.title") => "ImageViewer",
            (Language::En, "error.path_not_found") => {
                "The requested file or folder could not be found"
            }
            (Language::Ko, "error.path_not_found") => "요청한 파일 또는 폴더를 찾을 수 없습니다",
            (Language::En, "error.invalid_cli_args") => "Invalid command-line arguments",
            (Language::Ko, "error.invalid_cli_args") => "잘못된 명령줄 인수입니다",
            (Language::En, "error.unsupported_os_switch") => {
                "Shell registration switches are only supported on Windows"
            }
            (Language::Ko, "error.unsupported_os_switch") => {
                "셸 등록 스위치는 Windows에서만 지원됩니다"
            }
            (Language::En, "error.encrypted_archive") => "Encrypted archives are not supported",
            (Language::Ko, "error.encrypted_archive") => "암호화된 압축 파일은 지원되지 않습니다",
            (Language::En, "error.missing_os_codec") => {
                "No compatible system codec is available for this format"
            }
            (Language::Ko, "error.missing_os_codec") => {
                "이 형식을 열 수 있는 시스템 코덱이 설치되어 있지 않습니다"
            }
            (Language::En, "error.unsafe_image_or_archive") => {
                "Blocked unsafe or malformed image/archive input"
            }
            (Language::Ko, "error.unsafe_image_or_archive") => {
                "안전하지 않거나 손상된 이미지/압축 입력이 차단되었습니다"
            }
            (Language::En, "dialog.settings") => "Settings",
            (Language::Ko, "dialog.settings") => "설정",
            (Language::En, "dialog.about") => "About ImageViewer",
            (Language::Ko, "dialog.about") => "ImageViewer 정보",
            (Language::En, "dialog.image_list") => "Image List",
            (Language::Ko, "dialog.image_list") => "이미지 목록",
            (Language::En, "menu.open_file") => "Open File / Archive",
            (Language::Ko, "menu.open_file") => "파일 / 압축 열기",
            (Language::En, "menu.open_folder") => "Open Folder",
            (Language::Ko, "menu.open_folder") => "폴더 열기",
            (Language::En, "menu.exif") => "EXIF Info",
            (Language::Ko, "menu.exif") => "EXIF 정보",
            (Language::En, "menu.slideshow") => "Slideshow",
            (Language::Ko, "menu.slideshow") => "슬라이드쇼",
            (Language::En, "menu.pin_bars") => "Pin Toolbars",
            (Language::Ko, "menu.pin_bars") => "툴바 고정",
            (Language::En, "update.available") => "A new update is available",
            (Language::Ko, "update.available") => "새 업데이트가 있습니다",
            (Language::En, "update.skip") => "Skip Version",
            (Language::Ko, "update.skip") => "이 버전 건너뛰기",
            (Language::En, "update.dismiss") => "Dismiss",
            (Language::Ko, "update.dismiss") => "닫기",
            (Language::En, "update.install") => "Install",
            (Language::Ko, "update.install") => "설치",
            (Language::En, "settings.ok") => "OK",
            (Language::Ko, "settings.ok") => "확인",
            (Language::En, "settings.cancel") => "Cancel",
            (Language::Ko, "settings.cancel") => "취소",
            (Language::En, "about.copy") => "Copy Info",
            (Language::Ko, "about.copy") => "정보 복사",
            (Language::En, "about.check_updates") => "Check for Updates",
            (Language::Ko, "about.check_updates") => "업데이트 확인",
            _ => match key {
                "app.title" => "ImageViewer",
                "error.path_not_found" => "The requested file or folder could not be found",
                "error.invalid_cli_args" => "Invalid command-line arguments",
                "error.unsupported_os_switch" => {
                    "Shell registration switches are only supported on Windows"
                }
                "error.encrypted_archive" => "Encrypted archives are not supported",
                "error.missing_os_codec" => {
                    "No compatible system codec is available for this format"
                }
                "error.unsafe_image_or_archive" => {
                    "Blocked unsafe or malformed image/archive input"
                }
                "dialog.settings" => "Settings",
                "dialog.about" => "About ImageViewer",
                "dialog.image_list" => "Image List",
                _ => "",
            },
        }
    }
}
