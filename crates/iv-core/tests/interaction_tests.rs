//! Input → command mapping (`specs/004-launch-reliability-refactor/contracts/interaction-contract.md`).
//! One test per contract row, plus the precedence rules of §1.

use iv_core::interaction::{
    DialogKind, ExtendedFeatures, InputEvent, InteractionContext, Key, Mods, PointerButtonKind,
    ViewerCommand, WheelUnit, map_input,
};
use iv_core::settings::{DisplayMode, FilterMode, MouseWheelAction};
use std::path::PathBuf;

const CURSOR: (f64, f64) = (120.0, 80.0);
const CENTER: (f64, f64) = (400.0, 300.0);

fn ctx(is_macos: bool) -> InteractionContext {
    InteractionContext {
        is_macos,
        dialogs_open: 0,
        text_focus: false,
        fullscreen: false,
        wheel_action: MouseWheelAction::Navigate,
        image_exceeds_viewport: false,
        cursor: CURSOR,
        viewport_center: CENTER,
        extended: ExtendedFeatures {
            image_list: true,
            info_panel: true,
            slideshow: true,
        },
    }
}

fn none() -> Mods {
    Mods::default()
}

fn primary(is_macos: bool) -> Mods {
    if is_macos {
        Mods {
            mac_cmd: true,
            ..Mods::default()
        }
    } else {
        Mods {
            ctrl: true,
            ..Mods::default()
        }
    }
}

fn key(k: Key, mods: Mods, c: &InteractionContext) -> Vec<ViewerCommand> {
    map_input(&InputEvent::Key { key: k, mods }, c)
}

fn both_platforms(check: impl Fn(&InteractionContext, bool)) {
    for is_macos in [false, true] {
        check(&ctx(is_macos), is_macos);
    }
}

// ---------------------------------------------------------------- §2 keyboard

#[test]
fn arrow_right_and_down_go_next() {
    both_platforms(|c, _| {
        assert_eq!(key(Key::ArrowRight, none(), c), vec![ViewerCommand::Next]);
        assert_eq!(key(Key::ArrowDown, none(), c), vec![ViewerCommand::Next]);
    });
}

#[test]
fn arrow_left_and_up_go_prev() {
    both_platforms(|c, _| {
        assert_eq!(key(Key::ArrowLeft, none(), c), vec![ViewerCommand::Prev]);
        assert_eq!(key(Key::ArrowUp, none(), c), vec![ViewerCommand::Prev]);
    });
}

#[test]
fn home_and_end_go_first_and_last() {
    both_platforms(|c, _| {
        assert_eq!(key(Key::Home, none(), c), vec![ViewerCommand::First]);
        assert_eq!(key(Key::End, none(), c), vec![ViewerCommand::Last]);
    });
}

#[test]
fn page_down_and_up_step_pages() {
    both_platforms(|c, _| {
        assert_eq!(key(Key::PageDown, none(), c), vec![ViewerCommand::NextPage]);
        assert_eq!(key(Key::PageUp, none(), c), vec![ViewerCommand::PrevPage]);
    });
}

#[test]
fn plus_equals_and_minus_zoom_about_the_centre() {
    both_platforms(|c, _| {
        let zin = vec![ViewerCommand::ZoomBy {
            factor: 1.2,
            anchor: CENTER,
        }];
        assert_eq!(key(Key::Plus, none(), c), zin);
        assert_eq!(key(Key::Equals, none(), c), zin);
        assert_eq!(
            key(Key::Minus, none(), c),
            vec![ViewerCommand::ZoomBy {
                factor: 0.83,
                anchor: CENTER
            }]
        );
    });
}

#[test]
fn zero_sets_original_and_primary_zero_resets_view() {
    both_platforms(|c, mac| {
        assert_eq!(
            key(Key::Num0, none(), c),
            vec![ViewerCommand::SetFit(DisplayMode::Original)]
        );
        assert_eq!(
            key(Key::Num0, primary(mac), c),
            vec![ViewerCommand::ResetView]
        );
    });
}

