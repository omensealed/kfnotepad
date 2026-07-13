//! State assembly from settings, launch documents, panes, and browser projects.

#[path = "build_state/browser_projects.rs"]
mod browser_projects;
#[path = "build_state/documents.rs"]
mod documents;
#[path = "build_state/new_with_paths.rs"]
mod new_with_paths;
#[path = "build_state/panes.rs"]
mod panes;
#[path = "build_state/settings.rs"]
mod settings;
#[path = "build_state/types.rs"]
mod types;

use browser_projects::{build_gui_browser, load_gui_workspace_project_entries};
use documents::load_gui_launch_documents;
use panes::{build_gui_panes, build_workspace_and_pane_states};
use settings::load_gui_settings;
use types::{GuiBrowserBuild, GuiPaneBuild};
