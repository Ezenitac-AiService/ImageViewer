//! UI view overlays and independent OS child-window views.

pub mod about_view;
pub mod image_list_view;
pub mod main_view;
pub mod settings_view;
pub mod update_notice;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChildWindowKind {
    ImageList,
    Settings,
    About,
}
