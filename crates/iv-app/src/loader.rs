//! Background decode and prefetch with generation-based cancellation.
//!
//! Workers hold an `egui::Context` and request a repaint after delivering a result, so the UI
//! wakes exactly when there is something new to show (research R3).

use eframe::egui;
use iv_core::i18n::Language;
use iv_media::animation::{AnimationError, AnimationPlayer};
use iv_media::cache::{DecodedFrame, DecodedFrameCache};
use iv_media::decode::{DecodeError, DecodeOptions, MediaDecoder};
use iv_media::session::SessionCoordinator;
use iv_media::sources::{MediaSource, SourceError};
use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};

/// What to load.
#[derive(Debug, Clone, Copy)]
pub struct LoadRequest {
    pub index: usize,
    pub page: u32,
    pub full_resolution: bool,
    /// Physical viewport size used for downscaled decoding; `None` decodes at full size.
    pub target_viewport: Option<(u32, u32)>,
    pub max_megapixels: u32,
    pub play_animations: bool,
}

/// Why a load failed. The user sees [`LoadFailure::user_message`]; the English detail goes to the
/// diagnostic record only (FR-018).
#[derive(Debug)]
pub enum LoadFailure {
    Source(SourceError),
    Decode(DecodeError),
    Animation(AnimationError),
}

impl LoadFailure {
    pub fn user_message(&self, lang: Language) -> String {
        match self {
            Self::Source(e) => e.user_message(lang),
            Self::Decode(e) => e.user_message(lang),
            Self::Animation(e) => e.user_message(lang),
        }
    }

    pub fn detail(&self) -> String {
        match self {
            Self::Source(e) => e.to_string(),
            Self::Decode(e) => e.to_string(),
            Self::Animation(e) => e.to_string(),
        }
    }
}

/// A finished decode for the current generation.
pub struct Loaded {
    pub index: usize,
    pub item_name: String,
    pub elapsed: Duration,
    pub result: Result<(Arc<DecodedFrame>, Option<AnimationPlayer>), LoadFailure>,
}

struct Message {
    generation: u64,
    is_prefetch: bool,
    cache_key: String,
    index: usize,
    item_name: String,
    elapsed: Duration,
    result: Result<(DecodedFrame, Option<AnimationPlayer>), LoadFailure>,
}

pub struct Loader {
    ctx: egui::Context,
    session: SessionCoordinator,
    cache: DecodedFrameCache,
    tx: mpsc::Sender<Message>,
    rx: mpsc::Receiver<Message>,
    last_request: Instant,
    pub pending_since: Option<Instant>,
}

impl Loader {
    pub fn new(ctx: egui::Context) -> Self {
        let cpus = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        let (tx, rx) = mpsc::channel();
        Self {
            ctx,
            session: SessionCoordinator::new(cpus),
            cache: DecodedFrameCache::new(8),
            tx,
            rx,
            last_request: Instant::now(),
            pending_since: None,
        }
    }

