//! Owned `winit 0.30` `ApplicationHandler` + custom `wgpu` canvas + `egui-winit`/`egui-wgpu` multi-window shell.

mod renderer;
mod views;

use crate::renderer::{GpuRenderer, GpuTile, WindowGpuSurface};
use crate::views::about_view::draw_about_window;
use crate::views::image_list_view::draw_image_list;
use crate::views::main_view::{draw_main_overlays, MainUiAction};
use crate::views::settings_view::{draw_settings_window, SettingsDialogAction};
use crate::views::update_notice::draw_update_notice;
use crate::views::ChildWindowKind;
use iv_core::chrome_state::ChromeState;
use iv_core::cli::{CliCommand, CliOptions};
use iv_core::i18n::{I18n, Language};
use iv_core::layout::ViewportState;
use iv_core::logger::AsyncFileLogger;
use iv_core::settings::{DisplayMode, FilterMode, MouseWheelAction, SettingsStore, ViewerSettings};
use iv_core::trackpad::TrackpadAccumulator;
use iv_media::cache::{DecodedFrame, DecodedFrameCache};
use iv_media::decode::{DecodeOptions, MediaDecoder};
use iv_media::resize_filter::SimdResizer;
use iv_media::session::SessionCoordinator;
use iv_media::sources::folder::FolderSource;
use iv_media::sources::zip::ZipSource;
use iv_media::sources::MediaSource;
use iv_platform::paths_dialogs::{AppUserEvent, OsDialogs, OsPaths};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::keyboard::{Key, ModifiersState, NamedKey};
use winit::window::{Fullscreen, Window, WindowAttributes, WindowId};

struct MainWindowState {
    window: Arc<Window>,
    gpu_surface: WindowGpuSurface,
    egui_state: egui_winit::State,
    egui_ctx: egui::Context,
    tiles: Vec<GpuTile>,
    viewport: ViewportState,
    chrome: ChromeState,
    trackpad: TrackpadAccumulator,
    modifiers: ModifiersState,
    source: Option<Box<dyn MediaSource>>,
    current_index: usize,
    current_page: u32,
    current_frame: Option<Arc<DecodedFrame>>,
    frame_cache: DecodedFrameCache,
    session: SessionCoordinator,
    slideshow_playing: bool,
    status_error: Option<String>,
    start_instant: Instant,
}

struct ChildWindowState {
    kind: ChildWindowKind,
    window: Arc<Window>,
    gpu_surface: WindowGpuSurface,
    egui_state: egui_winit::State,
    egui_ctx: egui::Context,
}

struct ViewerApp {
    proxy: EventLoopProxy<AppUserEvent>,
    settings: ViewerSettings,
    logger: Option<AsyncFileLogger>,
    gpu: Option<GpuRenderer>,
    main_window: Option<MainWindowState>,
    child_windows: HashMap<WindowId, ChildWindowState>,
    initial_cli: CliOptions,
}

impl ViewerApp {
    fn new(proxy: EventLoopProxy<AppUserEvent>, initial_cli: CliOptions) -> Self {
        let settings = SettingsStore::load(&OsPaths::settings_file());
        let logger = AsyncFileLogger::new(&OsPaths::logs_dir(), 256).ok();
        Self {
            proxy,
            settings,
            logger,
            gpu: None,
            main_window: None,
            child_windows: HashMap::new(),
            initial_cli,
        }
    }

    fn is_wayland() -> bool {
        #[cfg(target_os = "linux")]
        {
            std::env::var_os("WAYLAND_DISPLAY").is_some()
        }
        #[cfg(not(target_os = "linux"))]
        {
            false
        }
    }

    fn open_target_path(&mut self, path: &Path) {
        let Some(main) = self.main_window.as_mut() else {
            return;
        };
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_ascii_lowercase())
            .unwrap_or_default();

        let source_res: Result<Box<dyn MediaSource>, String> = if ext == "zip" || ext == "cbz" {
            ZipSource::open(path, self.settings.limits.max_zip_entry_megabytes)
                .map(|z| Box::new(z) as Box<dyn MediaSource>)
                .map_err(|e| e.to_string())
        } else {
            FolderSource::open(path)
                .map(|f| Box::new(f) as Box<dyn MediaSource>)
                .map_err(|e| e.to_string())
        };

