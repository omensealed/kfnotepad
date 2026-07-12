//! GUI workspace tabs, layouts, projects, panel modes, and tile value types.

use super::*;

#[path = "gui_types/layout.rs"]
mod layout;
#[path = "gui_types/left_panel.rs"]
mod left_panel;
#[path = "gui_types/projects.rs"]
mod projects;
#[path = "gui_types/tabs.rs"]
mod tabs;
#[path = "gui_types/tiles.rs"]
mod tiles;

pub use layout::*;
pub use left_panel::*;
pub use projects::*;
pub use tabs::*;
pub use tiles::*;
