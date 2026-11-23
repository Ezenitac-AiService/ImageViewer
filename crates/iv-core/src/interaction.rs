//! Pure input → command mapping and viewer state model.
//!
//! Contract: `specs/004-launch-reliability-refactor/contracts/interaction-contract.md`.
//! Model: `specs/004-launch-reliability-refactor/data-model.md` §7.
//!
//! The UI layer translates toolkit input into [`InputEvent`], calls [`map_input`], and applies the
//! resulting [`ViewerCommand`]s with [`ViewerModel::apply`]. Nothing here depends on a window or a
//! GPU, so every rule is unit-testable without a screen (FR-036).

use crate::layout::ViewportState;
use crate::settings::{DisplayMode, FilterMode, MouseWheelAction};
use std::path::PathBuf;

/// Keyboard keys the viewer reacts to. Letters are upper-case ASCII.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    Home,
    End,
    PageUp,
    PageDown,
    Enter,
    Escape,
    Tab,
    Space,
    Plus,
    Equals,
    Minus,
    Comma,
    Num0,
    Num7,
    Num8,
    Num9,
    Letter(char),
}

/// Modifier state at the time of an input event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Mods {
    pub alt: bool,
    pub ctrl: bool,
    pub shift: bool,
    /// The Command key on macOS.
    pub mac_cmd: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WheelUnit {
    /// Classic mouse wheel notches.
    Line,
    /// Precise (trackpad) deltas in logical points.
    Pixel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerButtonKind {
    Back,
    Forward,
}

/// Toolkit-independent input event.
#[derive(Debug, Clone, PartialEq)]
pub enum InputEvent {
    Key {
        key: Key,
        mods: Mods,
    },
    /// Positive `delta_y` scrolls content down / away from the user (wheel rolled up).
    Wheel {
        delta_y: f64,
        unit: WheelUnit,
        mods: Mods,
    },
    DragPrimary {
        dx: f64,
        dy: f64,
    },
    DragMiddle {
        dx: f64,
        dy: f64,
    },
    Pinch {
        factor: f64,
    },
    PointerButton(PointerButtonKind),
    DoubleClickCanvas,
    FileDropped(PathBuf),
}

/// Which Extended-candidate capabilities are currently reachable (capability register).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtendedFeatures {
    pub image_list: bool,
    pub info_panel: bool,
    pub slideshow: bool,
}

