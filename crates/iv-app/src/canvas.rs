//! Image canvas: uploads decoded frames as egui textures (tiled to the GPU limit) and paints them
//! with zoom, pan, rotation and flips (research R1; layout maths in `iv_core::layout`).

use eframe::egui::{self, Color32, Mesh, Pos2, Rect, Shape, TextureHandle, TextureOptions};
use iv_core::layout::{TileRect, ViewportLayoutCalculator, ViewportState, tile_grid};
use iv_core::settings::{FilterMode, ViewerBackground};
use iv_media::cache::DecodedFrame;

struct Tile {
    rect: TileRect,
    texture: TextureHandle,
}

/// GPU-side copy of the current frame.
pub struct Canvas {
    tiles: Vec<Tile>,
    buffer_size: (u32, u32),
    source_size: (u32, u32),
    filter: FilterMode,
}

impl Default for Canvas {
    fn default() -> Self {
        Self {
            tiles: Vec::new(),
            buffer_size: (0, 0),
            source_size: (0, 0),
            filter: FilterMode::None,
        }
    }
}

impl Canvas {
    pub fn clear(&mut self) {
        self.tiles.clear();
        self.buffer_size = (0, 0);
        self.source_size = (0, 0);
    }

    /// Uploads `frame`. Reuses existing textures in place when the tile layout is unchanged
    /// (animation frames), so playback never allocates new GPU textures per frame.
    pub fn set_frame(&mut self, ctx: &egui::Context, frame: &DecodedFrame, filter: FilterMode) {
        let (w, h) = (frame.buffer_width, frame.buffer_height);
        let expected_len = (w as usize)
            .checked_mul(h as usize)
            .and_then(|p| p.checked_mul(4));
        if w == 0 || h == 0 || expected_len != Some(frame.rgba8.len()) {
            self.clear();
            return;
        }
        let max_side = u32::try_from(ctx.input(|i| i.max_texture_side)).unwrap_or(u32::MAX);
        let grid = tile_grid(w, h, max_side);
        let options = texture_options(filter);
        let reuse =
            self.buffer_size == (w, h) && self.filter == filter && self.tiles.len() == grid.len();
        if reuse {
            for tile in &mut self.tiles {
                let image = tile_image(frame, tile.rect);
                tile.texture.set(image, options);
            }
        } else {
            self.tiles = grid
                .into_iter()
                .enumerate()
                .map(|(i, rect)| Tile {
                    rect,
                    texture: ctx.load_texture(
                        format!("iv-tile-{i}"),
                        tile_image(frame, rect),
                        options,
                    ),
                })
                .collect();
        }
        self.buffer_size = (w, h);
        self.source_size = (frame.source_width.max(1), frame.source_height.max(1));
        self.filter = filter;
    }

    /// Changes the sampling filter of the current textures without a new decode.
    pub fn set_filter(&mut self, filter: FilterMode) {
        if self.filter == filter {
            return;
        }
        // Texture options are fixed at upload; the next `set_frame` rebuilds with the new filter.
        self.filter = filter;
        self.buffer_size = (0, 0);
    }

    pub fn filter(&self) -> FilterMode {
        self.filter
    }

    /// Paints the background and the image into `area` (logical points).
    pub fn paint(
        &self,
        painter: &egui::Painter,
        area: Rect,
        viewport: &ViewportState,
        background: ViewerBackground,
    ) {
        painter.rect_filled(area, 0.0, background_color(background));
        if self.tiles.is_empty() {
            return;
        }
        let quad =
            ViewportLayoutCalculator::image_quad(self.source_size.0, self.source_size.1, viewport);
        if background == ViewerBackground::Checkerboard {
            let r = Rect::from_min_size(
                Pos2::new(
                    area.min.x + quad.min_x as f32,
                    area.min.y + quad.min_y as f32,
                ),
                egui::vec2(quad.width as f32, quad.height as f32),
            )
            .intersect(area);
            paint_checkerboard(painter, r);
        }
        let (bw, bh) = (f64::from(self.buffer_size.0), f64::from(self.buffer_size.1));
        let to_screen = |u: f64, v: f64| {
            let (x, y) = quad.map(u, v);
            Pos2::new(area.min.x + x as f32, area.min.y + y as f32)
        };
        for tile in &self.tiles {
            let u0 = f64::from(tile.rect.x) / bw;
            let v0 = f64::from(tile.rect.y) / bh;
            let u1 = f64::from(tile.rect.x + tile.rect.w) / bw;
            let v1 = f64::from(tile.rect.y + tile.rect.h) / bh;
            let mut mesh = Mesh::with_texture(tile.texture.id());
            let corners = [
                (to_screen(u0, v0), Pos2::new(0.0, 0.0)),
                (to_screen(u1, v0), Pos2::new(1.0, 0.0)),
                (to_screen(u1, v1), Pos2::new(1.0, 1.0)),
                (to_screen(u0, v1), Pos2::new(0.0, 1.0)),
            ];
            for (pos, uv) in corners {
                mesh.vertices.push(egui::epaint::Vertex {
                    pos,
                    uv,
                    color: Color32::WHITE,
                });
            }
            mesh.add_triangle(0, 1, 2);
            mesh.add_triangle(0, 2, 3);
            painter.with_clip_rect(area).add(Shape::mesh(mesh));
        }
    }
}

fn texture_options(filter: FilterMode) -> TextureOptions {
    match filter {
        FilterMode::None => TextureOptions::NEAREST,
        FilterMode::Smooth | FilterMode::SmoothSharpen => TextureOptions::LINEAR,
    }
}

fn tile_image(frame: &DecodedFrame, rect: TileRect) -> egui::ColorImage {
    let stride = frame.buffer_width as usize * 4;
    let row_len = rect.w as usize * 4;
    let mut bytes = Vec::with_capacity(row_len * rect.h as usize);
    for row in rect.y as usize..(rect.y + rect.h) as usize {
        let start = row * stride + rect.x as usize * 4;
        bytes.extend_from_slice(&frame.rgba8[start..start + row_len]);
    }
    egui::ColorImage::from_rgba_unmultiplied([rect.w as usize, rect.h as usize], &bytes)
}

fn background_color(bg: ViewerBackground) -> Color32 {
    match bg {
        ViewerBackground::Black => Color32::BLACK,
        ViewerBackground::DarkGray => Color32::from_gray(31),
        ViewerBackground::White => Color32::WHITE,
        ViewerBackground::Checkerboard => Color32::from_gray(41),
    }
}

fn paint_checkerboard(painter: &egui::Painter, rect: Rect) {
    const CELL: f32 = 12.0;
    if !rect.is_positive() {
        return;
    }
    let light = Color32::from_gray(87);
    let dark = Color32::from_gray(56);
    let cols = (rect.width() / CELL).ceil() as i32;
    let rows = (rect.height() / CELL).ceil() as i32;
    for r in 0..rows {
        for c in 0..cols {
            let min = Pos2::new(rect.min.x + c as f32 * CELL, rect.min.y + r as f32 * CELL);
            let cell = Rect::from_min_size(min, egui::vec2(CELL, CELL)).intersect(rect);
            painter.rect_filled(cell, 0.0, if (r + c) % 2 == 0 { light } else { dark });
        }
    }
}