        match source_res {
            Ok(src) => {
                let idx = src.initial_index();
                main.source = Some(src);
                main.current_index = idx;
                main.current_page = 0;
                main.status_error = None;
                self.load_current_entry();
            }
            Err(err) => {
                if let Some(main) = self.main_window.as_mut() {
                    main.status_error = Some(err);
                }
            }
        }
    }

    fn load_current_entry(&mut self) {
        let (Some(main), Some(gpu)) = (self.main_window.as_mut(), self.gpu.as_ref()) else {
            return;
        };
        let Some(src) = main.source.as_ref() else {
            return;
        };
        if src.entries().is_empty() {
            return;
        }
        let idx = main.current_index.min(src.entries().len() - 1);
        let entry = &src.entries()[idx];
        let cache_key = format!("{}:{}", entry.source_key, main.current_page);

        if let Some(cached) = main.frame_cache.get(&cache_key) {
            main.tiles = gpu.upload_tiled_rgba8(&cached.rgba8, cached.buffer_width, cached.buffer_height);
            main.current_frame = Some(cached);
            main.status_error = None;
            return;
        }

        let _gen = main.session.begin_load(idx);
        let t0 = Instant::now();
        match src.read_entry_bytes(idx) {
            Ok(bytes) => {
                let opts = DecodeOptions {
                    max_megapixels: self.settings.limits.max_image_megapixels,
                    target_viewport_physical: Some((
                        main.viewport.physical_width(),
                        main.viewport.physical_height(),
                    )),
                    page_index: main.current_page,
                };
                match MediaDecoder::decode_bytes(&bytes, &entry.name, &opts) {
                    Ok(frame) => {
                        let arc = Arc::new(frame);
                        main.tiles = gpu.upload_tiled_rgba8(
                            &arc.rgba8,
                            arc.buffer_width,
                            arc.buffer_height,
                        );
                        main.frame_cache.insert(cache_key, Arc::clone(&arc));
                        main.current_frame = Some(arc);
                        main.status_error = None;
                        if let Some(log) = self.logger.as_ref() {
                            log.log_slow_render(&entry.name, t0.elapsed());
                        }
                    }
                    Err(err) => {
                        main.status_error = Some(err.to_string());
                    }
                }
            }
            Err(err) => {
                main.status_error = Some(err.to_string());
            }
        }
    }

    fn step_image(&mut self, delta: isize) {
        let Some(main) = self.main_window.as_mut() else {
            return;
        };
        let Some(src) = main.source.as_ref() else {
            return;
        };
        let len = src.entries().len();
        if len == 0 {
            return;
        }
        let next = if delta >= 0 {
            let n = main.current_index + delta as usize;
            if n >= len {
                if self.settings.navigation.wrap_around {
                    0
                } else {
                    len - 1
                }
            } else {
                n
            }
        } else {
            let back = (-delta) as usize;
            if main.current_index < back {
                if self.settings.navigation.wrap_around {
                    len - 1
                } else {
                    0
                }
            } else {
                main.current_index - back
            }
        };
        if next != main.current_index {
            main.current_index = next;
            main.current_page = 0;
            main.viewport.reset_transforms();
            self.load_current_entry();
        }
    }

    fn spawn_child_window(&mut self, event_loop: &ActiveEventLoop, kind: ChildWindowKind) {
        if let Some(existing) = self.child_windows.values().find(|c| c.kind == kind) {
            existing.window.focus_window();
            return;
        }
        let Some(gpu) = self.gpu.as_ref() else {
            return;
        };
        let title = match kind {
            ChildWindowKind::ImageList => I18n::tr(self.settings.language, "dialog.image_list"),
            ChildWindowKind::Settings => I18n::tr(self.settings.language, "dialog.settings"),
            ChildWindowKind::About => I18n::tr(self.settings.language, "dialog.about"),
        };
        let attrs = WindowAttributes::default()
            .with_title(title)
            .with_inner_size(LogicalSize::new(560.0, 440.0));
        if let Ok(win) = event_loop.create_window(attrs) {
            let arc = Arc::new(win);
            if let Ok(gpu_surface) = gpu.create_child_surface(Arc::clone(&arc)) {
                let egui_ctx = egui::Context::default();
                let egui_state = egui_winit::State::new(
                    egui_ctx.clone(),
                    egui::ViewportId::from_hash_of(title),
                    &arc,
                    Some(arc.scale_factor() as f32),
                    None,
                    None,
                );
                let wid = arc.id();
                self.child_windows.insert(
                    wid,
                    ChildWindowState {
                        kind,
                        window: arc,
                        gpu_surface,
                        egui_state,
                        egui_ctx,
                    },
                );
            }
        }
    }
}

