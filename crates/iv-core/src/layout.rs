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

    /// Places an image of `src_w x src_h` source pixels on screen in logical points.
    ///
    /// The returned rect already accounts for fit mode, zoom, pan, DPI and quarter-turn rotation;
    /// [`ImageQuad::map`] converts normalized image coordinates to screen points including
    /// rotation and screen-space flips.
    pub fn image_quad(src_w: u32, src_h: u32, vp: &ViewportState) -> ImageQuad {
        let fit = Self::compute_fit_scale(src_w, src_h, vp, vp.display_mode);
        let (eff_w, eff_h) = if vp.rotation_quarter_turns % 2 == 1 {
            (f64::from(src_h.max(1)), f64::from(src_w.max(1)))
        } else {
            (f64::from(src_w.max(1)), f64::from(src_h.max(1)))
        };
        let scale = fit * vp.zoom_scale / vp.dpi_scale_factor;
        let width = eff_w * scale;
        let height = eff_h * scale;
        let center_x = vp.logical_width * 0.5 + vp.pan_x;
        let center_y = vp.logical_height * 0.5 + vp.pan_y;
        ImageQuad {
            min_x: center_x - width * 0.5,
            min_y: center_y - height * 0.5,
            width,
            height,
            rotation_quarter_turns: vp.rotation_quarter_turns % 4,
            flip_h: vp.flip_h,
            flip_v: vp.flip_v,
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

/// Screen placement of an image: a logical-point rect plus the orientation used to map image
/// coordinates into it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImageQuad {
    pub min_x: f64,
    pub min_y: f64,
    pub width: f64,
    pub height: f64,
    pub rotation_quarter_turns: u8,
    pub flip_h: bool,
    pub flip_v: bool,
}

impl ImageQuad {
    /// Maps normalized image coordinates (`u` right, `v` down, both `0..=1` on the unrotated
    /// image) to screen points. Rotation is clockwise; flips mirror the displayed result.
    pub fn map(&self, u: f64, v: f64) -> (f64, f64) {
        let (mut x, mut y) = (u, v);
        for _ in 0..self.rotation_quarter_turns % 4 {
            (x, y) = (1.0 - y, x);
        }
        if self.flip_h {
            x = 1.0 - x;
        }
        if self.flip_v {
            y = 1.0 - y;
        }
        (self.min_x + x * self.width, self.min_y + y * self.height)
    }
}

/// One texture tile of a larger image, in source pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileRect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

/// Splits a `width x height` image into row-major tiles no larger than `max_side` on either axis.
pub fn tile_grid(width: u32, height: u32, max_side: u32) -> Vec<TileRect> {
    if width == 0 || height == 0 || max_side == 0 {
        return Vec::new();
    }
    let mut tiles = Vec::new();
    let mut y = 0;
    while y < height {
        let h = (height - y).min(max_side);
        let mut x = 0;
        while x < width {
            let w = (width - x).min(max_side);
            tiles.push(TileRect { x, y, w, h });
            x += w;
        }
        y += h;
    }
    tiles
}