#[test]
fn seven_eight_nine_set_fit_modes() {
    both_platforms(|c, _| {
        assert_eq!(
            key(Key::Num7, none(), c),
            vec![ViewerCommand::SetFit(DisplayMode::FitWidth)]
        );
        assert_eq!(
            key(Key::Num8, none(), c),
            vec![ViewerCommand::SetFit(DisplayMode::FitHeight)]
        );
        assert_eq!(
            key(Key::Num9, none(), c),
            vec![ViewerCommand::SetFit(DisplayMode::Fill)]
        );
    });
}

#[test]
fn r_l_h_v_rotate_and_flip() {
    both_platforms(|c, _| {
        assert_eq!(
            key(Key::Letter('R'), none(), c),
            vec![ViewerCommand::RotateCw]
        );
        assert_eq!(
            key(Key::Letter('L'), none(), c),
            vec![ViewerCommand::RotateCcw]
        );
        assert_eq!(key(Key::Letter('H'), none(), c), vec![ViewerCommand::FlipH]);
        assert_eq!(key(Key::Letter('V'), none(), c), vec![ViewerCommand::FlipV]);
    });
}

#[test]
fn letters_with_primary_modifier_do_not_rotate_or_flip() {
    both_platforms(|c, mac| {
        assert!(key(Key::Letter('R'), primary(mac), c).is_empty());
        assert!(key(Key::Letter('H'), primary(mac), c).is_empty());
    });
}

#[test]
fn u_and_i_set_filters_and_s_is_unmapped() {
    both_platforms(|c, _| {
        assert_eq!(
            key(Key::Letter('U'), none(), c),
            vec![ViewerCommand::SetFilter(FilterMode::None)]
        );
        assert_eq!(
            key(Key::Letter('I'), none(), c),
            vec![ViewerCommand::SetFilter(FilterMode::Smooth)]
        );
        assert!(key(Key::Letter('S'), none(), c).is_empty());
    });
}

#[test]
fn alt_enter_toggles_fullscreen_on_every_platform() {
    both_platforms(|c, _| {
        let alt = Mods {
            alt: true,
            ..Mods::default()
        };
        assert_eq!(
            key(Key::Enter, alt, c),
            vec![ViewerCommand::ToggleFullscreen]
        );
    });
}

#[test]
fn ctrl_cmd_f_toggles_fullscreen_on_macos_only() {
    let m = Mods {
        ctrl: true,
        mac_cmd: true,
        ..Mods::default()
    };
    assert_eq!(
        key(Key::Letter('F'), m, &ctx(true)),
        vec![ViewerCommand::ToggleFullscreen]
    );
    assert!(key(Key::Letter('F'), m, &ctx(false)).is_empty());
}

#[test]
fn enter_tab_space_open_extended_features_only_when_enabled() {
    both_platforms(|c, _| {
        assert_eq!(
            key(Key::Enter, none(), c),
            vec![ViewerCommand::OpenDialog(DialogKind::ImageList)]
        );
        assert_eq!(
            key(Key::Tab, none(), c),
            vec![ViewerCommand::ToggleInfoPanel]
        );
        assert_eq!(
            key(Key::Space, none(), c),
            vec![ViewerCommand::ToggleSlideshow]
        );
    });
    let mut c = ctx(false);
    c.extended = ExtendedFeatures {
        image_list: false,
        info_panel: false,
        slideshow: false,
    };
    assert!(key(Key::Enter, none(), &c).is_empty());
    assert!(key(Key::Tab, none(), &c).is_empty());
    assert!(key(Key::Space, none(), &c).is_empty());
}

