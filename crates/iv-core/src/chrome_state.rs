//! Frameless chrome pin/hover/fullscreen state (Wayland-aware).

use crate::settings::WindowGeometry;

#[derive(Debug, Clone, PartialEq)]
pub struct ChromeState {
    pub top_bar_pinned: bool,
    pub bottom_bar_pinned: bool,
    pub hover_top: bool,
    pub hover_bottom: bool,
    pub is_fullscreen: bool,
    pub info_panel_visible: bool,
    pub saved_window_geometry: Option<WindowGeometry>,
}

impl Default for ChromeState {
    fn default() -> Self {
        Self {
            top_bar_pinned: true,
            bottom_bar_pinned: true,
            hover_top: false,
            hover_bottom: false,
            is_fullscreen: false,
            info_panel_visible: false,
            saved_window_geometry: None,
        }
    }
}

impl ChromeState {
    /// Global `` ` `` toggle: if either bar is pinned -> unpin both; if both unpinned -> pin both.
    pub fn toggle_all_pins(&mut self) {
        if self.top_bar_pinned || self.bottom_bar_pinned {
            self.top_bar_pinned = false;
            self.bottom_bar_pinned = false;
        } else {
            self.top_bar_pinned = true;
            self.bottom_bar_pinned = true;
        }
    }

    pub fn top_bar_visible(&self) -> bool {
        self.top_bar_pinned || self.hover_top
    }

    pub fn bottom_bar_visible(&self) -> bool {
        self.bottom_bar_pinned || self.hover_bottom
    }

    /// Returns whether `(left, top)` coordinates should be applied on the current OS/display backend.
    pub fn should_restore_window_position(is_wayland: bool) -> bool {
        !is_wayland
    }
}
