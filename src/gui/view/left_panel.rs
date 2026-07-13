//! Files, workspaces, and preferences left-panel composition.

#[path = "left_panel/files.rs"]
mod files;
#[path = "left_panel/preferences.rs"]
mod preferences;
#[path = "left_panel/tabs.rs"]
mod tabs;
#[path = "left_panel/view.rs"]
mod view;
#[path = "left_panel/workspaces.rs"]
mod workspaces;

pub(super) use files::gui_files_panel;
pub(super) use preferences::gui_preferences_panel;
pub(super) use tabs::gui_left_panel_tabs;
pub(super) use view::gui_left_panel_view;
pub(super) use workspaces::gui_workspaces_panel;
