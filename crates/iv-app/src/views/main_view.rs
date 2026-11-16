//! Custom frameless titlebar, floating top/bottom toolbars, EXIF panel, and HUD overlays.

use egui::{Align2, Color32, RichText};
use iv_core::chrome_state::ChromeState;
use iv_core::i18n::{I18n, Language};
use iv_core::layout::ViewportState;
use iv_media::cache::DecodedFrame;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainUiAction {
    None,
    Minimize,
    ToggleMaximize,
    CloseWindow,
    OpenFile,
    OpenFolder,
    PrevImage,
    NextImage,
    ZoomIn,
    ZoomOut,
    ZoomReset,
    TogglePinAll,
    ToggleExif,
    ToggleSlideshow,
    OpenImageList,
    OpenSettings,
    OpenAbout,
    DragWindow,
}

#[allow(clippy::too_many_arguments)]
pub fn draw_main_overlays(
    ctx: &egui::Context,
    lang: Language,
    chrome: &ChromeState,
    viewport: &ViewportState,
    frame: Option<&DecodedFrame>,
    current_name: &str,
    pos_index: usize,
    total_count: usize,
    is_slideshow_playing: bool,
    status_error: Option<&str>,
) -> MainUiAction {
    let mut action = MainUiAction::None;

    // 1. Custom Frameless Titlebar
    egui::TopBottomPanel::top("iv_titlebar")
        .exact_height(32.0)
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                let drag_resp = ui.interact(
                    ui.available_rect_before_wrap(),
                    egui::Id::new("titlebar_drag"),
                    egui::Sense::click_and_drag(),
                );
                if drag_resp.drag_started() {
                    action = MainUiAction::DragWindow;
                }
                if drag_resp.double_clicked() {
                    action = MainUiAction::ToggleMaximize;
                }

                let title = if current_name.is_empty() {
                    I18n::tr(lang, "app.title").to_string()
                } else {
                    format!("{current_name} — {}", I18n::tr(lang, "app.title"))
                };
                ui.label(RichText::new(title).strong().color(Color32::WHITE));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("✕").clicked() {
                        action = MainUiAction::CloseWindow;
                    }
                    if ui.button("□").clicked() {
                        action = MainUiAction::ToggleMaximize;
                    }
                    if ui.button("—").clicked() {
                        action = MainUiAction::Minimize;
                    }
                });
            });
        });

    // 2. Floating Top Bar
    if chrome.top_bar_visible() {
        egui::Area::new(egui::Id::new("iv_top_bar"))
            .anchor(Align2::CENTER_TOP, [0.0, 40.0])
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button(I18n::tr(lang, "menu.open_file")).clicked() {
                            action = MainUiAction::OpenFile;
                        }
                        if ui.button(I18n::tr(lang, "menu.open_folder")).clicked() {
                            action = MainUiAction::OpenFolder;
                        }
                        if ui.button(I18n::tr(lang, "menu.exif")).clicked() {
                            action = MainUiAction::ToggleExif;
                        }
                        if ui.button(I18n::tr(lang, "dialog.image_list")).clicked() {
                            action = MainUiAction::OpenImageList;
                        }
                        if ui.button(I18n::tr(lang, "dialog.settings")).clicked() {
                            action = MainUiAction::OpenSettings;
                        }
                        if ui.button(I18n::tr(lang, "dialog.about")).clicked() {
                            action = MainUiAction::OpenAbout;
                        }
                    });
                });
            });
    }

    // 3. Floating Bottom Action Bubble
    if chrome.bottom_bar_visible() {
        egui::Area::new(egui::Id::new("iv_bottom_bar"))
            .anchor(Align2::CENTER_BOTTOM, [0.0, -16.0])
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("－").clicked() {
                            action = MainUiAction::ZoomOut;
                        }
                        if ui.button(format!("{:.0}%", viewport.zoom_scale * 100.0)).clicked() {
                            action = MainUiAction::ZoomReset;
                        }
                        if ui.button("＋").clicked() {
                            action = MainUiAction::ZoomIn;
                        }
                        ui.separator();
                        if ui.button("◀").clicked() {
                            action = MainUiAction::PrevImage;
                        }
                        let pos_label = if total_count > 0 {
                            format!("{} / {}", pos_index + 1, total_count)
                        } else {
                            "0 / 0".to_string()
                        };
                        ui.label(pos_label);
                        if ui.button("▶").clicked() {
                            action = MainUiAction::NextImage;
                        }
                        ui.separator();
                        let ss_lbl = if is_slideshow_playing { "⏸" } else { "▶ Slideshow" };
                        if ui.button(ss_lbl).clicked() {
                            action = MainUiAction::ToggleSlideshow;
                        }
                        if ui.button("📌").clicked() {
                            action = MainUiAction::TogglePinAll;
                        }
                    });
                });
            });
    }

    // 4. Bottom-Left HUD Badge (always reports true post-orientation source_width x source_height)
    if let Some(f) = frame {
        egui::Area::new(egui::Id::new("iv_hud_badge"))
            .anchor(Align2::LEFT_BOTTOM, [12.0, -16.0])
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    let page_str = if f.page_count > 1 {
                        format!("page {}/{} · ", f.page_index + 1, f.page_count)
                    } else {
                        String::new()
                    };
                    ui.label(format!(
                        "{page_str}{}×{} · {}",
                        f.source_width, f.source_height, f.format_label
                    ));
                });
            });
    }

    // 5. EXIF Panel (`Tab`)
    if chrome.info_panel_visible
        && let Some(f) = frame
    {
        egui::SidePanel::right("iv_exif_panel")
            .resizable(true)
            .default_width(260.0)
            .show(ctx, |ui| {
                ui.heading(I18n::tr(lang, "menu.exif"));
                ui.separator();
                ui.label(format!("{} × {}", f.source_width, f.source_height));
                for item in &f.exif_items {
                    ui.horizontal(|ui| {
                        ui.strong(item.label_key);
                        ui.label(&item.value);
                    });
                }
            });
    }

    if let Some(err_msg) = status_error {
        egui::Area::new(egui::Id::new("iv_status_error"))
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.colored_label(Color32::LIGHT_RED, err_msg);
                });
            });
    }

    action
}
