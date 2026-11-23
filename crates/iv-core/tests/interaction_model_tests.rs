//! `ViewerModel::apply` for every command (`specs/004-launch-reliability-refactor/data-model.md` §7).

use iv_core::interaction::{DialogKind, Outcome, ViewerCommand, ViewerModel};
use iv_core::settings::{DisplayMode, FilterMode};
use std::path::PathBuf;

fn model(len: usize, wrap: bool) -> ViewerModel {
    let mut m = ViewerModel::new(800.0, 600.0, 1.0);
    m.set_source_len(len, if len == 0 { None } else { Some(0) });
    m.wrap_around = wrap;
    m
}

#[test]
fn next_and_prev_move_within_bounds_and_reset_view() {
    let mut m = model(3, false);
    m.viewport.rotate_cw();
    assert_eq!(m.apply(ViewerCommand::Next), Outcome::ImageChanged);
    assert_eq!(m.current, Some(1));
    assert_eq!(m.viewport.rotation_quarter_turns, 0);
    assert_eq!(m.apply(ViewerCommand::Prev), Outcome::ImageChanged);
    assert_eq!(m.current, Some(0));
}

#[test]
fn next_on_last_entry_respects_wrap_around() {
    let mut m = model(3, false);
    m.current = Some(2);
    assert_eq!(m.apply(ViewerCommand::Next), Outcome::Nothing);
    assert_eq!(m.current, Some(2));
    let mut m = model(3, true);
    m.current = Some(2);
    assert_eq!(m.apply(ViewerCommand::Next), Outcome::ImageChanged);
    assert_eq!(m.current, Some(0));
}

#[test]
fn prev_on_first_entry_respects_wrap_around() {
    let mut m = model(3, true);
    assert_eq!(m.apply(ViewerCommand::Prev), Outcome::ImageChanged);
    assert_eq!(m.current, Some(2));
    let mut m = model(3, false);
    assert_eq!(m.apply(ViewerCommand::Prev), Outcome::Nothing);
    assert_eq!(m.current, Some(0));
}

#[test]
fn navigation_on_an_empty_source_is_a_no_op() {
    let mut m = model(0, true);
    for cmd in [
        ViewerCommand::Next,
        ViewerCommand::Prev,
        ViewerCommand::First,
        ViewerCommand::Last,
        ViewerCommand::NextPage,
        ViewerCommand::PrevPage,
    ] {
        assert_eq!(m.apply(cmd), Outcome::Nothing);
        assert_eq!(m.current, None);
    }
}

#[test]
fn first_and_last_jump_to_ends() {
    let mut m = model(5, false);
    assert_eq!(m.apply(ViewerCommand::Last), Outcome::ImageChanged);
    assert_eq!(m.current, Some(4));
    assert_eq!(m.apply(ViewerCommand::Last), Outcome::Nothing);
    assert_eq!(m.apply(ViewerCommand::First), Outcome::ImageChanged);
    assert_eq!(m.current, Some(0));
}

#[test]
fn next_page_steps_pages_then_falls_through_to_next() {
    let mut m = model(2, false);
    m.page_count = 3;
    assert_eq!(m.apply(ViewerCommand::NextPage), Outcome::PageChanged);
    assert_eq!(m.page, 1);
    m.page = 2;
    assert_eq!(m.apply(ViewerCommand::NextPage), Outcome::ImageChanged);
    assert_eq!((m.current, m.page, m.page_count), (Some(1), 0, 1));
    // Single-page image: PageDown behaves like Next.
    let mut m = model(2, false);
    assert_eq!(m.apply(ViewerCommand::NextPage), Outcome::ImageChanged);
    assert_eq!(m.current, Some(1));
}

#[test]
fn prev_page_steps_back_then_falls_through_to_prev() {
    let mut m = model(2, false);
    m.current = Some(1);
    m.page_count = 2;
    m.page = 1;
    assert_eq!(m.apply(ViewerCommand::PrevPage), Outcome::PageChanged);
    assert_eq!(m.page, 0);
    assert_eq!(m.apply(ViewerCommand::PrevPage), Outcome::ImageChanged);
    assert_eq!(m.current, Some(0));
}

