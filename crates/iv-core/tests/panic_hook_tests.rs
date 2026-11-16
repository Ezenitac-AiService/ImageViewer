//! Panic hook writes a fatal record before the process dies (specs/004 T036).
//! Isolated in its own test binary because the panic hook is process-global.

use iv_core::diagnostics::{LaunchId, LaunchRecord, install_panic_hook};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

#[test]
fn panic_writes_one_fatal_line_and_calls_on_fatal_once() {
    let lines: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let fatal_calls = Arc::new(AtomicUsize::new(0));
    let sink_lines = Arc::clone(&lines);
    let calls = Arc::clone(&fatal_calls);
    install_panic_hook(
        LaunchRecord::new(LaunchId::from_u32(0x1234_abcd)),
        move |line| sink_lines.lock().expect("lock").push(line.to_string()),
        move || {
            calls.fetch_add(1, Ordering::SeqCst);
        },
    );
    let result = std::panic::catch_unwind(|| panic!("boom for test"));
    let _ = std::panic::take_hook();
    assert!(result.is_err());
    let lines = lines.lock().expect("lock");
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(
        lines[0].starts_with("launch=1234abcd fatal category=panic at="),
        "{}",
        lines[0]
    );
    assert!(lines[0].contains("panic_hook_tests.rs:"), "{}", lines[0]);
    assert_eq!(fatal_calls.load(Ordering::SeqCst), 1);
}
