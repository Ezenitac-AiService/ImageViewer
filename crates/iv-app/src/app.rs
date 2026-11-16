//! `eframe::App` shell: translates egui input into `iv_core::interaction` commands, drives
//! loading, and paints the canvas, toolbars, and dialogs.

use crate::canvas::Canvas;
use crate::loader::{LoadRequest, Loaded, Loader};
use crate::views::about_view::draw_about_window;
use crate::views::image_list_view::{ImageListAction, draw_image_list};
use crate::views::settings_view::{SettingsDialogAction, draw_settings_window};
use crate::views::status::{self, StatusMessage};
use crate::views::toolbar::{ToolbarState, bottom_bar, top_bar};
use eframe::egui::{self, Rect, Sense};
use iv_core::diagnostics::{LaunchRecord, Phase, source_name};
use iv_core::i18n::I18n;
use iv_core::interaction::{
    DialogKind, ExtendedFeatures, InputEvent, InteractionContext, Key, Mods, Outcome,
    PointerButtonKind, ViewerCommand, ViewerModel, WheelUnit, map_input,
};
use iv_core::layout::ViewportLayoutCalculator;
use iv_core::logger::AsyncFileLogger;
use iv_core::settings::{
    DisplayMode, SettingsStore, SortDirection, SortField, ViewerSettings, WindowGeometry,
};
use iv_core::slideshow::{SlideShowController, SystemClock};
use iv_core::trackpad::TrackpadAccumulator;
use iv_media::animation::AnimationPlayer;
use iv_media::cache::DecodedFrame;
use iv_media::sources::folder::FolderSource;
use iv_media::sources::watcher::FolderWatcher;
use iv_media::sources::zip::ZipSource;
use iv_media::sources::{MediaSource, SourceError};
use iv_platform::paths_dialogs::OsDialogs;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};

/// Extended-candidate capabilities reachable in this build (capability register, T086).
pub const EXTENDED: ExtendedFeatures = ExtendedFeatures {
    image_list: true,
    info_panel: true,
    slideshow: true,
};

/// Startup inputs handed from `main` to the app.
pub struct AppInit {
    pub settings: ViewerSettings,
    pub settings_path: PathBuf,
    pub logger: Option<Arc<AsyncFileLogger>>,
    pub record: LaunchRecord,
    pub open_path: Option<PathBuf>,
    pub initial_status: Option<StatusMessage>,
}

pub struct ViewerApp {
    settings: ViewerSettings,
    settings_path: PathBuf,
    logger: Option<Arc<AsyncFileLogger>>,
    record: LaunchRecord,
    model: ViewerModel,
    canvas: Canvas,
    loader: Loader,
    source: Option<Arc<dyn MediaSource>>,
    source_path: Option<PathBuf>,
    watcher: Option<FolderWatcher>,
    frame: Option<Arc<DecodedFrame>>,
    animation: Option<AnimationPlayer>,
    anim_next_at: Option<Instant>,
    status: Option<StatusMessage>,
    trackpad: TrackpadAccumulator,
    slideshow: SlideShowController,
    started: Instant,
    picks_tx: mpsc::Sender<PathBuf>,
    picks_rx: mpsc::Receiver<PathBuf>,
    settings_draft: Option<ViewerSettings>,
    sort_field: SortField,
    sort_dir: SortDirection,
    first_frame_logged: bool,
    requested_first: Option<String>,
    last_geometry: Option<WindowGeometry>,
}

