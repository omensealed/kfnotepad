use super::*;
use crate::tui::menu::*;
use crate::tui::render::*;
use crate::tui::sidebar::write_workspace_manager_overlay;

#[path = "render_chrome_layout/search_prompts.rs"]
mod search_prompts;
#[path = "render_chrome_layout/status_lines.rs"]
mod status_lines;
#[path = "render_chrome_layout/tabs_frames.rs"]
mod tabs_frames;
#[path = "render_chrome_layout/viewport_layout.rs"]
mod viewport_layout;
#[path = "render_chrome_layout/workspace_header.rs"]
mod workspace_header;