/// State needed to resolve an input event.
#[derive(Debug, Clone, PartialEq)]
pub struct InteractionContext {
    pub is_macos: bool,
    pub dialogs_open: usize,
    pub text_focus: bool,
    pub fullscreen: bool,
    pub wheel_action: MouseWheelAction,
    pub image_exceeds_viewport: bool,
    /// Pointer position in logical points.
    pub cursor: (f64, f64),
    pub viewport_center: (f64, f64),
    pub extended: ExtendedFeatures,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogKind {
    Settings,
    About,
    ImageList,
}

/// Closed set of viewer actions.
#[derive(Debug, Clone, PartialEq)]
pub enum ViewerCommand {
    Next,
    Prev,
    First,
    Last,
    SelectIndex(usize),
    NextPage,
    PrevPage,
    ZoomBy {
        factor: f64,
        anchor: (f64, f64),
    },
    PanBy {
        dx: f64,
        dy: f64,
    },
    SetFit(DisplayMode),
    ResetView,
    RotateCw,
    RotateCcw,
    FlipH,
    FlipV,
    ToggleFullscreen,
    OpenDialog(DialogKind),
    CloseTopDialog,
    SetFilter(FilterMode),
    ToggleInfoPanel,
    ToggleSlideshow,
    /// Trackpad navigation delta (positive = towards the next image); the app accumulates it.
    ScrollNavigate {
        delta: f64,
    },
    RequestOpenFile,
    RequestOpenFolder,
    OpenPath(PathBuf),
    Quit,
}

/// Maps one input event to the commands it triggers (contract §1–§3).
pub fn map_input(event: &InputEvent, ctx: &InteractionContext) -> Vec<ViewerCommand> {
    match event {
        InputEvent::Key { key, mods } => map_key(*key, *mods, ctx),
        InputEvent::Wheel {
            delta_y,
            unit,
            mods,
        } => map_wheel(*delta_y, *unit, *mods, ctx),
        InputEvent::DragPrimary { dx, dy } => {
            if ctx.image_exceeds_viewport {
                vec![ViewerCommand::PanBy { dx: *dx, dy: *dy }]
            } else {
                Vec::new()
            }
        }
        InputEvent::DragMiddle { dx, dy } => vec![ViewerCommand::PanBy { dx: *dx, dy: *dy }],
        InputEvent::Pinch { factor } => {
            if factor.is_finite() && *factor > 0.0 && (*factor - 1.0).abs() > f64::EPSILON {
                vec![ViewerCommand::ZoomBy {
                    factor: *factor,
                    anchor: ctx.cursor,
                }]
            } else {
                Vec::new()
            }
        }
        InputEvent::PointerButton(PointerButtonKind::Back) => vec![ViewerCommand::Prev],
        InputEvent::PointerButton(PointerButtonKind::Forward) => vec![ViewerCommand::Next],
        InputEvent::DoubleClickCanvas => vec![ViewerCommand::ToggleFullscreen],
        InputEvent::FileDropped(path) => vec![ViewerCommand::OpenPath(path.clone())],
    }
}

fn map_key(key: Key, mods: Mods, ctx: &InteractionContext) -> Vec<ViewerCommand> {
    // §1 precedence: Esc first, then dialogs and text focus swallow everything else.
    if key == Key::Escape {
        let cmd = if ctx.dialogs_open > 0 {
            ViewerCommand::CloseTopDialog
        } else if ctx.fullscreen {
            ViewerCommand::ToggleFullscreen
        } else {
            ViewerCommand::Quit
        };
        return vec![cmd];
    }
    if ctx.text_focus || ctx.dialogs_open > 0 {
        return Vec::new();
    }

    let primary = if ctx.is_macos {
        mods.mac_cmd
    } else {
        mods.ctrl
    };
    let cmd = match key {
        Key::ArrowRight | Key::ArrowDown => Some(ViewerCommand::Next),
        Key::ArrowLeft | Key::ArrowUp => Some(ViewerCommand::Prev),
        Key::Home => Some(ViewerCommand::First),
        Key::End => Some(ViewerCommand::Last),
        Key::PageDown => Some(ViewerCommand::NextPage),
        Key::PageUp => Some(ViewerCommand::PrevPage),
        Key::Plus | Key::Equals => Some(ViewerCommand::ZoomBy {
            factor: 1.2,
            anchor: ctx.viewport_center,
        }),
        Key::Minus => Some(ViewerCommand::ZoomBy {
            factor: 0.83,
            anchor: ctx.viewport_center,
        }),
        Key::Num0 if primary => Some(ViewerCommand::ResetView),
        Key::Num0 => Some(ViewerCommand::SetFit(DisplayMode::Original)),
        Key::Num7 => Some(ViewerCommand::SetFit(DisplayMode::FitWidth)),
        Key::Num8 => Some(ViewerCommand::SetFit(DisplayMode::FitHeight)),
        Key::Num9 => Some(ViewerCommand::SetFit(DisplayMode::Fill)),
        Key::Enter if mods.alt => Some(ViewerCommand::ToggleFullscreen),
        Key::Enter if ctx.extended.image_list => {
            Some(ViewerCommand::OpenDialog(DialogKind::ImageList))
        }
        Key::Tab if ctx.extended.info_panel => Some(ViewerCommand::ToggleInfoPanel),
        Key::Space if ctx.extended.slideshow => Some(ViewerCommand::ToggleSlideshow),
        Key::Comma if primary => Some(ViewerCommand::OpenDialog(DialogKind::Settings)),
        Key::Letter(c) => map_letter(c, mods, primary, ctx),
        _ => None,
    };
    cmd.into_iter().collect()
}

fn map_letter(
    c: char,
    mods: Mods,
    primary: bool,
    ctx: &InteractionContext,
) -> Option<ViewerCommand> {
    let c = c.to_ascii_uppercase();
    if ctx.is_macos && c == 'F' && mods.ctrl && mods.mac_cmd {
        return Some(ViewerCommand::ToggleFullscreen);
    }
    if primary {
        return match c {
            'O' if mods.shift => Some(ViewerCommand::RequestOpenFolder),
            'O' => Some(ViewerCommand::RequestOpenFile),
            'Q' if ctx.is_macos => Some(ViewerCommand::Quit),
            _ => None,
        };
    }
    if mods.ctrl || mods.mac_cmd || mods.alt {
        return None;
    }
    match c {
        'R' => Some(ViewerCommand::RotateCw),
        'L' => Some(ViewerCommand::RotateCcw),
        'H' => Some(ViewerCommand::FlipH),
        'V' => Some(ViewerCommand::FlipV),
        'U' => Some(ViewerCommand::SetFilter(FilterMode::None)),
        'I' => Some(ViewerCommand::SetFilter(FilterMode::Smooth)),
        _ => None,
    }
}

fn map_wheel(dy: f64, unit: WheelUnit, mods: Mods, ctx: &InteractionContext) -> Vec<ViewerCommand> {
    if !dy.is_finite() || dy == 0.0 {
        return Vec::new();
    }
    if mods.shift {
        let step = match unit {
            WheelUnit::Line => dy * 32.0,
            WheelUnit::Pixel => dy,
        };
        return vec![ViewerCommand::PanBy { dx: 0.0, dy: step }];
    }
    let invert = mods.ctrl || mods.mac_cmd;
    let navigate = matches!(
        (ctx.wheel_action, invert),
        (MouseWheelAction::Navigate, false) | (MouseWheelAction::Zoom, true)
    );
    let cmd = match (navigate, unit) {
        (true, WheelUnit::Line) if dy < 0.0 => ViewerCommand::Next,
        (true, WheelUnit::Line) => ViewerCommand::Prev,
        (true, WheelUnit::Pixel) => ViewerCommand::ScrollNavigate { delta: -dy },
        (false, WheelUnit::Line) => ViewerCommand::ZoomBy {
            factor: if dy > 0.0 { 1.1 } else { 0.9 },
            anchor: ctx.cursor,
        },
        (false, WheelUnit::Pixel) => ViewerCommand::ZoomBy {
            factor: (1.0 + dy * 0.002).clamp(0.8, 1.25),
            anchor: ctx.cursor,
        },
    };
    vec![cmd]
}

/// Result of applying one command to the model.
#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    Nothing,
    /// Zoom, pan, fit, rotation, flip, filter, panel, or dialog state changed: repaint only.
    ViewChanged,
    /// A different entry is current: load it.
    ImageChanged,
    /// Same entry, different page: load that page.
    PageChanged,
    FullscreenChanged(bool),
    /// The command needs the OS or the app shell (dialogs, quit, open path, scroll accumulation).
    External(ViewerCommand),
}