impl ViewerApp {
    pub fn new(cc: &eframe::CreationContext<'_>, init: AppInit) -> Self {
        crate::fonts::install_cjk_fallback(&cc.egui_ctx);
        let (picks_tx, picks_rx) = mpsc::channel();
        let mut model = ViewerModel::new(1280.0, 800.0, 1.0);
        model.wrap_around = init.settings.navigation.wrap_around;
        model.filter = init.settings.filter.mode;
        model.viewport.display_mode = init.settings.display.mode;
        let slideshow = SlideShowController::new(
            Box::new(SystemClock),
            init.settings.slideshow.interval_seconds,
            init.settings.slideshow.mode,
            init.settings.slideshow.repeat,
        );
        let mut app = Self {
            sort_field: init.settings.navigation.sort_field,
            sort_dir: init.settings.navigation.sort_direction,
            settings: init.settings,
            settings_path: init.settings_path,
            logger: init.logger,
            record: init.record,
            model,
            canvas: Canvas::default(),
            loader: Loader::new(cc.egui_ctx.clone()),
            source: None,
            source_path: None,
            watcher: None,
            frame: None,
            animation: None,
            anim_next_at: None,
            status: init.initial_status,
            trackpad: TrackpadAccumulator::default(),
            slideshow,
            started: Instant::now(),
            picks_tx,
            picks_rx,
            settings_draft: None,
            first_frame_logged: false,
            requested_first: None,
            last_geometry: None,
        };
        if let Some(path) = init.open_path {
            app.requested_first = Some(source_name(&path.to_string_lossy()));
            app.open_path(&path);
        } else {
            app.log_phase(Phase::FirstFrame, &[("source", "none")]);
            app.first_frame_logged = true;
        }
        app
    }

    fn lang(&self) -> iv_core::i18n::Language {
        self.settings.language
    }

    fn log_phase(&self, phase: Phase, fields: &[(&str, &str)]) {
        if let Some(log) = &self.logger {
            log.write_sync("INFO", &self.record.phase(phase, fields));
        }
    }

    fn log_error(&self, message: &str) {
        if let Some(log) = &self.logger {
            log.log_error(&format!("launch={} {message}", self.record.id().as_str()));
        }
    }

    // ------------------------------------------------------------------ opening