#[test]
fn primary_o_opens_file_and_primary_shift_o_opens_folder() {
    both_platforms(|c, mac| {
        assert_eq!(
            key(Key::Letter('O'), primary(mac), c),
            vec![ViewerCommand::RequestOpenFile]
        );
        let mut m = primary(mac);
        m.shift = true;
        assert_eq!(
            key(Key::Letter('O'), m, c),
            vec![ViewerCommand::RequestOpenFolder]
        );
        assert!(key(Key::Letter('O'), none(), c).is_empty());
    });
}

#[test]
fn primary_comma_opens_settings() {
    both_platforms(|c, mac| {
        assert_eq!(
            key(Key::Comma, primary(mac), c),
            vec![ViewerCommand::OpenDialog(DialogKind::Settings)]
        );
    });
}

#[test]
fn esc_quits_when_no_dialog_and_not_fullscreen() {
    both_platforms(|c, _| {
        assert_eq!(key(Key::Escape, none(), c), vec![ViewerCommand::Quit]);
    });
}

#[test]
fn esc_leaves_fullscreen_before_quitting() {
    let mut c = ctx(false);
    c.fullscreen = true;
    assert_eq!(
        key(Key::Escape, none(), &c),
        vec![ViewerCommand::ToggleFullscreen]
    );
}

#[test]
fn cmd_q_quits_on_macos() {
    assert_eq!(
        key(Key::Letter('Q'), primary(true), &ctx(true)),
        vec![ViewerCommand::Quit]
    );
}

// ---------------------------------------------------------------- §3 pointer

fn wheel(dy: f64, unit: WheelUnit, mods: Mods, c: &InteractionContext) -> Vec<ViewerCommand> {
    map_input(
        &InputEvent::Wheel {
            delta_y: dy,
            unit,
            mods,
        },
        c,
    )
}

#[test]
fn wheel_navigates_when_setting_is_navigate() {
    let c = ctx(false);
    assert_eq!(
        wheel(-1.0, WheelUnit::Line, none(), &c),
        vec![ViewerCommand::Next]
    );
    assert_eq!(
        wheel(1.0, WheelUnit::Line, none(), &c),
        vec![ViewerCommand::Prev]
    );
    assert!(wheel(0.0, WheelUnit::Line, none(), &c).is_empty());
}

#[test]
fn trackpad_pixel_navigation_is_accumulated_by_the_app() {
    let c = ctx(false);
    assert_eq!(
        wheel(-12.5, WheelUnit::Pixel, none(), &c),
        vec![ViewerCommand::ScrollNavigate { delta: 12.5 }]
    );
}

#[test]
fn wheel_zooms_at_cursor_when_setting_is_zoom() {
    let mut c = ctx(false);
    c.wheel_action = MouseWheelAction::Zoom;
    assert_eq!(
        wheel(1.0, WheelUnit::Line, none(), &c),
        vec![ViewerCommand::ZoomBy {
            factor: 1.1,
            anchor: CURSOR
        }]
    );
    assert_eq!(
        wheel(-1.0, WheelUnit::Line, none(), &c),
        vec![ViewerCommand::ZoomBy {
            factor: 0.9,
            anchor: CURSOR
        }]
    );
    assert_eq!(
        wheel(50.0, WheelUnit::Pixel, none(), &c),
        vec![ViewerCommand::ZoomBy {
            factor: 1.1,
            anchor: CURSOR
        }]
    );
    assert_eq!(
        wheel(500.0, WheelUnit::Pixel, none(), &c),
        vec![ViewerCommand::ZoomBy {
            factor: 1.25,
            anchor: CURSOR
        }]
    );
}

#[test]
fn primary_wheel_inverts_the_setting() {
    both_platforms(|c, mac| {
        assert_eq!(
            wheel(1.0, WheelUnit::Line, primary(mac), c),
            vec![ViewerCommand::ZoomBy {
                factor: 1.1,
                anchor: CURSOR
            }]
        );
    });
}