/// Viewer state the commands act on.
#[derive(Debug, Clone, PartialEq)]
pub struct ViewerModel {
    pub entries_len: usize,
    /// Index of the displayed entry; `None` exactly when there are no entries.
    pub current: Option<usize>,
    pub page: u32,
    pub page_count: u32,
    pub wrap_around: bool,
    pub viewport: ViewportState,
    pub fullscreen: bool,
    /// Open dialogs; the last element is the topmost.
    pub dialogs: Vec<DialogKind>,
    pub filter: FilterMode,
    pub info_panel: bool,
    pub slideshow_playing: bool,
}

impl ViewerModel {
    pub fn new(logical_width: f64, logical_height: f64, dpi_scale_factor: f64) -> Self {
        Self {
            entries_len: 0,
            current: None,
            page: 0,
            page_count: 1,
            wrap_around: true,
            viewport: ViewportState::new(logical_width, logical_height, dpi_scale_factor),
            fullscreen: false,
            dialogs: Vec::new(),
            filter: FilterMode::None,
            info_panel: false,
            slideshow_playing: false,
        }
    }

    /// Replaces the entry count, selecting `preferred` clamped into range (data-model §8).
    pub fn set_source_len(&mut self, len: usize, preferred: Option<usize>) {
        self.entries_len = len;
        self.current = if len == 0 {
            None
        } else {
            Some(preferred.unwrap_or(0).min(len - 1))
        };
    }