    fn open_path(&mut self, path: &Path) {
        let is_zip = path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("zip") || e.eq_ignore_ascii_case("cbz"));
        let opened: Result<Arc<dyn MediaSource>, SourceError> = if is_zip {
            ZipSource::open(path, self.settings.limits.max_zip_entry_megabytes)
                .map(|z| Arc::new(z) as Arc<dyn MediaSource>)
        } else {
            FolderSource::open(path).map(|f| Arc::new(f) as Arc<dyn MediaSource>)
        };
        match opened {
            Ok(src) => {
                let initial = src.initial_index();
                self.model
                    .set_source_len(src.entries().len(), Some(initial));
                self.model.viewport.reset_transforms();
                self.source = Some(src);
                self.source_path = Some(path.to_path_buf());
                self.loader.clear_cache();
                self.status = None;
                self.watcher = if !is_zip && self.settings.navigation.watch_folder {
                    let dir = if path.is_dir() {
                        path
                    } else {
                        path.parent().unwrap_or(path)
                    };
                    match FolderWatcher::watch(dir, Duration::from_millis(700)) {
                        Ok(w) => Some(w),
                        Err(e) => {
                            self.log_error(&format!("watcher unavailable: {e}"));
                            None
                        }
                    }
                } else {
                    None
                };
                self.load_current(false);
            }
            Err(err) => {
                self.log_error(&format!("open failed: {err}"));
                self.status = Some(StatusMessage::Error(err.user_message(self.lang())));
            }
        }
    }

    fn rescan_keeping_position(&mut self) {
        let Some(path) = self.source_path.clone() else {
            return;
        };
        let current_key = self.current_entry_key();
        let old_index = self.model.current;
        let Ok(src) = FolderSource::open(&path) else {
            return;
        };
        let src: Arc<dyn MediaSource> = Arc::new(src);
        let rebound = current_key
            .as_deref()
            .and_then(|key| src.entries().iter().position(|e| e.source_key == key));
        let changed_current = rebound.is_none();
        self.model
            .set_source_len(src.entries().len(), rebound.or(old_index));
        self.source = Some(src);
        if changed_current {
            self.load_current(false);
        }
    }

    fn current_entry_key(&self) -> Option<String> {
        let src = self.source.as_ref()?;
        let idx = self.model.current?;
        src.entries().get(idx).map(|e| e.source_key.clone())
    }

    fn load_current(&mut self, full_resolution: bool) {
        let (Some(src), Some(index)) = (self.source.clone(), self.model.current) else {
            self.frame = None;
            self.animation = None;
            self.canvas.clear();
            return;
        };
        let vp = &self.model.viewport;
        let req = LoadRequest {
            index,
            page: self.model.page,
            full_resolution: full_resolution || vp.display_mode == DisplayMode::Original,
            target_viewport: Some((vp.physical_width(), vp.physical_height())),
            max_megapixels: self.settings.limits.max_image_megapixels,
            play_animations: self.settings.display.play_animations,
        };
        self.animation = None;
        self.anim_next_at = None;
        if let Some(cached) = self.loader.request(&src, req) {
            self.show_frame(cached, None);
        }
    }

    fn show_frame(&mut self, frame: Arc<DecodedFrame>, anim: Option<AnimationPlayer>) {
        let ctx = self.loader.ctx().clone();
        self.canvas.set_frame(&ctx, &frame, self.model.filter);
        self.model.page_count = frame.page_count.max(1);
        self.anim_next_at = anim.as_ref().map(|a| Instant::now() + a.current_delay());
        self.animation = anim;
        if !self.first_frame_logged {
            self.first_frame_logged = true;
            let name = self
                .requested_first
                .clone()
                .unwrap_or_else(|| "none".into());
            let (w, h) = (
                frame.source_width.to_string(),
                frame.source_height.to_string(),
            );
            self.log_phase(
                Phase::FirstFrame,
                &[
                    ("source", name.as_str()),
                    ("w", w.as_str()),
                    ("h", h.as_str()),
                ],
            );
        }
        self.frame = Some(frame);
    }

    fn handle_loaded(&mut self, loaded: Vec<Loaded>) {
        for item in loaded {
            if Some(item.index) != self.model.current {
                continue;
            }
            match item.result {
                Ok((frame, anim)) => {
                    if let Some(log) = &self.logger {
                        log.log_slow_render(&item.item_name, item.elapsed);
                    }
                    self.status = None;
                    self.show_frame(frame, anim);
                }
                Err(err) => {
                    self.log_error(&format!(
                        "decode failed item={} error={}",
                        item.item_name,
                        err.detail()
                    ));
                    self.status = Some(StatusMessage::Error(err.user_message(self.lang())));
                    self.canvas.clear();
                    self.frame = None;
                }
            }
        }
    }

    // ------------------------------------------------------------------ commands

    fn run(&mut self, ctx: &egui::Context, cmd: ViewerCommand) {
        let index_before = self.model.current;
        match self.model.apply(cmd) {
            Outcome::Nothing | Outcome::ViewChanged => {}
            Outcome::ImageChanged => {
                self.slideshow.notify_manual_navigation();
                self.after_index_change(index_before);
            }
            Outcome::PageChanged => self.load_current(false),
            Outcome::FullscreenChanged(on) => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(on));
            }
            Outcome::External(ext) => self.run_external(ctx, ext),
        }
        if self.model.filter != self.canvas.filter() {
            self.canvas.set_filter(self.model.filter);
            if let Some(frame) = self.frame.clone() {
                self.canvas.set_frame(ctx, &frame, self.model.filter);
            }
        }
        if self.model.slideshow_playing != self.slideshow.playing {
            if self.model.slideshow_playing {
                self.slideshow.play();
            } else {
                self.slideshow.pause();
            }
        }
    }

    fn after_index_change(&mut self, before: Option<usize>) {
        if before == self.model.current {
            return;
        }
        if let (Some(i), total) = (self.model.current, self.model.entries_len) {
            let (i, total) = (i.to_string(), total.to_string());
            self.log_phase(
                Phase::Navigate,
                &[("index", i.as_str()), ("total", total.as_str())],
            );
        }
        self.load_current(false);
    }

    fn run_external(&mut self, ctx: &egui::Context, cmd: ViewerCommand) {
        match cmd {
            ViewerCommand::RequestOpenFile => {
                let tx = self.picks_tx.clone();
                let repaint = ctx.clone();
                OsDialogs::pick_image_or_archive_async(
                    I18n::tr(self.lang(), "menu.open_file"),
                    I18n::tr(self.lang(), "picker.images"),
                    Box::new(move |picked| {
                        if let Some(p) = picked
                            && tx.send(p).is_ok()
                        {
                            repaint.request_repaint();
                        }
                    }),
                );
            }
            ViewerCommand::RequestOpenFolder => {
                let tx = self.picks_tx.clone();
                let repaint = ctx.clone();
                OsDialogs::pick_folder_async(
                    I18n::tr(self.lang(), "menu.open_folder"),
                    Box::new(move |picked| {
                        if let Some(p) = picked
                            && tx.send(p).is_ok()
                        {
                            repaint.request_repaint();
                        }
                    }),
                );
            }
            ViewerCommand::OpenPath(p) => self.open_path(&p),
            ViewerCommand::ScrollNavigate { delta } => {
                let now_ms = u64::try_from(self.started.elapsed().as_millis()).unwrap_or(u64::MAX);
                let step = self.trackpad.accumulate_scroll_step(delta, now_ms);
                let cmd = match step.signum() {
                    1 => Some(ViewerCommand::Next),
                    -1 => Some(ViewerCommand::Prev),
                    _ => None,
                };
                if let Some(cmd) = cmd {
                    self.run(ctx, cmd);
                }
            }
            ViewerCommand::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            other => self.log_error(&format!("unexpected external command {other:?}")),
        }
    }

    // ------------------------------------------------------------------ input

    fn collect_input(&self, ctx: &egui::Context, canvas: &egui::Response) -> Vec<InputEvent> {
        let mut events = Vec::new();
        let pointer_on_canvas = canvas.hovered();
        ctx.input(|i| {
            for ev in &i.events {
                match ev {
                    egui::Event::Key {
                        key,
                        pressed: true,
                        modifiers,
                        ..
                    } => {
                        if let Some(k) = map_key(*key) {
                            events.push(InputEvent::Key {
                                key: k,
                                mods: mods(*modifiers),
                            });
                        }
                    }
                    egui::Event::MouseWheel {
                        unit,
                        delta,
                        modifiers,
                        ..
                    } if pointer_on_canvas => {
                        let unit = match unit {
                            egui::MouseWheelUnit::Point => WheelUnit::Pixel,
                            egui::MouseWheelUnit::Line | egui::MouseWheelUnit::Page => {
                                WheelUnit::Line
                            }
                        };
                        events.push(InputEvent::Wheel {
                            delta_y: f64::from(delta.y),
                            unit,
                            mods: mods(*modifiers),
                        });
                    }
                    egui::Event::Zoom(factor) if pointer_on_canvas => {
                        events.push(InputEvent::Pinch {
                            factor: f64::from(*factor),
                        });
                    }
                    egui::Event::PointerButton {
                        button,
                        pressed: true,
                        ..
                    } if pointer_on_canvas => match button {
                        egui::PointerButton::Extra1 => {
                            events.push(InputEvent::PointerButton(PointerButtonKind::Back));
                        }
                        egui::PointerButton::Extra2 => {
                            events.push(InputEvent::PointerButton(PointerButtonKind::Forward));
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
            for file in &i.raw.dropped_files {
                let path = file.path();
                if !path.as_os_str().is_empty() {
                    events.push(InputEvent::FileDropped(path.to_path_buf()));
                }
            }
        });
        if canvas.double_clicked() {
            events.push(InputEvent::DoubleClickCanvas);
        }
        let drag = canvas.drag_delta();
        if drag != egui::Vec2::ZERO {
            let (dx, dy) = (f64::from(drag.x), f64::from(drag.y));
            if canvas.dragged_by(egui::PointerButton::Middle) {
                events.push(InputEvent::DragMiddle { dx, dy });
            } else if canvas.dragged_by(egui::PointerButton::Primary) {
                events.push(InputEvent::DragPrimary { dx, dy });
            }
        }
        events
    }

    fn interaction_context(&self, ctx: &egui::Context, area: Rect) -> InteractionContext {
        let cursor = ctx
            .input(|i| i.pointer.hover_pos())
            .unwrap_or(area.center());
        let image_exceeds_viewport = self.frame.as_ref().is_some_and(|f| {
            let q = ViewportLayoutCalculator::image_quad(
                f.source_width,
                f.source_height,
                &self.model.viewport,
            );
            q.width > f64::from(area.width()) + 0.5 || q.height > f64::from(area.height()) + 0.5
        });
        InteractionContext {
            is_macos: cfg!(target_os = "macos"),
            dialogs_open: self.model.dialogs.len(),
            text_focus: ctx.egui_wants_keyboard_input(),
            fullscreen: self.model.fullscreen,
            wheel_action: self.settings.display.mouse_wheel,
            image_exceeds_viewport,
            cursor: (
                f64::from(cursor.x - area.min.x),
                f64::from(cursor.y - area.min.y),
            ),
            viewport_center: (
                f64::from(area.width()) * 0.5,
                f64::from(area.height()) * 0.5,
            ),
            extended: EXTENDED,
        }
    }

    // ------------------------------------------------------------------ periodic work

    fn tick(&mut self, ctx: &egui::Context) {
        while let Ok(path) = self.picks_rx.try_recv() {
            self.open_path(&path);
        }
        let loaded = self.loader.poll();
        self.handle_loaded(loaded);

        if self
            .watcher
            .as_ref()
            .is_some_and(|w| w.dirty.swap(false, Ordering::SeqCst))
        {
            self.rescan_keeping_position();
        }

        if let Some(next_at) = self.anim_next_at {
            let now = Instant::now();
            if now >= next_at {
                if let Some(anim) = self.animation.as_mut() {
                    anim.advance();
                    let frame = anim.current_frame_arc();
                    let delay = anim.current_delay();
                    self.canvas.set_frame(ctx, &frame, self.model.filter);
                    self.frame = Some(frame);
                    self.anim_next_at = Some(now + delay);
                    ctx.request_repaint_after(delay);
                }
            } else {
                ctx.request_repaint_after(next_at - now);
            }
        }

        if self.slideshow.playing {
            if let Some(next) = self
                .slideshow
                .poll_advance(self.model.current.unwrap_or(0), self.model.entries_len)
            {
                self.run(ctx, ViewerCommand::SelectIndex(next));
            }
            ctx.request_repaint_after(Duration::from_millis(250));
        }
        if self.loader.pending_since.is_some() {
            ctx.request_repaint_after(Duration::from_millis(100));
        }

        ctx.input(|i| {
            let info = i.viewport();
            if let Some(inner) = info.inner_rect {
                let maximized = info.maximized.unwrap_or(false);
                let outer = info.outer_rect.unwrap_or(inner);
                if info.fullscreen != Some(true) {
                    self.last_geometry = Some(WindowGeometry {
                        left: Some(f64::from(outer.min.x)),
                        top: Some(f64::from(outer.min.y)),
                        width: f64::from(inner.width()),
                        height: f64::from(inner.height()),
                        is_maximized: maximized,
                    });
                }
            }
        });
    }

    fn title(&self) -> String {
        match (&self.source, self.model.current) {
            (Some(src), Some(i)) => src
                .entries()
                .get(i)
                .map(|e| e.name.clone())
                .unwrap_or_default(),
            _ => String::new(),
        }
    }

    fn draw_dialogs(&mut self, ctx: &egui::Context) {
        for kind in self.model.dialogs.clone() {
            match kind {
                DialogKind::Settings => {
                    let draft = self
                        .settings_draft
                        .get_or_insert_with(|| self.settings.clone());
                    match draw_settings_window(ctx, draft) {
                        SettingsDialogAction::None => {}
                        SettingsDialogAction::Cancel => {
                            self.settings_draft = None;
                            self.close_dialog(DialogKind::Settings);
                        }
                        SettingsDialogAction::SaveAndClose => {
                            if let Some(mut draft) = self.settings_draft.take() {
                                draft.normalize();
                                self.apply_settings(draft);
                            }
                            self.close_dialog(DialogKind::Settings);
                        }
                    }
                }
                DialogKind::About => {
                    if !draw_about_window(ctx, self.lang()) {
                        self.close_dialog(DialogKind::About);
                    }
                }
                DialogKind::ImageList => {
                    let entries = self
                        .source
                        .as_ref()
                        .map(|s| s.entries().to_vec())
                        .unwrap_or_default();
                    match draw_image_list(
                        ctx,
                        self.lang(),
                        &entries,
                        self.model.current,
                        &mut self.sort_field,
                        &mut self.sort_dir,
                    ) {
                        ImageListAction::None => {}
                        ImageListAction::Close => self.close_dialog(DialogKind::ImageList),
                        ImageListAction::Select(i) => self.run(ctx, ViewerCommand::SelectIndex(i)),
                    }
                }
            }
        }
    }

    fn close_dialog(&mut self, kind: DialogKind) {
        self.model.dialogs.retain(|d| *d != kind);
    }

    fn apply_settings(&mut self, new: ViewerSettings) {
        let reload = new.limits.max_image_megapixels != self.settings.limits.max_image_megapixels
            || new.display.play_animations != self.settings.display.play_animations;
        self.settings = new;
        self.model.wrap_around = self.settings.navigation.wrap_around;
        self.slideshow.interval_seconds = self.settings.slideshow.interval_seconds;
        self.slideshow.repeat = self.settings.slideshow.repeat;
        if let Err(e) = SettingsStore::save_preferences(&self.settings_path, &self.settings) {
            self.log_error(&format!("settings_save_failed error={e}"));
            self.status = Some(StatusMessage::Error(
                I18n::tr(self.lang(), "settings.save_failed").to_string(),
            ));
        }
        if reload {
            self.loader.clear_cache();
            self.load_current(false);
        }
    }

    fn draw_info_panel(&self, ui: &mut egui::Ui) {
        let Some(frame) = self.frame.as_ref() else {
            return;
        };
        egui::Panel::right("iv_info_panel")
            .resizable(true)
            .default_size(260.0)
            .show(ui, |ui| {
                ui.heading(I18n::tr(self.lang(), "menu.exif"));
                ui.separator();
                ui.label(format!(
                    "{} × {} · {}",
                    frame.source_width, frame.source_height, frame.format_label
                ));
                if frame.page_count > 1 {
                    ui.label(format!("{} / {}", frame.page_index + 1, frame.page_count));
                }
                if let Some(anim) = &self.animation {
                    ui.label(format!(
                        "{} / {} · {} ms",
                        anim.current_index() + 1,
                        anim.frame_count(),
                        anim.current_delay().as_millis()
                    ));
                }
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for item in &frame.exif_items {
                        ui.horizontal_wrapped(|ui| {
                            ui.strong(item.label_key);
                            ui.label(&item.value);
                        });
                    }
                });
            });
    }
}

impl eframe::App for ViewerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.tick(&ctx);

        let lang = self.lang();
        let mut commands = Vec::new();
        if !self.model.fullscreen {
            let title = self.title();
            let state = ToolbarState {
                lang,
                zoom_percent: self.model.viewport.zoom_scale * 100.0,
                position: self.model.current,
                total: self.model.entries_len,
                title: &title,
                extended: EXTENDED,
                slideshow_playing: self.model.slideshow_playing,
                info_panel: self.model.info_panel,
            };
            commands.extend(top_bar(ui, &state));
            let center = (
                self.model.viewport.logical_width * 0.5,
                self.model.viewport.logical_height * 0.5,
            );
            commands.extend(bottom_bar(ui, &state, center));
        }
        if self.model.info_panel {
            self.draw_info_panel(ui);
        }

        let area = ui.available_rect_before_wrap();
        let response = ui.allocate_rect(area, Sense::click_and_drag());
        let ppp = f64::from(ctx.pixels_per_point());
        let vp = &mut self.model.viewport;
        vp.logical_width = f64::from(area.width()).max(1.0);
        vp.logical_height = f64::from(area.height()).max(1.0);
        vp.dpi_scale_factor = ppp;

        self.canvas.paint(
            ui.painter(),
            area,
            &self.model.viewport,
            self.settings.display.background,
        );
        if self.source.is_none() && self.status.is_none() {
            status::empty_hint(&ctx, I18n::tr(lang, "status.empty"));
        }
        if self
            .loader
            .pending_since
            .is_some_and(|t| t.elapsed() >= Duration::from_millis(250))
        {
            status::loading(&ctx, I18n::tr(lang, "status.loading"));
        }
        if let Some(msg) = self.status.clone()
            && status::message(&ctx, &msg)
        {
            self.status = None;
        }

        self.draw_dialogs(&ctx);

        let ictx = self.interaction_context(&ctx, area);
        for event in self.collect_input(&ctx, &response) {
            commands.extend(map_input(&event, &ictx));
        }
        let zoom_before = self.model.viewport.zoom_scale;
        for cmd in commands {
            self.run(&ctx, cmd);
        }
        // Zooming past fit on a downscaled preview loads full resolution (existing behaviour).
        if self.model.viewport.zoom_scale > 1.2
            && zoom_before <= 1.2
            && self.frame.as_ref().is_some_and(|f| f.is_downscaled)
        {
            self.load_current(true);
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Some(geometry) = self.last_geometry.clone() {
            self.settings.window = geometry;
        }
        if let Err(e) =
            SettingsStore::save_window_geometry(&self.settings_path, &self.settings.window)
        {
            self.log_error(&format!("settings_save_failed error={e}"));
        }
        if let Some(log) = &self.logger {
            log.write_sync("INFO", &self.record.exit_clean());
        }
    }
}

