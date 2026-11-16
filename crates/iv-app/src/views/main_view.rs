//! Custom frameless titlebar, floating top/bottom toolbars, EXIF panel, and HUD overlays.

use egui::{Align2, Color32, RichText};
use iv_core::chrome_state::ChromeState;
use iv_core::i18n::{I18n, Language};
use iv_core::layout::ViewportState;
use iv_core::settings::ThumbnailSettings;
use iv_media::cache::DecodedFrame;
use iv_media::sources::MediaEntry;

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
    SelectIndex(usize),
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
    thumb_cfg: &ThumbnailSettings,
    entries: &[MediaEntry],
    frame: Option<&DecodedFrame>,
    anim_status: Option<(usize, usize, u128, bool)>,
    loading_spinner: Option<bool>,
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
                    let prefix = if let Some((a_idx, a_total, delay_ms, playing)) = anim_status {
                        let st = if playing { "playing" } else { "paused" };
                        format!("frame {}/{a_total} · {st} · {delay_ms}ms · ", a_idx + 1)
                    } else if f.page_count > 1 {
                        format!("page {}/{} · ", f.page_index + 1, f.page_count)
                    } else {
                        String::new()
                    };
                    ui.label(format!(
                        "{prefix}{}×{} · {}",
                        f.source_width, f.source_height, f.format_label
                    ));
                });
            });
    }

    if let Some(show_spinner) = loading_spinner {
        egui::Area::new(egui::Id::new("iv_loading_indicator"))
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    if show_spinner {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Loading...");
                        });
                    } else {
                        ui.label("Preview...");
                    }
                });
            });
    }

    // 5. Thumbnail Strip (48–256 px + optional scrollbar)
    if thumb_cfg.visible && !entries.is_empty() && !chrome.is_fullscreen {
        let strip_h = thumb_cfg.size.clamp(48, 256) as f32 * 0.45 + 24.0;
        egui::TopBottomPanel::bottom("iv_thumb_strip")
            .exact_height(strip_h)
            .show(ctx, |ui| {
                let mut scroll = egui::ScrollArea::horizontal();
                if !thumb_cfg.show_scroll_bar {
                    scroll = scroll.scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden);
                }
                scroll.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        for (idx, entry) in entries.iter().enumerate() {
                            if ui.selectable_label(idx == pos_index, &entry.name).clicked() {
                                action = MainUiAction::SelectIndex(idx);
                            }
                        }
                    });
                });
            });
    }

    // 6. EXIF Panel (`Tab`)
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