#[test]
fn zoom_pan_fit_and_reset_change_the_view() {
    let mut m = model(1, false);
    assert_eq!(
        m.apply(ViewerCommand::ZoomBy {
            factor: 2.0,
            anchor: (400.0, 300.0)
        }),
        Outcome::ViewChanged
    );
    assert!((m.viewport.zoom_scale - 2.0).abs() < 1e-9);
    assert_eq!(
        m.apply(ViewerCommand::PanBy { dx: 10.0, dy: -4.0 }),
        Outcome::ViewChanged
    );
    assert_eq!((m.viewport.pan_x, m.viewport.pan_y), (10.0, -4.0));
    assert_eq!(
        m.apply(ViewerCommand::SetFit(DisplayMode::FitWidth)),
        Outcome::ViewChanged
    );
    assert_eq!(m.viewport.display_mode, DisplayMode::FitWidth);
    assert_eq!(m.apply(ViewerCommand::ResetView), Outcome::ViewChanged);
    assert!((m.viewport.zoom_scale - 1.0).abs() < 1e-9);
    assert_eq!((m.viewport.pan_x, m.viewport.pan_y), (0.0, 0.0));
}

#[test]
fn rotate_and_flip_commands_update_transforms() {
    let mut m = model(1, false);
    m.apply(ViewerCommand::RotateCw);
    assert_eq!(m.viewport.rotation_quarter_turns, 1);
    m.apply(ViewerCommand::RotateCcw);
    m.apply(ViewerCommand::RotateCcw);
    assert_eq!(m.viewport.rotation_quarter_turns, 3);
    m.apply(ViewerCommand::FlipH);
    m.apply(ViewerCommand::FlipV);
    assert!(m.viewport.flip_h && m.viewport.flip_v);
}

#[test]
fn fullscreen_toggle_reports_new_state() {
    let mut m = model(1, false);
    assert_eq!(
        m.apply(ViewerCommand::ToggleFullscreen),
        Outcome::FullscreenChanged(true)
    );
    assert_eq!(
        m.apply(ViewerCommand::ToggleFullscreen),
        Outcome::FullscreenChanged(false)
    );
}

#[test]
fn dialog_stack_opens_raises_and_closes_one_at_a_time() {
    let mut m = model(1, false);
    m.apply(ViewerCommand::OpenDialog(DialogKind::Settings));
    m.apply(ViewerCommand::OpenDialog(DialogKind::About));
    assert_eq!(m.dialogs, vec![DialogKind::Settings, DialogKind::About]);
    // Re-opening an open dialog raises it instead of duplicating it.
    m.apply(ViewerCommand::OpenDialog(DialogKind::Settings));
    assert_eq!(m.dialogs, vec![DialogKind::About, DialogKind::Settings]);
    m.apply(ViewerCommand::CloseTopDialog);
    assert_eq!(m.dialogs, vec![DialogKind::About]);
    m.apply(ViewerCommand::CloseTopDialog);
    assert!(m.dialogs.is_empty());
    assert_eq!(m.apply(ViewerCommand::CloseTopDialog), Outcome::Nothing);
}

#[test]
fn filter_info_panel_and_slideshow_toggles() {
    let mut m = model(1, false);
    m.apply(ViewerCommand::SetFilter(FilterMode::Smooth));
    assert_eq!(m.filter, FilterMode::Smooth);
    m.apply(ViewerCommand::ToggleInfoPanel);
    assert!(m.info_panel);
    m.apply(ViewerCommand::ToggleSlideshow);
    assert!(m.slideshow_playing);
}

#[test]
fn commands_handled_outside_the_model_are_returned_as_external() {
    let mut m = model(1, false);
    for cmd in [
        ViewerCommand::RequestOpenFile,
        ViewerCommand::RequestOpenFolder,
        ViewerCommand::Quit,
        ViewerCommand::OpenPath(PathBuf::from("x.png")),
        ViewerCommand::ScrollNavigate { delta: 3.0 },
    ] {
        assert_eq!(m.apply(cmd.clone()), Outcome::External(cmd));
    }
}

#[test]
fn select_index_jumps_and_rejects_out_of_range() {
    let mut m = model(4, false);
    assert_eq!(
        m.apply(ViewerCommand::SelectIndex(3)),
        Outcome::ImageChanged
    );
    assert_eq!(m.current, Some(3));
    assert_eq!(m.apply(ViewerCommand::SelectIndex(9)), Outcome::Nothing);
    assert_eq!(m.current, Some(3));
}

#[test]
fn set_source_len_clamps_current_into_range() {
    let mut m = model(5, false);
    m.current = Some(4);
    m.set_source_len(2, Some(4));
    assert_eq!(m.current, Some(1));
    m.set_source_len(0, Some(1));
    assert_eq!(m.current, None);
}