fn mods(m: egui::Modifiers) -> Mods {
    Mods {
        alt: m.alt,
        ctrl: m.ctrl,
        shift: m.shift,
        mac_cmd: m.mac_cmd,
    }
}

fn map_key(key: egui::Key) -> Option<Key> {
    use egui::Key as E;
    Some(match key {
        E::ArrowLeft => Key::ArrowLeft,
        E::ArrowRight => Key::ArrowRight,
        E::ArrowUp => Key::ArrowUp,
        E::ArrowDown => Key::ArrowDown,
        E::Home => Key::Home,
        E::End => Key::End,
        E::PageUp => Key::PageUp,
        E::PageDown => Key::PageDown,
        E::Enter => Key::Enter,
        E::Escape => Key::Escape,
        E::Tab => Key::Tab,
        E::Space => Key::Space,
        E::Plus => Key::Plus,
        E::Equals => Key::Equals,
        E::Minus => Key::Minus,
        E::Comma => Key::Comma,
        E::Num0 => Key::Num0,
        E::Num7 => Key::Num7,
        E::Num8 => Key::Num8,
        E::Num9 => Key::Num9,
        other => {
            let name = other.name();
            let mut chars = name.chars();
            match (chars.next(), chars.next()) {
                (Some(c), None) if c.is_ascii_alphabetic() => Key::Letter(c.to_ascii_uppercase()),
                _ => return None,
            }
        }
    })
}