impl ApplicationHandler<AppUserEvent> for ViewerApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.main_window.is_some() {
            return;
        }
        let mut attrs = WindowAttributes::default()
            .with_title("ImageViewer")
            .with_decorations(false)
            .with_transparent(false)
            .with_inner_size(LogicalSize::new(
                self.settings.window.width,
                self.settings.window.height,
            ))
            .with_maximized(self.settings.window.is_maximized);

        if ChromeState::should_restore_window_position(Self::is_wayland())
            && let (Some(left), Some(top)) = (self.settings.window.left, self.settings.window.top)
        {
            attrs = attrs.with_position(LogicalPosition::new(left, top));
        }

        let Ok(win) = event_loop.create_window(attrs) else {
            event_loop.exit();
            return;
        };
        let arc_win = Arc::new(win);
        let Ok((gpu, gpu_surface)) = GpuRenderer::new_for_window(Arc::clone(&arc_win)) else {
            event_loop.exit();
            return;
        };
        self.gpu = Some(gpu);

        let egui_ctx = egui::Context::default();
        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &arc_win,
            Some(arc_win.scale_factor() as f32),
            None,
            None,
        );
        let size = arc_win.inner_size();
        let scale = arc_win.scale_factor();
        let mut chrome = ChromeState::default();
        chrome.top_bar_pinned = self.settings.chrome.top_bar_pinned;
        chrome.bottom_bar_pinned = self.settings.chrome.bottom_bar_pinned;

        let cpus = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);

        self.main_window = Some(MainWindowState {
            window: arc_win,
            gpu_surface,
            egui_state,
            egui_ctx,
            tiles: Vec::new(),
            viewport: ViewportState::new(
                f64::from(size.width) / scale,
                f64::from(size.height) / scale,
                scale,
            ),
            chrome,
            trackpad: TrackpadAccumulator::default(),
            modifiers: ModifiersState::empty(),
            source: None,
            current_index: 0,
            current_page: 0,
            current_frame: None,
            frame_cache: DecodedFrameCache::new(8),
            session: SessionCoordinator::new(cpus),
            slideshow_playing: false,
            status_error: self.initial_cli.error.clone(),
            start_instant: Instant::now(),
        });

        if let CliCommand::OpenPath(path) = self.initial_cli.command.clone() {
            self.open_target_path(&path);
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: AppUserEvent) {
        match event {
            AppUserEvent::PickedOpenFile(Some(p)) | AppUserEvent::PickedOpenFolder(Some(p)) => {
                self.open_target_path(&p);
            }
            AppUserEvent::OpenUrlPaths(paths) => {
                if let Some(first) = paths.first() {
                    self.open_target_path(first);
                }
            }
            _ => {}
        }
        if let Some(main) = self.main_window.as_ref() {
            main.window.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        if let Some(child) = self.child_windows.get_mut(&window_id) {
            let _ = child.egui_state.on_window_event(&child.window, &event);
            match event {
                WindowEvent::CloseRequested => {
                    self.child_windows.remove(&window_id);
                }
                WindowEvent::Resized(new_size) => {
                    if new_size.width > 0 && new_size.height > 0 {
                        child.gpu_surface.config.width = new_size.width;
                        child.gpu_surface.config.height = new_size.height;
                        child.gpu_surface.needs_reconfigure = true;
                    }
                }
                WindowEvent::RedrawRequested => {
                    let mut close_child = false;
                    let mut save_settings = false;
                    let mut jump_index = None;
                    if let Some(gpu) = self.gpu.as_ref() {
                        let raw_input = child.egui_state.take_egui_input(&child.window);
                        let lang = self.settings.language;
                        let mut draft_settings = self.settings.clone();
                        let entries = self
                            .main_window
                            .as_ref()
                            .and_then(|m| m.source.as_ref())
                            .map(|s| s.entries().to_vec())
                            .unwrap_or_default();
                        let cur_idx = self.main_window.as_ref().map(|m| m.current_index).unwrap_or(0);
                        let kind = child.kind;

                        let full = child.egui_ctx.run(raw_input, |ctx| match kind {
                            ChildWindowKind::ImageList => {
                                if let Some(idx) = draw_image_list(ctx, lang, &entries, cur_idx) {
                                    jump_index = Some(idx);
                                }
                            }
                            ChildWindowKind::Settings => match draw_settings_window(ctx, &mut draft_settings) {
                                SettingsDialogAction::SaveAndClose => {
                                    save_settings = true;
                                    close_child = true;
                                }
                                SettingsDialogAction::Cancel => {
                                    close_child = true;
                                }
                                SettingsDialogAction::None => {}
                            },
                            ChildWindowKind::About => {
                                let _ = draw_about_window(ctx, lang);
                            }
                        });
                        if save_settings {
                            draft_settings.normalize();
                            self.settings = draft_settings;
                            let _ = SettingsStore::save_preferences(&OsPaths::settings_file(), &self.settings);
                        }
                        child.egui_state.handle_platform_output(&child.window, full.platform_output);
                        let prims = child.egui_ctx.tessellate(full.shapes, full.pixels_per_point);
                        gpu.render_frame(
                            &mut child.gpu_surface,
                            &[],
                            1,
                            1,
                            self.settings.display.background,
                            SimdResizer::compute_filter_uniforms(FilterMode::None, 0.5, 1.0, 1.0, 1.0, false),
                            Some((&child.egui_ctx, prims, full.textures_delta, full.pixels_per_point)),
                        );
                    }
                    if close_child {
                        self.child_windows.remove(&window_id);
                    }
                    if let Some(idx) = jump_index
                        && let Some(main) = self.main_window.as_mut()
                    {
                        main.current_index = idx;
                        main.current_page = 0;
                        main.viewport.reset_transforms();
                        self.load_current_entry();
                    }
                }
                _ => {}
            }
            return;
        }

        let Some(main) = self.main_window.as_mut() else {
            return;
        };
        if main.window.id() != window_id {
            return;
        }
        let consumed = main.egui_state.on_window_event(&main.window, &event).consumed;

        match event {
            WindowEvent::CloseRequested => {
                let scale = main.window.scale_factor();
                let size = main.window.inner_size();
                self.settings.window.width = f64::from(size.width) / scale;
                self.settings.window.height = f64::from(size.height) / scale;
                self.settings.window.is_maximized = main.window.is_maximized();
                if let Ok(pos) = main.window.outer_position() {
                    self.settings.window.left = Some(f64::from(pos.x) / scale);
                    self.settings.window.top = Some(f64::from(pos.y) / scale);
                }
                let _ = SettingsStore::save_window_geometry(
                    &OsPaths::settings_file(),
                    &self.settings.window,
                );
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if new_size.width > 0 && new_size.height > 0 {
                    main.gpu_surface.config.width = new_size.width;
                    main.gpu_surface.config.height = new_size.height;
                    main.gpu_surface.needs_reconfigure = true;
                    let scale = main.window.scale_factor();
                    main.viewport.logical_width = f64::from(new_size.width) / scale;
                    main.viewport.logical_height = f64::from(new_size.height) / scale;
                }
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                main.viewport.dpi_scale_factor = scale_factor;
                main.gpu_surface.needs_reconfigure = true;
            }
            WindowEvent::ModifiersChanged(mods) => {
                main.modifiers = mods.state();
            }
            WindowEvent::DroppedFile(path) => {
                self.open_target_path(&path);
            }
            WindowEvent::PinchGesture { delta, .. } => {
                let factor = TrackpadAccumulator::pinch_delta_to_zoom_factor(delta);
                let cx = main.viewport.logical_width * 0.5;
                let cy = main.viewport.logical_height * 0.5;
                main.viewport.zoom_at_cursor(factor, cx, cy);
                main.window.request_redraw();
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let invert = main.modifiers.control_key() || main.modifiers.super_key();
                let action = match (self.settings.display.mouse_wheel, invert) {
                    (MouseWheelAction::Navigate, false) | (MouseWheelAction::Zoom, true) => {
                        MouseWheelAction::Navigate
                    }
                    _ => MouseWheelAction::Zoom,
                };
                match (delta, action) {
                    (MouseScrollDelta::LineDelta(_, y), MouseWheelAction::Navigate) => {
                        if y < 0.0 {
                            self.step_image(1);
                        } else if y > 0.0 {
                            self.step_image(-1);
                        }
                    }
                    (MouseScrollDelta::PixelDelta(pos), MouseWheelAction::Navigate) => {
                        let now_ms = main.start_instant.elapsed().as_millis() as u64;
                        let step = main.trackpad.accumulate_scroll_step(-pos.y, now_ms);
                        if step != 0 {
                            self.step_image(step as isize);
                        }
                    }
                    (MouseScrollDelta::LineDelta(_, y), MouseWheelAction::Zoom) => {
                        let f = if y > 0.0 { 1.1 } else { 0.9 };
                        let cx = main.viewport.logical_width * 0.5;
                        let cy = main.viewport.logical_height * 0.5;
                        main.viewport.zoom_at_cursor(f, cx, cy);
                    }
                    (MouseScrollDelta::PixelDelta(pos), MouseWheelAction::Zoom) => {
                        let f = (1.0 + pos.y * 0.002).clamp(0.8, 1.25);
                        let cx = main.viewport.logical_width * 0.5;
                        let cy = main.viewport.logical_height * 0.5;
                        main.viewport.zoom_at_cursor(f, cx, cy);
                    }
                }
                if let Some(m) = self.main_window.as_ref() {
                    m.window.request_redraw();
                }
            }
            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } if !consumed => {
                if !main.chrome.is_fullscreen && main.viewport.zoom_scale <= 1.01 {
                    let _ = main.window.drag_window();
                }
            }
            WindowEvent::KeyboardInput { event: key_ev, .. } if key_ev.state == ElementState::Pressed => {
                let primary_mod = if cfg!(target_os = "macos") {
                    main.modifiers.super_key()
                } else {
                    main.modifiers.control_key()
                };
                match &key_ev.logical_key {
                    Key::Named(NamedKey::Escape) => event_loop.exit(),
                    Key::Named(NamedKey::ArrowRight) | Key::Named(NamedKey::ArrowDown) => self.step_image(1),
                    Key::Named(NamedKey::ArrowLeft) | Key::Named(NamedKey::ArrowUp) => self.step_image(-1),
                    Key::Named(NamedKey::Home) => {
                        main.current_index = 0;
                        self.load_current_entry();
                    }
                    Key::Named(NamedKey::End) => {
                        if let Some(src) = main.source.as_ref()
                            && !src.entries().is_empty()
                        {
                            main.current_index = src.entries().len() - 1;
                            self.load_current_entry();
                        }
                    }
                    Key::Named(NamedKey::Tab) => {
                        main.chrome.info_panel_visible = !main.chrome.info_panel_visible;
                    }
                    Key::Named(NamedKey::Enter) if main.modifiers.alt_key() => {
                        let next_fs = if main.window.fullscreen().is_some() {
                            None
                        } else {
                            Some(Fullscreen::Borderless(None))
                        };
                        main.chrome.is_fullscreen = next_fs.is_some();
                        main.window.set_fullscreen(next_fs);
                    }
                    Key::Named(NamedKey::Enter) => {
                        self.spawn_child_window(event_loop, ChildWindowKind::ImageList);
                    }
                    Key::Named(NamedKey::Space) => {
                        main.slideshow_playing = !main.slideshow_playing;
                    }
                    Key::Character(c) => match c.as_str() {
                        "`" => main.chrome.toggle_all_pins(),
                        "r" | "R" => main.viewport.rotate_cw(),
                        "l" | "L" => main.viewport.rotate_ccw(),
                        "h" | "H" => main.viewport.toggle_flip_h(),
                        "v" | "V" => main.viewport.toggle_flip_v(),
                        "0" if primary_mod => main.viewport.reset_transforms(),
                        "0" => main.viewport.display_mode = DisplayMode::Original,
                        "7" => main.viewport.display_mode = DisplayMode::FitWidth,
                        "8" => main.viewport.display_mode = DisplayMode::FitHeight,
                        "9" => main.viewport.display_mode = DisplayMode::Fill,
                        "o" | "O" if primary_mod && main.modifiers.shift_key() => {
                            OsDialogs::pick_folder_async(self.proxy.clone());
                        }
                        "o" | "O" if primary_mod => {
                            OsDialogs::pick_image_or_archive_async(self.proxy.clone());
                        }
                        _ => {}
                    },
                    _ => {}
                }
                if let Some(m) = self.main_window.as_ref() {
                    m.window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                let mut ui_action = MainUiAction::None;
                if let Some(gpu) = self.gpu.as_ref() {
                    let current_name = main
                        .source
                        .as_ref()
                        .and_then(|s| s.entries().get(main.current_index))
                        .map(|e| e.name.as_str())
                        .unwrap_or("");
                    let total = main.source.as_ref().map(|s| s.entries().len()).unwrap_or(0);
                    let raw_input = main.egui_state.take_egui_input(&main.window);
                    let full = main.egui_ctx.run(raw_input, |ctx| {
                        ui_action = draw_main_overlays(
                            ctx,
                            self.settings.language,
                            &main.chrome,
                            &main.viewport,
                            main.current_frame.as_deref(),
                            current_name,
                            main.current_index,
                            total,
                            main.slideshow_playing,
                            main.status_error.as_deref(),
                        );
                    });
                    main.egui_state.handle_platform_output(&main.window, full.platform_output);
                    let prims = main.egui_ctx.tessellate(full.shapes, full.pixels_per_point);
                    let (tw, th) = main
                        .current_frame
                        .as_ref()
                        .map(|f| (f.buffer_width, f.buffer_height))
                        .unwrap_or((1, 1));
                    let filters = SimdResizer::compute_filter_uniforms(
                        self.settings.filter.mode,
                        self.settings.filter.smooth_sigma,
                        self.settings.filter.sharpen_sigma,
                        self.settings.filter.sharpen_amount,
                        main.viewport.zoom_scale,
                        false,
                    );
                    gpu.render_frame(
                        &mut main.gpu_surface,
                        &main.tiles,
                        tw,
                        th,
                        self.settings.display.background,
                        filters,
                        Some((&main.egui_ctx, prims, full.textures_delta, full.pixels_per_point)),
                    );
                }

                match ui_action {
                    MainUiAction::CloseWindow => event_loop.exit(),
                    MainUiAction::Minimize => main.window.set_minimized(true),
                    MainUiAction::ToggleMaximize => {
                        main.window.set_maximized(!main.window.is_maximized());
                    }
                    MainUiAction::DragWindow => {
                        let _ = main.window.drag_window();
                    }
                    MainUiAction::OpenFile => OsDialogs::pick_image_or_archive_async(self.proxy.clone()),
                    MainUiAction::OpenFolder => OsDialogs::pick_folder_async(self.proxy.clone()),
                    MainUiAction::PrevImage => self.step_image(-1),
                    MainUiAction::NextImage => self.step_image(1),
                    MainUiAction::TogglePinAll => main.chrome.toggle_all_pins(),
                    MainUiAction::ToggleExif => {
                        main.chrome.info_panel_visible = !main.chrome.info_panel_visible;
                    }
                    MainUiAction::OpenImageList => {
                        self.spawn_child_window(event_loop, ChildWindowKind::ImageList);
                    }
                    MainUiAction::OpenSettings => {
                        self.spawn_child_window(event_loop, ChildWindowKind::Settings);
                    }
                    MainUiAction::OpenAbout => {
                        self.spawn_child_window(event_loop, ChildWindowKind::About);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

fn main() {
    let _ = draw_update_notice;
    let args: Vec<String> = std::env::args().collect();
    let cli = CliOptions::parse(&args, Language::En, cfg!(target_os = "windows"));

    if matches!(cli.command, CliCommand::RegisterShell | CliCommand::UnregisterShell)
        && cli.error.is_some()
    {
        eprintln!("{}", cli.error.unwrap_or_default());
        std::process::exit(1);
    }

    let Ok(event_loop) = EventLoop::<AppUserEvent>::with_user_event().build() else {
        std::process::exit(1);
    };
    let proxy = event_loop.create_proxy();
    let mut app = ViewerApp::new(proxy, cli);
    let _ = event_loop.run_app(&mut app);
}
