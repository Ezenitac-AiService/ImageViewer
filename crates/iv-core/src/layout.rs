//! DPI-aware viewport layout, cursor-anchored zoom, pan clamping, and screen-only transforms.

use crate::settings::DisplayMode;

/// Mutable viewport presentation state for a viewer window.
#[derive(Debug, Clone, PartialEq)]
pub struct ViewportState {
    pub display_mode: DisplayMode,
    pub logical_width: f64,
    pub logical_height: f64,
    pub dpi_scale_factor: f64,
    pub zoom_scale: f64,
    pub pan_x: f64,
    pub pan_y: f64,
    pub rotation_quarter_turns: u8,
    pub flip_h: bool,
    pub flip_v: bool,
}

impl ViewportState {
    pub fn new(logical_width: f64, logical_height: f64, dpi_scale_factor: f64) -> Self {
        Self {
            display_mode: DisplayMode::Fill,
            logical_width: logical_width.max(1.0),
            logical_height: logical_height.max(1.0),
            dpi_scale_factor: if dpi_scale_factor.is_finite() && dpi_scale_factor > 0.0 {
                dpi_scale_factor
            } else {
                1.0
            },
            zoom_scale: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
            rotation_quarter_turns: 0,
            flip_h: false,
            flip_v: false,
        }
    }

    pub fn physical_width(&self) -> u32 {
        (self.logical_width * self.dpi_scale_factor)
            .round()
            .max(1.0) as u32
    }

    pub fn physical_height(&self) -> u32 {
        (self.logical_height * self.dpi_scale_factor)
            .round()
            .max(1.0) as u32
    }

    pub fn rotate_cw(&mut self) {
        self.rotation_quarter_turns = (self.rotation_quarter_turns + 1) % 4;
    }

    pub fn rotate_ccw(&mut self) {
        self.rotation_quarter_turns = (self.rotation_quarter_turns + 3) % 4;
    }

    pub fn toggle_flip_h(&mut self) {
        self.flip_h = !self.flip_h;
    }

    pub fn toggle_flip_v(&mut self) {
        self.flip_v = !self.flip_v;
    }

    pub fn reset_transforms(&mut self) {
        self.rotation_quarter_turns = 0;
        self.flip_h = false;
        self.flip_v = false;
        self.zoom_scale = 1.0;
        self.pan_x = 0.0;
        self.pan_y = 0.0;
    }

    /// Zooms around a logical cursor coordinate, adjusting pan so the point under the cursor stays anchored.
    pub fn zoom_at_cursor(&mut self, factor: f64, cursor_x: f64, cursor_y: f64) {
        if !factor.is_finite() || factor <= 0.0 {
            return;
        }
        let old_zoom = self.zoom_scale;
        let new_zoom = (old_zoom * factor).clamp(0.05, 32.0);
        let ratio = new_zoom / old_zoom;
        let cx = cursor_x - self.logical_width * 0.5;
        let cy = cursor_y - self.logical_height * 0.5;
        self.pan_x = (self.pan_x - cx) * ratio + cx;
        self.pan_y = (self.pan_y - cy) * ratio + cy;
        self.zoom_scale = new_zoom;
    }
}

pub struct ViewportLayoutCalculator;

impl ViewportLayoutCalculator {
    /// Computes the physical-pixel fit scale for an image of `src_w x src_h` (pre-rotation).
    pub fn compute_fit_scale(src_w: u32, src_h: u32, vp: &ViewportState, mode: DisplayMode) -> f64 {
        let (eff_w, eff_h) = if vp.rotation_quarter_turns % 2 == 1 {
            (src_h.max(1) as f64, src_w.max(1) as f64)
        } else {
            (src_w.max(1) as f64, src_h.max(1) as f64)
        };
        let pw = f64::from(vp.physical_width());
        let ph = f64::from(vp.physical_height());

        match mode {
            DisplayMode::Original => 1.0,
            DisplayMode::FitWidth => pw / eff_w,
            DisplayMode::FitHeight => ph / eff_h,
            DisplayMode::Fill => (pw / eff_w).min(ph / eff_h),
        }
    }

    /// Clamps pan offsets so the rendered image stays within reachable bounds of the viewport.
    pub fn clamp_pan(
        rendered_w: f64,
        rendered_h: f64,
        vp_w: f64,
        vp_h: f64,
        pan_x: f64,
        pan_y: f64,
    ) -> (f64, f64) {
        let max_x = ((rendered_w - vp_w) * 0.5).max(0.0);
        let max_y = ((rendered_h - vp_h) * 0.5).max(0.0);
        (pan_x.clamp(-max_x, max_x), pan_y.clamp(-max_y, max_y))
    }
}