    pub fn ctx(&self) -> &egui::Context {
        &self.ctx
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Starts loading `req` from `src`. Returns the frame immediately when it is cached.
    pub fn request(
        &mut self,
        src: &Arc<dyn MediaSource>,
        req: LoadRequest,
    ) -> Option<Arc<DecodedFrame>> {
        let entries = src.entries();
        let entry = entries.get(req.index)?.clone();
        let cache_key = key_for(&entry.source_key, req.page, req.full_resolution);
        // Any request supersedes in-flight work for the previous image.
        let generation = self.session.begin_load(req.index);
        if let Some(cached) = self.cache.get(&cache_key) {
            self.pending_since = None;
            return Some(cached);
        }
        self.pending_since = Some(Instant::now());
        let is_burst = self.last_request.elapsed() <= Duration::from_millis(600);
        self.last_request = Instant::now();

        let active = Arc::clone(&self.session.load_gen);
        let tx = self.tx.clone();
        let ctx = self.ctx.clone();
        let src_main = Arc::clone(src);
        let task = move || {
            if active.load(Ordering::SeqCst) != generation {
                return;
            }
            let t0 = Instant::now();
            let result = decode_entry(&*src_main, req.index, &entry.name, req, &active, generation);
            let Some(result) = result else {
                return;
            };
            let msg = Message {
                generation,
                is_prefetch: false,
                cache_key,
                index: req.index,
                item_name: entry.name,
                elapsed: t0.elapsed(),
                result,
            };
            if tx.send(msg).is_ok() {
                ctx.request_repaint();
            }
        };
        self.spawn(task);

        // Directional prefetch of still frames (+1, +2, -1; +3, +4 when stepping quickly).
        for p_idx in self
            .session
            .prefetch_targets(req.index, entries.len(), is_burst)
        {
            let Some(p_entry) = entries.get(p_idx).cloned() else {
                continue;
            };
            let p_key = key_for(&p_entry.source_key, 0, false);
            if self.cache.get(&p_key).is_some() {
                continue;
            }
            let p_tx = self.tx.clone();
            let p_ctx = self.ctx.clone();
            let p_src = Arc::clone(src);
            let p_req = LoadRequest {
                index: p_idx,
                page: 0,
                full_resolution: false,
                play_animations: false,
                ..req
            };
            self.spawn(move || {
                let t0 = Instant::now();
                let Ok(bytes) = p_src.read_entry_bytes(p_idx) else {
                    return;
                };
                let Ok(frame) =
                    MediaDecoder::decode_bytes(&bytes, &p_entry.name, &decode_options(p_req))
                else {
                    return;
                };
                let msg = Message {
                    generation,
                    is_prefetch: true,
                    cache_key: p_key,
                    index: p_idx,
                    item_name: p_entry.name,
                    elapsed: t0.elapsed(),
                    result: Ok((frame, None)),
                };
                if p_tx.send(msg).is_ok() {
                    p_ctx.request_repaint();
                }
            });
        }
        None
    }

    /// Collects finished work. Prefetched frames go to the cache; stale results are dropped.
    pub fn poll(&mut self) -> Vec<Loaded> {
        let mut out = Vec::new();
        while let Ok(msg) = self.rx.try_recv() {
            if msg.is_prefetch {
                if let Ok((frame, _)) = msg.result {
                    self.cache.insert(msg.cache_key, Arc::new(frame));
                }
                continue;
            }
            if !self.session.is_load_current(msg.generation) {
                continue;
            }
            self.pending_since = None;
            let result = msg.result.map(|(frame, anim)| {
                let arc = Arc::new(frame);
                if anim.is_none() {
                    self.cache.insert(msg.cache_key, Arc::clone(&arc));
                }
                (arc, anim)
            });
            out.push(Loaded {
                index: msg.index,
                item_name: msg.item_name,
                elapsed: msg.elapsed,
                result,
            });
        }
        out
    }

    fn spawn(&self, task: impl FnOnce() + Send + 'static) {
        match self.session.pool.as_ref() {
            Some(pool) => pool.spawn(task),
            None => {
                std::thread::spawn(task);
            }
        }
    }
}

fn key_for(source_key: &str, page: u32, full: bool) -> String {
    format!("{source_key}:{page}:{full}")
}

fn decode_options(req: LoadRequest) -> DecodeOptions {
    DecodeOptions {
        max_megapixels: req.max_megapixels,
        target_viewport_physical: if req.full_resolution {
            None
        } else {
            req.target_viewport
        },
        page_index: req.page,
    }
}

/// Reads and decodes one entry. Returns `None` when the generation became stale mid-way.
fn decode_entry(
    src: &dyn MediaSource,
    index: usize,
    name: &str,
    req: LoadRequest,
    active: &std::sync::atomic::AtomicU64,
    generation: u64,
) -> Option<Result<(DecodedFrame, Option<AnimationPlayer>), LoadFailure>> {
    let bytes = match src.read_entry_bytes(index) {
        Ok(b) => b,
        Err(e) => return Some(Err(LoadFailure::Source(e))),
    };
    if active.load(Ordering::SeqCst) != generation {
        return None;
    }
    let ext = Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    if req.play_animations && AnimationPlayer::is_animated_container(&bytes, &ext) {
        return Some(
            AnimationPlayer::from_bytes_guarded(&bytes, name, req.max_megapixels)
                .map(|player| {
                    let first = (*player.current_frame_arc()).clone();
                    (first, Some(player))
                })
                .map_err(LoadFailure::Animation),
        );
    }
    Some(
        MediaDecoder::decode_bytes(&bytes, name, &decode_options(req))
            .map(|f| (f, None))
            .map_err(LoadFailure::Decode),
    )
}
