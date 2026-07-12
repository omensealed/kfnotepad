//! Sidebar and overlay rendering helpers for the terminal UI.

#[path = "sidebar/colors.rs"]
mod colors;
#[path = "sidebar/command_palette_overlay.rs"]
mod command_palette_overlay;
#[path = "sidebar/file_sidebar.rs"]
mod file_sidebar;
#[path = "sidebar/workspace_overlay.rs"]
mod workspace_overlay;

pub(crate) use command_palette_overlay::write_command_palette_overlay;
pub(crate) use file_sidebar::render_file_sidebar;
pub(crate) use workspace_overlay::write_workspace_manager_overlay;
