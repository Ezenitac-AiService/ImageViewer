//! Owned `winit 0.30` `ApplicationHandler` + custom `wgpu` canvas + `egui-winit`/`egui-wgpu` multi-window shell.

mod renderer;
mod views;

use crate::renderer::{GpuRenderer, GpuTile, WindowGpuSurface};
use crate::views::about_view::draw_about_window;
use crate::views::image_list_view::draw_image_list;
use crate::views::main_view::{draw_main_overlays, MainUiAction};
use crate::views::settings_view::{draw_settings_window, SettingsDialogAction};
use crate::views::update_notice::{draw_update_notice, UpdateNoticeAction};
use crate::views::ChildWindowKind;
use iv_core::chrome_state::ChromeState;
use iv_core::cli::{CliCommand, CliOptions};
use iv_core::i18n::Language;
use iv_core::layout::ViewportState;
use iv_core::logger::AsyncFileLogger;
use iv_core::settings::{
    DisplayMode, FilterMode, MouseWheelAction, SettingsStore, SortDirection, SortField,
    ViewerSettings,
};
use iv_core::slideshow::{SlideShowController, SystemClock};
use iv_core::trackpad::TrackpadAccumulator;
use iv_media::animation::AnimationPlayer;
use iv_media::cache::{DecodedFrame, DecodedFrameCache};
use iv_media::decode::{DecodeOptions, MediaDecoder};
use iv_media::resize_filter::SimdResizer;
use iv_media::session::SessionCoordinator;
use iv_media::sources::folder::FolderSource;
use iv_media::sources::watcher::FolderWatcher;
use iv_media::sources::zip::ZipSource;
use iv_media::sources::MediaSource;
use iv_platform::codecs::HostOsCodecProvider;
use iv_platform::integration::OsIntegration;
use iv_platform::paths_dialogs::{AppUserEvent, OsDialogs, OsPaths};
use iv_platform::update::{ReleaseInfo, ReqwestTlsTransport, UpdateClient};
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::keyboard::{Key, ModifiersState, NamedKey};
use winit::window::{Fullscreen, Window, WindowAttributes, WindowId};

struct AsyncDecodeMsg {
    load_gen: u64,
    is_prefetch: bool,
    cache_key: String,
    item_name: String,
    elapsed: Duration,
    result: Result<(DecodedFrame, Option<AnimationPlayer>), String>,
}

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
    source: Option<Arc<dyn MediaSource>>,
    watcher: Option<FolderWatcher>,
    current_path: Option<std::path::PathBuf>,
    current_index: usize,
    current_page: u32,
    current_frame: Option<Arc<DecodedFrame>>,
    animation: Option<AnimationPlayer>,
    anim_last_step: Instant,
    last_step_instant: Instant,
    frame_cache: DecodedFrameCache,
    session: SessionCoordinator,
    decode_tx: mpsc::Sender<AsyncDecodeMsg>,
    decode_rx: mpsc::Receiver<AsyncDecodeMsg>,
    update_tx: mpsc::Sender<Option<ReleaseInfo>>,
    update_rx: mpsc::Receiver<Option<ReleaseInfo>>,
    slideshow: SlideShowController,
    middle_dragging: bool,
    last_cursor: (f64, f64),
    pending_load_started: Option<Instant>,
    status_error: Option<String>,
    pending_release: Option<ReleaseInfo>,
    start_instant: Instant,
    auto_check_dispatched: bool,
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
    sort_field: SortField,
    sort_dir: SortDirection,
}

