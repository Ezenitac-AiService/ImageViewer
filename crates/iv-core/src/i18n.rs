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
                "Shell registration switches are not supported in this version"
            }
            (Language::Ko, "error.unsupported_os_switch") => {
                "이 버전에서는 셸 등록 스위치를 지원하지 않습니다"
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
            (Language::En, "fatal.no_renderer") => {
                "ImageViewer could not start because no graphics renderer is available on this computer."
            }
            (Language::Ko, "fatal.no_renderer") => {
                "이 컴퓨터에서 사용할 수 있는 그래픽 렌더러가 없어 ImageViewer를 시작할 수 없습니다."
            }
            (Language::En, "fatal.window_failed") => "ImageViewer could not open its window.",
            (Language::Ko, "fatal.window_failed") => "ImageViewer 창을 열 수 없습니다.",
            (Language::En, "fatal.panic") => "ImageViewer stopped because of an internal error.",
            (Language::Ko, "fatal.panic") => "내부 오류로 ImageViewer가 종료되었습니다.",
            (Language::En, "fatal.log_location") => "Details were saved in this folder:",
            (Language::Ko, "fatal.log_location") => "자세한 내용은 다음 폴더에 저장되었습니다:",
            (Language::En, "settings.reset_notice") => {
                "Your settings could not be read and were reset to the defaults."
            }
            (Language::Ko, "settings.reset_notice") => {
                "설정을 읽을 수 없어 기본값으로 초기화했습니다."
            }
            (Language::En, "settings.unreadable_notice") => {
                "Settings cannot be loaded or saved here, so changes last only until you quit."
            }
            (Language::Ko, "settings.unreadable_notice") => {
                "이 위치에서는 설정을 불러오거나 저장할 수 없어, 변경 사항은 종료할 때까지만 유지됩니다."
            }
            (Language::En, "settings.save_failed") => "Settings could not be saved.",
            (Language::Ko, "settings.save_failed") => "설정을 저장하지 못했습니다.",
            (Language::En, "error.unsupported_format") => {
                "This format is not supported in this version."
            }
            (Language::Ko, "error.unsupported_format") => "이 버전에서는 지원하지 않는 형식입니다.",
            (Language::En, "error.unsupported_switch") => {
                "This command-line option is not available in this version."
            }
            (Language::Ko, "error.unsupported_switch") => {
                "이 버전에서는 사용할 수 없는 명령줄 옵션입니다."
            }
            (Language::En, "error.security_rejected") => {
                "This file was blocked because it is not a safe image."
            }
            (Language::Ko, "error.security_rejected") => {
                "안전한 이미지가 아니어서 이 파일을 차단했습니다."
            }
            (Language::En, "error.too_large") => {
                "This image is larger than the configured size limit."
            }
            (Language::Ko, "error.too_large") => "이미지가 설정된 크기 제한보다 큽니다.",
            (Language::En, "error.corrupt") => "This image is damaged or cannot be decoded.",
            (Language::Ko, "error.corrupt") => "이미지가 손상되었거나 디코딩할 수 없습니다.",
            (Language::En, "error.read_failed") => "The file could not be read.",
            (Language::Ko, "error.read_failed") => "파일을 읽을 수 없습니다.",
            (Language::En, "error.archive_failed") => "The archive could not be read.",
            (Language::Ko, "error.archive_failed") => "압축 파일을 읽을 수 없습니다.",
            (Language::En, "error.archive_limit") => "The archive exceeds the safety limits.",
            (Language::Ko, "error.archive_limit") => "압축 파일이 안전 제한을 초과합니다.",
            (Language::En, "error.animation_failed") => "The animation could not be played.",
            (Language::Ko, "error.animation_failed") => "애니메이션을 재생할 수 없습니다.",
            (Language::En, "toolbar.zoom_in") => "Zoom in",
            (Language::Ko, "toolbar.zoom_in") => "확대",
            (Language::En, "toolbar.zoom_out") => "Zoom out",
            (Language::Ko, "toolbar.zoom_out") => "축소",
            (Language::En, "toolbar.reset_view") => "Reset view",
            (Language::Ko, "toolbar.reset_view") => "보기 초기화",
            (Language::En, "toolbar.prev") => "Previous image",
            (Language::Ko, "toolbar.prev") => "이전 이미지",
            (Language::En, "toolbar.next") => "Next image",
            (Language::Ko, "toolbar.next") => "다음 이미지",
            (Language::En, "status.loading") => "Loading…",
            (Language::Ko, "status.loading") => "불러오는 중…",
            (Language::En, "status.empty") => "Open a file or folder to start",
            (Language::Ko, "status.empty") => "파일이나 폴더를 열어 시작하세요",
            (Language::En, "picker.images") => "Images & archives",
            (Language::Ko, "picker.images") => "이미지 및 압축 파일",
            (Language::En, "settings.language") => "Language",
            (Language::Ko, "settings.language") => "언어",
            (Language::En, "settings.background") => "Background",
            (Language::Ko, "settings.background") => "배경",
            (Language::En, "settings.bg_black") => "Black",
            (Language::Ko, "settings.bg_black") => "검정",
            (Language::En, "settings.bg_dark_gray") => "Dark gray",
            (Language::Ko, "settings.bg_dark_gray") => "진회색",
            (Language::En, "settings.bg_white") => "White",
            (Language::Ko, "settings.bg_white") => "흰색",
            (Language::En, "settings.bg_checkerboard") => "Checkerboard",
            (Language::Ko, "settings.bg_checkerboard") => "체크무늬",
            (Language::En, "settings.mouse_wheel") => "Mouse wheel",
            (Language::Ko, "settings.mouse_wheel") => "마우스 휠",
            (Language::En, "settings.wheel_navigate") => "Change image",
            (Language::Ko, "settings.wheel_navigate") => "이미지 이동",
            (Language::En, "settings.wheel_zoom") => "Zoom",
            (Language::Ko, "settings.wheel_zoom") => "확대/축소",
            (Language::En, "settings.wrap_around") => "Wrap around at the ends",
            (Language::Ko, "settings.wrap_around") => "끝에서 처음으로 돌아가기",
            (Language::En, "settings.watch_folder") => "Watch the folder for changes",
            (Language::Ko, "settings.watch_folder") => "폴더 변경 감시",
            (Language::En, "settings.play_animations") => "Play animations",
            (Language::Ko, "settings.play_animations") => "애니메이션 재생",
            (Language::En, "settings.slideshow_interval") => "Slideshow interval (seconds)",
            (Language::Ko, "settings.slideshow_interval") => "슬라이드쇼 간격(초)",
            (Language::En, "settings.slideshow_repeat") => "Repeat slideshow",
            (Language::Ko, "settings.slideshow_repeat") => "슬라이드쇼 반복",
            (Language::En, "settings.max_megapixels") => "Maximum image size (megapixels)",
            (Language::Ko, "settings.max_megapixels") => "최대 이미지 크기(메가픽셀)",
            (Language::En, "image_list.name") => "Name",
            (Language::Ko, "image_list.name") => "이름",
            (Language::En, "image_list.date") => "Date",
            (Language::Ko, "image_list.date") => "날짜",
            (Language::En, "image_list.size") => "Size",
            (Language::Ko, "image_list.size") => "크기",
            (Language::En, "image_list.unsupported") => "unsupported",
            (Language::Ko, "image_list.unsupported") => "지원 안 함",
            _ => match key {
                "app.title" => "ImageViewer",
                "error.path_not_found" => "The requested file or folder could not be found",
                "error.invalid_cli_args" => "Invalid command-line arguments",
                "error.unsupported_os_switch" => {
                    "Shell registration switches are not supported in this version"
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