    pub fn apply(&mut self, cmd: ViewerCommand) -> Outcome {
        match cmd {
            ViewerCommand::Next => self.step(1),
            ViewerCommand::Prev => self.step(-1),
            ViewerCommand::First => self.select(0),
            ViewerCommand::Last => match self.entries_len {
                0 => Outcome::Nothing,
                n => self.select(n - 1),
            },
            ViewerCommand::SelectIndex(i) => self.select(i),
            ViewerCommand::NextPage => {
                if self.current.is_some() && self.page_count > 1 && self.page + 1 < self.page_count
                {
                    self.page += 1;
                    Outcome::PageChanged
                } else {
                    self.step(1)
                }
            }
            ViewerCommand::PrevPage => {
                if self.current.is_some() && self.page_count > 1 && self.page > 0 {
                    self.page -= 1;
                    Outcome::PageChanged
                } else {
                    self.step(-1)
                }
            }
            ViewerCommand::ZoomBy { factor, anchor } => {
                self.viewport.zoom_at_cursor(factor, anchor.0, anchor.1);
                Outcome::ViewChanged
            }
            ViewerCommand::PanBy { dx, dy } => {
                if dx.is_finite() && dy.is_finite() {
                    self.viewport.pan_x += dx;
                    self.viewport.pan_y += dy;
                }
                Outcome::ViewChanged
            }
            ViewerCommand::SetFit(mode) => {
                self.viewport.display_mode = mode;
                Outcome::ViewChanged
            }
            ViewerCommand::ResetView => {
                self.viewport.reset_transforms();
                Outcome::ViewChanged
            }
            ViewerCommand::RotateCw => {
                self.viewport.rotate_cw();
                Outcome::ViewChanged
            }
            ViewerCommand::RotateCcw => {
                self.viewport.rotate_ccw();
                Outcome::ViewChanged
            }
            ViewerCommand::FlipH => {
                self.viewport.toggle_flip_h();
                Outcome::ViewChanged
            }
            ViewerCommand::FlipV => {
                self.viewport.toggle_flip_v();
                Outcome::ViewChanged
            }
            ViewerCommand::ToggleFullscreen => {
                self.fullscreen = !self.fullscreen;
                Outcome::FullscreenChanged(self.fullscreen)
            }
            ViewerCommand::OpenDialog(kind) => {
                self.dialogs.retain(|d| *d != kind);
                self.dialogs.push(kind);
                Outcome::ViewChanged
            }
            ViewerCommand::CloseTopDialog => match self.dialogs.pop() {
                Some(_) => Outcome::ViewChanged,
                None => Outcome::Nothing,
            },
            ViewerCommand::SetFilter(mode) => {
                self.filter = mode;
                Outcome::ViewChanged
            }
            ViewerCommand::ToggleInfoPanel => {
                self.info_panel = !self.info_panel;
                Outcome::ViewChanged
            }
            ViewerCommand::ToggleSlideshow => {
                self.slideshow_playing = !self.slideshow_playing;
                Outcome::ViewChanged
            }
            external @ (ViewerCommand::ScrollNavigate { .. }
            | ViewerCommand::RequestOpenFile
            | ViewerCommand::RequestOpenFolder
            | ViewerCommand::OpenPath(_)
            | ViewerCommand::Quit) => Outcome::External(external),
        }
    }

    fn step(&mut self, delta: isize) -> Outcome {
        let Some(cur) = self.current else {
            return Outcome::Nothing;
        };
        let len = self.entries_len;
        let next = if delta >= 0 {
            if cur + 1 < len {
                cur + 1
            } else if self.wrap_around {
                0
            } else {
                cur
            }
        } else if cur > 0 {
            cur - 1
        } else if self.wrap_around {
            len - 1
        } else {
            cur
        };
        self.select(next)
    }

    fn select(&mut self, index: usize) -> Outcome {
        if index >= self.entries_len || self.current == Some(index) {
            return Outcome::Nothing;
        }
        self.current = Some(index);
        self.page = 0;
        self.page_count = 1;
        self.viewport.reset_transforms();
        Outcome::ImageChanged
    }
}