#[test]
fn shift_wheel_pans_vertically() {
    let c = ctx(false);
    let shift = Mods {
        shift: true,
        ..Mods::default()
    };
    assert_eq!(
        wheel(1.0, WheelUnit::Line, shift, &c),
        vec![ViewerCommand::PanBy { dx: 0.0, dy: 32.0 }]
    );
    assert_eq!(
        wheel(7.0, WheelUnit::Pixel, shift, &c),
        vec![ViewerCommand::PanBy { dx: 0.0, dy: 7.0 }]
    );
}

#[test]
fn left_drag_pans_only_when_image_exceeds_viewport() {
    let mut c = ctx(false);
    let drag = InputEvent::DragPrimary { dx: 5.0, dy: -3.0 };
    assert!(map_input(&drag, &c).is_empty());
    c.image_exceeds_viewport = true;
    assert_eq!(
        map_input(&drag, &c),
        vec![ViewerCommand::PanBy { dx: 5.0, dy: -3.0 }]
    );
}

#[test]
fn middle_drag_always_pans() {
    let c = ctx(false);
    assert_eq!(
        map_input(&InputEvent::DragMiddle { dx: 2.0, dy: 4.0 }, &c),
        vec![ViewerCommand::PanBy { dx: 2.0, dy: 4.0 }]
    );
}

#[test]
fn pinch_zooms_at_cursor() {
    let c = ctx(true);
    assert_eq!(
        map_input(&InputEvent::Pinch { factor: 1.05 }, &c),
        vec![ViewerCommand::ZoomBy {
            factor: 1.05,
            anchor: CURSOR
        }]
    );
    assert!(map_input(&InputEvent::Pinch { factor: 1.0 }, &c).is_empty());
}

#[test]
fn mouse_back_and_forward_buttons_navigate() {
    let c = ctx(false);
    assert_eq!(
        map_input(&InputEvent::PointerButton(PointerButtonKind::Back), &c),
        vec![ViewerCommand::Prev]
    );
    assert_eq!(
        map_input(&InputEvent::PointerButton(PointerButtonKind::Forward), &c),
        vec![ViewerCommand::Next]
    );
}

#[test]
fn double_click_on_canvas_toggles_fullscreen() {
    let c = ctx(false);
    assert_eq!(
        map_input(&InputEvent::DoubleClickCanvas, &c),
        vec![ViewerCommand::ToggleFullscreen]
    );
}

#[test]
fn dropped_file_opens_that_path() {
    let c = ctx(false);
    let p = PathBuf::from("/tmp/a.png");
    assert_eq!(
        map_input(&InputEvent::FileDropped(p.clone()), &c),
        vec![ViewerCommand::OpenPath(p)]
    );
}

// ---------------------------------------------------------------- §1 precedence

#[test]
fn esc_closes_exactly_the_top_dialog_when_any_is_open() {
    for open in [1, 2] {
        let mut c = ctx(false);
        c.dialogs_open = open;
        c.fullscreen = true;
        assert_eq!(
            key(Key::Escape, none(), &c),
            vec![ViewerCommand::CloseTopDialog]
        );
    }
}

#[test]
fn keyboard_commands_do_not_reach_the_viewer_while_a_dialog_is_open() {
    let mut c = ctx(false);
    c.dialogs_open = 1;
    for k in [
        Key::ArrowRight,
        Key::ArrowLeft,
        Key::Home,
        Key::Num0,
        Key::Letter('R'),
    ] {
        assert!(key(k, none(), &c).is_empty(), "{k:?}");
    }
    assert!(key(Key::Letter('O'), primary(false), &c).is_empty());
}

#[test]
fn text_focus_swallows_every_key_except_esc() {
    let mut c = ctx(false);
    c.text_focus = true;
    assert!(key(Key::ArrowRight, none(), &c).is_empty());
    assert!(key(Key::Letter('R'), none(), &c).is_empty());
    c.dialogs_open = 1;
    assert_eq!(
        key(Key::Escape, none(), &c),
        vec![ViewerCommand::CloseTopDialog]
    );
}