impl ViewerApp {
    fn new(proxy: EventLoopProxy<AppUserEvent>, initial_cli: CliOptions) -> Self {
        let settings = SettingsStore::load(&OsPaths::settings_file());
        let logger = AsyncFileLogger::new(&OsPaths::logs_dir(), 256).ok();
        let sort_field = settings.navigation.sort_field;
        let sort_dir = settings.navigation.sort_direction;
        Self {
            proxy,
            settings,
            logger,
            gpu: None,
            main_window: None,
            child_windows: HashMap::new(),
            initial_cli,
            sort_field,
            sort_dir,
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

    fn install_cjk_fallback_fonts(ctx: &egui::Context) {
        let candidates = [
            "/System/Library/Fonts/AppleSDGothicNeo.ttc",
            "C:\\Windows\\Fonts\\malgun.ttf",
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        ];
        for path in candidates {
            if let Ok(bytes) = std::fs::read(path) {
                let mut fonts = egui::FontDefinitions::default();
                fonts.font_data.insert(
                    "iv_cjk".to_string(),
                    Arc::new(egui::FontData::from_owned(bytes)),
                );
                if let Some(fam) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                    fam.push("iv_cjk".to_string());
                }
                ctx.set_fonts(fonts);
                break;
            }
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

        let is_zip = ext == "zip" || ext == "cbz";
        let source_res: Result<Arc<dyn MediaSource>, String> = if is_zip {
            ZipSource::open(path, self.settings.limits.max_zip_entry_megabytes)
                .map(|z| Arc::new(z) as Arc<dyn MediaSource>)
                .map_err(|e| e.to_string())
        } else {
            FolderSource::open(path)
                .map(|f| Arc::new(f) as Arc<dyn MediaSource>)
                .map_err(|e| e.to_string())
        };

        match source_res {
            Ok(src) => {
                let idx = src.initial_index();
                main.source = Some(src);
                main.current_path = Some(path.to_path_buf());
                main.current_index = idx;
                main.current_page = 0;
                main.status_error = None;
                main.watcher = if !is_zip && self.settings.navigation.watch_folder {
                    let watch_dir = if path.is_dir() {
                        path
                    } else {
                        path.parent().unwrap_or(path)
                    };
                    FolderWatcher::watch(watch_dir, Duration::from_millis(700)).ok()
                } else {
                    None
                };
                self.dispatch_async_load(false);
            }
            Err(err) => {
                if let Some(main) = self.main_window.as_mut() {
                    main.status_error = Some(err);
                }
            }
        }
    }

    fn dispatch_async_load(&mut self, force_full_res: bool) {
        let (Some(main), Some(gpu)) = (self.main_window.as_mut(), self.gpu.as_ref()) else {
            return;
        };
        let Some(src) = main.source.as_ref().map(Arc::clone) else {
            return;
        };
        if src.entries().is_empty() {
            return;
        }
        let idx = main.current_index.min(src.entries().len() - 1);
        let entry = src.entries()[idx].clone();
        let cache_key = format!("{}:{}:{force_full_res}", entry.source_key, main.current_page);

        if let Some(cached) = main.frame_cache.get(&cache_key) {
            main.tiles = gpu.upload_tiled_rgba8(&cached.rgba8, cached.buffer_width, cached.buffer_height);
            main.current_frame = Some(cached);
            main.status_error = None;
            main.pending_load_started = None;
            return;
        }

        let load_gen = main.session.begin_load(idx);
        main.pending_load_started = Some(Instant::now());
        main.animation = None;
        let is_burst = main.last_step_instant.elapsed() <= Duration::from_millis(600);
        main.last_step_instant = Instant::now();
        let active_gen = Arc::clone(&main.session.load_gen);
        let tx = main.decode_tx.clone();
        let proxy = self.proxy.clone();
        let page = main.current_page;
        let max_mp = self.settings.limits.max_image_megapixels;
        let play_anims = self.settings.display.play_animations;
        let target_vp = if force_full_res || main.viewport.display_mode == DisplayMode::Original {
            None
        } else {
            Some((main.viewport.physical_width(), main.viewport.physical_height()))
        };

        let task = move || {
            if active_gen.load(Ordering::SeqCst) != load_gen {
                return;
            }
            let t0 = Instant::now();
            let result = match src.read_entry_bytes(idx) {
                Ok(bytes) => {
                    if active_gen.load(Ordering::SeqCst) != load_gen {
                        return;
                    }
                    let ext = Path::new(&entry.name)
                        .extension()
                        .and_then(|e| e.to_str())
                        .map(|s| s.to_ascii_lowercase())
                        .unwrap_or_default();
                    if play_anims && AnimationPlayer::is_animated_container(&bytes, &ext) {
                        match AnimationPlayer::from_bytes_guarded(&bytes, &entry.name, max_mp) {
                            Ok(player) => {
                                let first = (*player.current_frame_arc()).clone();
                                Ok((first, Some(player)))
                            }
                            Err(err) => Err(err.to_string()),
                        }
                    } else {
                        let opts = DecodeOptions {
                            max_megapixels: max_mp,
                            target_viewport_physical: target_vp,
                            page_index: page,
                        };
                        let os_provider = HostOsCodecProvider;
                        MediaDecoder::decode_bytes_with_os(&bytes, &entry.name, &opts, Some(&os_provider))
                            .map(|f| (f, None))
                            .map_err(|e| e.to_string())
                    }
                }
                Err(err) => Err(err.to_string()),
            };
            let _ = tx.send(AsyncDecodeMsg {
                load_gen,
                is_prefetch: false,
                cache_key,
                item_name: entry.name,
                elapsed: t0.elapsed(),
                result,
            });
            let _ = proxy.send_event(AppUserEvent::DecodeWakeup);
        };

        if let Some(pool) = main.session.pool.as_ref() {
            pool.spawn(task);
        } else {
            std::thread::spawn(task);
        }

        // Dispatch directional +1, +2, -1 (burst +4) background prefetching
        let prefetch_indices = main
            .session
            .prefetch_targets(idx, main.source.as_ref().map(|s| s.entries().len()).unwrap_or(0), is_burst);
        if let Some(src_arc) = main.source.as_ref().map(Arc::clone) {
            for p_idx in prefetch_indices {
                let p_entry = src_arc.entries()[p_idx].clone();
                let p_key = format!("{}:0:false", p_entry.source_key);
                if main.frame_cache.get(&p_key).is_some() {
                    continue;
                }
                let p_tx = main.decode_tx.clone();
                let p_proxy = self.proxy.clone();
                let p_src = Arc::clone(&src_arc);
                let p_task = move || {
                    let t0 = Instant::now();
                    if let Ok(bytes) = p_src.read_entry_bytes(p_idx) {
                        let opts = DecodeOptions {
                            max_megapixels: max_mp,
                            target_viewport_physical: target_vp,
                            page_index: 0,
                        };
                        let os_provider = HostOsCodecProvider;
                        if let Ok(frame) = MediaDecoder::decode_bytes_with_os(
                            &bytes,
                            &p_entry.name,
                            &opts,
                            Some(&os_provider),
                        ) {
                            let _ = p_tx.send(AsyncDecodeMsg {
                                load_gen,
                                is_prefetch: true,
                                cache_key: p_key,
                                item_name: p_entry.name,
                                elapsed: t0.elapsed(),
                                result: Ok((frame, None)),
                            });
                            let _ = p_proxy.send_event(AppUserEvent::DecodeWakeup);
                        }
                    }
                };
                if let Some(pool) = main.session.pool.as_ref() {
                    pool.spawn(p_task);
                }
            }
        }
    }

    fn drain_async_decodes(&mut self) {
        let (Some(main), Some(gpu)) = (self.main_window.as_mut(), self.gpu.as_ref()) else {
            return;
        };
        while let Ok(msg) = main.decode_rx.try_recv() {
            if msg.is_prefetch {
                if let Ok((frame, _)) = msg.result {
                    main.frame_cache.insert(msg.cache_key, Arc::new(frame));
                }
                continue;
            }
            if !main.session.is_load_current(msg.load_gen) {
                continue;
            }
            main.pending_load_started = None;
            match msg.result {
                Ok((frame, anim_opt)) => {
                    let arc = Arc::new(frame);
                    main.tiles = gpu.upload_tiled_rgba8(
                        &arc.rgba8,
                        arc.buffer_width,
                        arc.buffer_height,
                    );
                    main.frame_cache.insert(msg.cache_key, Arc::clone(&arc));
                    main.current_frame = Some(arc);
                    main.animation = anim_opt;
                    main.anim_last_step = Instant::now();
                    main.status_error = None;
                    if let Some(log) = self.logger.as_ref() {
                        log.log_slow_render(&msg.item_name, msg.elapsed);
                    }
                }
                Err(err) => {
                    main.status_error = Some(err);
                }
            }
            main.window.request_redraw();
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
            main.slideshow.notify_manual_navigation();
            self.dispatch_async_load(false);
        }
    }

    fn step_page_or_image(&mut self, delta: isize) {
        let Some(main) = self.main_window.as_mut() else {
            return;
        };
        let page_count = main.current_frame.as_ref().map(|f| f.page_count).unwrap_or(1);
        if page_count > 1 {
            let next_page = if delta > 0 {
                (main.current_page + 1).min(page_count - 1)
            } else {
                main.current_page.saturating_sub(1)
            };
            if next_page != main.current_page {
                main.current_page = next_page;
                // Preserve user R/L/H/V transforms across multi-page TIFF PgUp/PgDn
                self.dispatch_async_load(false);
                return;
            }
        }
        self.step_image(delta);
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
            ChildWindowKind::ImageList => iv_core::i18n::I18n::tr(self.settings.language, "dialog.image_list"),
            ChildWindowKind::Settings => iv_core::i18n::I18n::tr(self.settings.language, "dialog.settings"),
            ChildWindowKind::About => iv_core::i18n::I18n::tr(self.settings.language, "dialog.about"),
        };
        let attrs = WindowAttributes::default()
            .with_title(title)
            .with_inner_size(LogicalSize::new(560.0, 440.0));
        if let Ok(win) = event_loop.create_window(attrs) {
            let arc = Arc::new(win);
            if let Ok(gpu_surface) = gpu.create_child_surface(Arc::clone(&arc)) {
                let egui_ctx = egui::Context::default();
                Self::install_cjk_fallback_fonts(&egui_ctx);
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
        Self::install_cjk_fallback_fonts(&egui_ctx);
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
        let (decode_tx, decode_rx) = mpsc::channel();
        let (update_tx, update_rx) = mpsc::channel();
        let slideshow = SlideShowController::new(
            Box::new(SystemClock),
            self.settings.slideshow.interval_seconds,
            self.settings.slideshow.mode,
            self.settings.slideshow.repeat,
        );

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
            watcher: None,
            current_path: None,
            current_index: 0,
            current_page: 0,
            current_frame: None,
            animation: None,
            anim_last_step: Instant::now(),
            last_step_instant: Instant::now(),
            frame_cache: DecodedFrameCache::new(8),
            session: SessionCoordinator::new(cpus),
            decode_tx,
            decode_rx,
            update_tx,
            update_rx,
            slideshow,
            middle_dragging: false,
            last_cursor: (0.0, 0.0),
            pending_load_started: None,
            status_error: self.initial_cli.error.clone(),
            pending_release: None,
            start_instant: Instant::now(),
            auto_check_dispatched: false,
        });

        if let CliCommand::OpenPath(path) = self.initial_cli.command.clone() {
            self.open_target_path(&path);
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::WaitUntil(
            Instant::now() + Duration::from_millis(16),
        ));
        self.drain_async_decodes();

        let (ss_next, reload_path, anim_redraw) = if let (Some(main), Some(gpu)) = (self.main_window.as_mut(), self.gpu.as_ref()) {
            // Advance animated playback on wall-clock delay
            let mut anim_redraw = false;
            if self.settings.display.play_animations
                && let Some(anim) = main.animation.as_mut()
                && main.anim_last_step.elapsed() >= anim.current_delay()
            {
                anim.advance();
                main.anim_last_step = Instant::now();
                let next_frame = anim.current_frame_arc();
                main.tiles = gpu.upload_tiled_rgba8(
                    &next_frame.rgba8,
                    next_frame.buffer_width,
                    next_frame.buffer_height,
                );
                main.current_frame = Some(next_frame);
                anim_redraw = true;
            }

            // Receive completed startup AutoCheck updates
            while let Ok(rel_opt) = main.update_rx.try_recv() {
                let now_utc = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64;
                self.settings.update.last_check_utc = Some(now_utc);
                let _ = SettingsStore::save_preferences(&OsPaths::settings_file(), &self.settings);
                if let Some(rel) = rel_opt
                    && is_newer_version(&rel.version, env!("CARGO_PKG_VERSION"))
                    && self.settings.update.skipped_version.as_deref() != Some(rel.version.as_str())
                {
                    main.pending_release = Some(rel);
                    anim_redraw = true;
                }
            }

            let total = main.source.as_ref().map(|s| s.entries().len()).unwrap_or(0);
            let ss_next = main.slideshow.poll_advance(main.current_index, total);
            let reload = if main
                .watcher
                .as_ref()
                .is_some_and(|w| w.dirty.swap(false, Ordering::SeqCst))
            {
                main.current_path.clone()
            } else {
                None
            };

            let now_utc = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;
            let cadence_due = self
                .settings
                .update
                .last_check_utc
                .is_none_or(|prev| now_utc.saturating_sub(prev) >= 20 * 3600);
            if !main.auto_check_dispatched
                && self.settings.update.auto_check
                && cadence_due
                && main.start_instant.elapsed() >= Duration::from_secs(5)
            {
                main.auto_check_dispatched = true;
                let proxy = self.proxy.clone();
                let u_tx = main.update_tx.clone();
                std::thread::spawn(move || {
                    let found = if let Ok(transport) = ReqwestTlsTransport::new() {
                        UpdateClient::check_latest_with_transport(
                            &transport,
                            "https://api.github.com/repos/ImageViewer/ImageViewer/releases/latest",
                            env!("CARGO_PKG_VERSION"),
                        )
                        .ok()
                    } else {
                        None
                    };
                    let _ = u_tx.send(found);
                    let _ = proxy.send_event(AppUserEvent::UpdateChecked);
                });
            }
            (ss_next, reload, anim_redraw)
        } else {
            (None, None, false)
        };

        if anim_redraw && let Some(main) = self.main_window.as_ref() {
            main.window.request_redraw();
        }
        if let Some(next_idx) = ss_next
            && let Some(main) = self.main_window.as_mut()
        {
            main.current_index = next_idx;
            main.current_page = 0;
            main.viewport.reset_transforms();
            self.dispatch_async_load(false);
        }
        if let Some(path) = reload_path {
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
            AppUserEvent::DecodeWakeup => {
                self.drain_async_decodes();
            }
            AppUserEvent::UpdateChecked => {}
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
                        let mut sf = self.sort_field;
                        let mut sd = self.sort_dir;

                        let full = child.egui_ctx.run(raw_input, |ctx| match kind {
                            ChildWindowKind::ImageList => {
                                if let Some(idx) = draw_image_list(ctx, lang, &entries, cur_idx, &mut sf, &mut sd) {
                                    jump_index = Some(idx);
                                }
                            }
                            ChildWindowKind::Settings => match draw_settings_window(ctx, &mut draft_settings) {
                                SettingsDialogAction::SaveAndClose => {
                                    save_settings = true;
                                    close_child = true;
                                }
                                SettingsDialogAction::Cancel => close_child = true,
                                SettingsDialogAction::None => {}
                            },
                            ChildWindowKind::About => {
                                if draw_about_window(ctx, lang) {
                                    let _ = UpdateClient::open_https_release_page(
                                        "https://github.com/ImageViewer/ImageViewer/releases",
                                    );
                                }
                            }
                        });
                        self.sort_field = sf;
                        self.sort_dir = sd;
                        if save_settings {
                            draft_settings.normalize();
                            self.settings = draft_settings;
                            let _ = SettingsStore::save_preferences(&OsPaths::settings_file(), &self.settings);
                        }
                        child.egui_state.handle_platform_output(&child.window, full.platform_output);
                        let prims = child.egui_ctx.tessellate(full.shapes, full.pixels_per_point);
                        let dummy_vp = ViewportState::new(560.0, 440.0, 1.0);
                        gpu.render_frame(
                            &mut child.gpu_surface,
                            &[],
                            1,
                            1,
                            &dummy_vp,
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
                        self.dispatch_async_load(false);
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
        let mut trigger_fullres = false;

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
            WindowEvent::CursorMoved { position, .. } => {
                let scale = main.viewport.dpi_scale_factor.max(1.0);
                let lx = position.x / scale;
                let ly = position.y / scale;
                main.chrome.hover_top = ly <= 72.0;
                main.chrome.hover_bottom = ly >= (main.viewport.logical_height - 72.0);
                if main.middle_dragging {
                    main.viewport.pan_x += lx - main.last_cursor.0;
                    main.viewport.pan_y += ly - main.last_cursor.1;
                    main.window.request_redraw();
                }
                main.last_cursor = (lx, ly);
            }
            WindowEvent::DroppedFile(path) => {
                self.open_target_path(&path);
            }
            WindowEvent::PinchGesture { delta, .. } => {
                let factor = TrackpadAccumulator::pinch_delta_to_zoom_factor(delta);
                main.viewport.zoom_at_cursor(factor, main.last_cursor.0, main.last_cursor.1);
                if main.current_frame.as_ref().is_some_and(|f| f.is_downscaled) && main.viewport.zoom_scale > 1.2 {
                    trigger_fullres = true;
                }
                main.window.request_redraw();
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if main.modifiers.shift_key() {
                    let dy = match delta {
                        MouseScrollDelta::LineDelta(_, y) => f64::from(y) * 32.0,
                        MouseScrollDelta::PixelDelta(pos) => pos.y,
                    };
                    main.viewport.pan_y += dy;
                    main.window.request_redraw();
                    return;
                }
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
                        main.viewport.zoom_at_cursor(f, main.last_cursor.0, main.last_cursor.1);
                        if main.current_frame.as_ref().is_some_and(|fr| fr.is_downscaled) && main.viewport.zoom_scale > 1.2 {
                            trigger_fullres = true;
                        }
                    }
                    (MouseScrollDelta::PixelDelta(pos), MouseWheelAction::Zoom) => {
                        let f = (1.0 + pos.y * 0.002).clamp(0.8, 1.25);
                        main.viewport.zoom_at_cursor(f, main.last_cursor.0, main.last_cursor.1);
                    }
                }
                if let Some(m) = self.main_window.as_ref() {
                    m.window.request_redraw();
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                match (state, button) {
                    (ElementState::Pressed, MouseButton::Left) if !consumed => {
                        if !main.chrome.is_fullscreen && main.viewport.zoom_scale <= 1.01 {
                            let _ = main.window.drag_window();
                        }
                    }
                    (ElementState::Pressed, MouseButton::Middle) => main.middle_dragging = true,
                    (ElementState::Released, MouseButton::Middle) => main.middle_dragging = false,
                    (ElementState::Pressed, MouseButton::Back) => self.step_image(-1),
                    (ElementState::Pressed, MouseButton::Forward) => self.step_image(1),
                    _ => {}
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
                    Key::Named(NamedKey::PageDown) => self.step_page_or_image(1),
                    Key::Named(NamedKey::PageUp) => self.step_page_or_image(-1),
                    Key::Named(NamedKey::Home) => {
                        main.current_index = 0;
                        main.current_page = 0;
                        self.dispatch_async_load(false);
                    }
                    Key::Named(NamedKey::End) => {
                        if let Some(src) = main.source.as_ref()
                            && !src.entries().is_empty()
                        {
                            main.current_index = src.entries().len() - 1;
                            main.current_page = 0;
                            self.dispatch_async_load(false);
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
                        if main.slideshow.playing {
                            main.slideshow.pause();
                        } else {
                            main.slideshow.play();
                        }
                    }
                    Key::Character(c) => match c.as_str() {
                        "`" => main.chrome.toggle_all_pins(),
                        "r" | "R" => main.viewport.rotate_cw(),
                        "l" | "L" => main.viewport.rotate_ccw(),
                        "h" | "H" => main.viewport.toggle_flip_h(),
                        "v" | "V" => main.viewport.toggle_flip_v(),
                        "u" | "U" => self.settings.filter.mode = FilterMode::None,
                        "i" | "I" => self.settings.filter.mode = FilterMode::Smooth,
                        "s" | "S" => self.settings.filter.mode = FilterMode::SmoothSharpen,
                        "+" | "=" => {
                            let cx = main.viewport.logical_width * 0.5;
                            let cy = main.viewport.logical_height * 0.5;
                            main.viewport.zoom_at_cursor(1.2, cx, cy);
                            if main.current_frame.as_ref().is_some_and(|f| f.is_downscaled) {
                                trigger_fullres = true;
                            }
                        }
                        "-" => {
                            let cx = main.viewport.logical_width * 0.5;
                            let cy = main.viewport.logical_height * 0.5;
                            main.viewport.zoom_at_cursor(0.83, cx, cy);
                        }
                        "f" | "F" if cfg!(target_os = "macos") && main.modifiers.control_key() && main.modifiers.super_key() => {
                            let next_fs = if main.window.fullscreen().is_some() {
                                None
                            } else {
                                Some(Fullscreen::Borderless(None))
                            };
                            main.chrome.is_fullscreen = next_fs.is_some();
                            main.window.set_fullscreen(next_fs);
                        }
                        "0" if primary_mod => main.viewport.reset_transforms(),
                        "0" => {
                            main.viewport.display_mode = DisplayMode::Original;
                            if main.current_frame.as_ref().is_some_and(|f| f.is_downscaled) {
                                trigger_fullres = true;
                            }
                        }
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
                let mut notice_action = UpdateNoticeAction::None;
                if let Some(gpu) = self.gpu.as_ref() {
                    let current_name = main
                        .source
                        .as_ref()
                        .and_then(|s| s.entries().get(main.current_index))
                        .map(|e| e.name.as_str())
                        .unwrap_or("");
                    let entries = main.source.as_ref().map(|s| s.entries()).unwrap_or(&[]);
                    let total = entries.len();
                    let anim_status = main.animation.as_ref().map(|a| {
                        (
                            a.current_index(),
                            a.frame_count(),
                            a.current_delay().as_millis(),
                            self.settings.display.play_animations,
                        )
                    });
                    let loading_spinner = main.pending_load_started.and_then(|t0| {
                        let elapsed = t0.elapsed();
                        if SessionCoordinator::should_show_spinner(elapsed) {
                            Some(true)
                        } else if SessionCoordinator::should_show_placeholder(
                            elapsed,
                            main.viewport.display_mode == DisplayMode::Original,
                        ) {
                            Some(false)
                        } else {
                            None
                        }
                    });
                    let raw_input = main.egui_state.take_egui_input(&main.window);
                    let full = main.egui_ctx.run(raw_input, |ctx| {
                        ui_action = draw_main_overlays(
                            ctx,
                            self.settings.language,
                            &main.chrome,
                            &main.viewport,
                            &self.settings.thumbnail,
                            entries,
                            main.current_frame.as_deref(),
                            anim_status,
                            loading_spinner,
                            current_name,
                            main.current_index,
                            total,
                            main.slideshow.playing,
                            main.status_error.as_deref(),
                        );
                        if let Some(rel) = main.pending_release.as_ref() {
                            notice_action = draw_update_notice(ctx, self.settings.language, rel, None);
                        }
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
                        main.animation.is_some(),
                    );
                    gpu.render_frame(
                        &mut main.gpu_surface,
                        &main.tiles,
                        tw,
                        th,
                        &main.viewport,
                        self.settings.display.background,
                        filters,
                        Some((&main.egui_ctx, prims, full.textures_delta, full.pixels_per_point)),
                    );
                }

                match notice_action {
                    UpdateNoticeAction::SkipVersion(v) => {
                        self.settings.update.skipped_version = Some(v);
                        let _ = SettingsStore::save_preferences(&OsPaths::settings_file(), &self.settings);
                        if let Some(m) = self.main_window.as_mut() {
                            m.pending_release = None;
                        }
                    }
                    UpdateNoticeAction::Dismiss => {
                        if let Some(m) = self.main_window.as_mut() {
                            m.pending_release = None;
                        }
                    }
                    UpdateNoticeAction::Install => {
                        if let Some(rel) = self.main_window.as_ref().and_then(|m| m.pending_release.as_ref()) {
                            let _ = UpdateClient::open_https_release_page(&rel.page_url);
                        }
                    }
                    UpdateNoticeAction::None => {}
                }

                match ui_action {
                    MainUiAction::CloseWindow => event_loop.exit(),
                    MainUiAction::Minimize => {
                        if let Some(m) = self.main_window.as_ref() {
                            m.window.set_minimized(true);
                        }
                    }
                    MainUiAction::ToggleMaximize => {
                        if let Some(m) = self.main_window.as_ref() {
                            m.window.set_maximized(!m.window.is_maximized());
                        }
                    }
                    MainUiAction::DragWindow => {
                        if let Some(m) = self.main_window.as_ref() {
                            let _ = m.window.drag_window();
                        }
                    }
                    MainUiAction::OpenFile => OsDialogs::pick_image_or_archive_async(self.proxy.clone()),
                    MainUiAction::OpenFolder => OsDialogs::pick_folder_async(self.proxy.clone()),
                    MainUiAction::PrevImage => self.step_image(-1),
                    MainUiAction::NextImage => self.step_image(1),
                    MainUiAction::SelectIndex(idx) => {
                        if let Some(m) = self.main_window.as_mut() {
                            m.current_index = idx;
                            m.current_page = 0;
                            m.viewport.reset_transforms();
                            self.dispatch_async_load(false);
                        }
                    }
                    MainUiAction::ZoomIn => {
                        if let Some(m) = self.main_window.as_mut() {
                            let cx = m.viewport.logical_width * 0.5;
                            let cy = m.viewport.logical_height * 0.5;
                            m.viewport.zoom_at_cursor(1.2, cx, cy);
                        }
                    }
                    MainUiAction::ZoomOut => {
                        if let Some(m) = self.main_window.as_mut() {
                            let cx = m.viewport.logical_width * 0.5;
                            let cy = m.viewport.logical_height * 0.5;
                            m.viewport.zoom_at_cursor(0.83, cx, cy);
                        }
                    }
                    MainUiAction::ZoomReset => {
                        if let Some(m) = self.main_window.as_mut() {
                            m.viewport.reset_transforms();
                        }
                    }
                    MainUiAction::TogglePinAll => {
                        if let Some(m) = self.main_window.as_mut() {
                            m.chrome.toggle_all_pins();
                        }
                    }
                    MainUiAction::ToggleExif => {
                        if let Some(m) = self.main_window.as_mut() {
                            m.chrome.info_panel_visible = !m.chrome.info_panel_visible;
                        }
                    }
                    MainUiAction::ToggleSlideshow => {
                        if let Some(m) = self.main_window.as_mut() {
                            if m.slideshow.playing {
                                m.slideshow.pause();
                            } else {
                                m.slideshow.play();
                            }
                        }
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
                    MainUiAction::None => {}
                }
            }
            _ => {}
        }

        if trigger_fullres {
            self.dispatch_async_load(true);
        }
    }
}

fn is_newer_version(candidate: &str, current: &str) -> bool {
    let parse = |s: &str| -> (u64, u64, u64) {
        let mut parts = s.trim_start_matches('v').split('.');
        let a = parts.next().and_then(|x| x.parse().ok()).unwrap_or(0);
        let b = parts.next().and_then(|x| x.parse().ok()).unwrap_or(0);
        let c = parts.next().and_then(|x| x.parse().ok()).unwrap_or(0);
        (a, b, c)
    };
    parse(candidate) > parse(current)
}

fn main() {
    if let Ok(exe) = std::env::current_exe() {
        UpdateClient::cleanup_old_exe_on_startup(&exe);
    }

    let args: Vec<String> = std::env::args().collect();
    let cli = CliOptions::parse(&args, Language::En, cfg!(target_os = "windows"));

    match cli.command {
        CliCommand::RegisterShell => {
            if cli.error.is_some() {
                eprintln!("{}", cli.error.unwrap_or_default());
                std::process::exit(1);
            }
            if let Ok(exe) = std::env::current_exe() {
                let _ = OsIntegration::register_windows_shell(&exe.to_string_lossy());
            }
            return;
        }
        CliCommand::UnregisterShell => {
            if cli.error.is_some() {
                eprintln!("{}", cli.error.unwrap_or_default());
                std::process::exit(1);
            }
            let _ = OsIntegration::unregister_windows_shell();
            return;
        }
        _ => {}
    }

    let Ok(event_loop) = EventLoop::<AppUserEvent>::with_user_event().build() else {
        std::process::exit(1);
    };
    let proxy = event_loop.create_proxy();
    OsIntegration::install_macos_open_url_handler(proxy.clone());
    let mut app = ViewerApp::new(proxy, cli);
    let _ = event_loop.run_app(&mut app);
}
