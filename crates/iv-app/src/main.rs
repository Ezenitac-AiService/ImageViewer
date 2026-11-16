//! ImageViewer entry point.
//!
//! Startup order follows `specs/004-launch-reliability-refactor/contracts/launch-and-diagnostics-contract.md` §1:
//! logger → panic hook → `phase=start` → settings → CLI → renderer probe → window → first frame.
//! No step ends the process silently: every fatal path writes a record and shows a dialog.

#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;
mod canvas;
mod fonts;
mod loader;
mod probe;
mod views;

use crate::app::{AppInit, ViewerApp};
use crate::probe::RendererChoice;
use crate::views::status::StatusMessage;
use eframe::egui;
use iv_core::cli::{CliCommand, CliOptions};
use iv_core::diagnostics::{
    ExitCode as IvExit, FatalCategory, LaunchId, LaunchRecord, Phase, install_panic_hook,
};
use iv_core::i18n::{I18n, Language};
use iv_core::logger::AsyncFileLogger;
use iv_core::settings::{LoadOutcome, SettingsStore};
use iv_platform::fatal_dialog;
use iv_platform::paths_dialogs::OsPaths;
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

/// UI language for the panic dialog, updated once settings are known.
static PANIC_LANG: AtomicU8 = AtomicU8::new(0);

fn main() -> ExitCode {
    // 1. Diagnostics first, so every later failure leaves a trace.
    let record = LaunchRecord::new(LaunchId::random());
    // A logger that cannot start (unwritable or symlinked log directory) must not stop the
    // viewer: the launch continues without a record, and fatal paths still show the dialog.
    let logger = AsyncFileLogger::new(&OsPaths::logs_dir(), 256)
        .ok()
        .map(Arc::new);
    let write = {
        let logger = logger.clone();
        move |level: &str, line: &str| {
            if let Some(log) = &logger {
                log.write_sync(level, line);
            }
        }
    };
    {
        let write = write.clone();
        install_panic_hook(
            record.clone(),
            move |line| write("ERROR", line),
            || {
                fatal_dialog::show_fatal(panic_lang(), FatalCategory::Panic);
                std::process::exit(IvExit::Panic.code());
            },
        );
    }
    write(
        "INFO",
        &record.start(
            env!("CARGO_PKG_VERSION"),
            std::env::consts::OS,
            &os_version(),
            std::env::consts::ARCH,
        ),
    );

    // 2. Settings (never fatal).
    let settings_path = OsPaths::settings_file();
    let (settings, outcome) = SettingsStore::load_with_outcome(&settings_path);
    let lang = settings.language;
    PANIC_LANG.store(u8::from(lang == Language::Ko), Ordering::SeqCst);
    write(
        "INFO",
        &record.phase(Phase::Settings, &[("outcome", outcome_name(outcome))]),
    );

    // 3. Command line.
    let args: Vec<String> = std::env::args().collect();
    let cli = CliOptions::parse(&args, lang, cfg!(target_os = "windows"));
    let open_path = match cli.command {
        CliCommand::OpenPath(p) => Some(p),
        CliCommand::OpenEmpty | CliCommand::RegisterShell | CliCommand::UnregisterShell => None,
    };
    let initial_status = cli
        .error
        .clone()
        .map(StatusMessage::Error)
        .or_else(|| settings_notice(lang, outcome));

    // 4. Renderer probe.
    let choice = probe::probe();
    let renderer = match &choice {
        RendererChoice::Wgpu(adapter) => {
            write(
                "INFO",
                &record.phase(
                    Phase::Probe,
                    &[
                        ("renderer", "wgpu"),
                        ("adapter", &adapter.name),
                        ("type", adapter.kind.as_str()),
                        ("backend", &adapter.backend),
                    ],
                ),
            );
            eframe::Renderer::Wgpu
        }
        RendererChoice::Glow => {
            write("INFO", &record.phase(Phase::Probe, &[("renderer", "glow")]));
            eframe::Renderer::Glow
        }
    };

    // 5. Window.
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("ImageViewer")
        .with_app_id("com.ezenitac.imageviewer")
        .with_decorations(true)
        .with_inner_size([settings.window.width as f32, settings.window.height as f32])
        .with_min_inner_size([480.0, 320.0])
        .with_maximized(settings.window.is_maximized)
        .with_drag_and_drop(true);
    if !is_wayland()
        && let (Some(left), Some(top)) = (settings.window.left, settings.window.top)
    {
        viewport = viewport.with_position([left as f32, top as f32]);
    }
    let mut wgpu_options = eframe::egui_wgpu::WgpuConfiguration::default();
    if let eframe::egui_wgpu::WgpuSetup::CreateNew(setup) = &mut wgpu_options.wgpu_setup {
        setup.native_adapter_selector = Some(probe::adapter_selector());
    }
    let options = eframe::NativeOptions {
        viewport,
        renderer,
        wgpu_options,
        ..Default::default()
    };

    let init = AppInit {
        settings,
        settings_path,
        logger: logger.clone(),
        record: record.clone(),
        open_path,
        initial_status,
    };
    let on_window = {
        let write = write.clone();
        let record = record.clone();
        move || write("INFO", &record.phase(Phase::Window, &[]))
    };
    let result = eframe::run_native(
        "ImageViewer",
        options,
        Box::new(move |cc| {
            on_window();
            Ok(Box::new(ViewerApp::new(cc, init)))
        }),
    );

    match result {
        Ok(()) => ExitCode::from(exit_code(IvExit::Clean)),
        Err(err) => {
            let category = if matches!(choice, RendererChoice::Glow) {
                FatalCategory::NoRenderer
            } else {
                FatalCategory::WindowFailed
            };
            write(
                "ERROR",
                &record.fatal(category, &[("detail", &err.to_string())]),
            );
            fatal_dialog::show_fatal(lang, category);
            ExitCode::from(exit_code(IvExit::FatalStartup))
        }
    }
}

fn exit_code(code: IvExit) -> u8 {
    u8::try_from(code.code()).unwrap_or(1)
}

fn panic_lang() -> Language {
    if PANIC_LANG.load(Ordering::SeqCst) == 1 {
        Language::Ko
    } else {
        Language::En
    }
}

fn outcome_name(outcome: LoadOutcome) -> &'static str {
    match outcome {
        LoadOutcome::Loaded => "loaded",
        LoadOutcome::CreatedDefaults => "created_defaults",
        LoadOutcome::RecoveredFromDamage => "recovered_from_damage",
        LoadOutcome::Unreadable => "unreadable",
    }
}

fn settings_notice(lang: Language, outcome: LoadOutcome) -> Option<StatusMessage> {
    let key = match outcome {
        LoadOutcome::Loaded | LoadOutcome::CreatedDefaults => return None,
        LoadOutcome::RecoveredFromDamage => "settings.reset_notice",
        LoadOutcome::Unreadable => "settings.unreadable_notice",
    };
    Some(StatusMessage::Info(I18n::tr(lang, key).to_string()))
}

fn is_wayland() -> bool {
    cfg!(target_os = "linux") && std::env::var_os("WAYLAND_DISPLAY").is_some()
}

fn os_version() -> String {
    #[cfg(target_os = "macos")]
    {
        if let Ok(out) = std::process::Command::new("sw_vers")
            .arg("-productVersion")
            .output()
        {
            return String::from_utf8_lossy(&out.stdout).trim().to_string();
        }
    }
    "unknown".to_string()
}
