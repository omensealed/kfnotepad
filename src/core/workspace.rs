//! Workspace and browser state for TUI/GUI flows.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fmt, fs, io};

use super::{EditorTabState, OpenError, TextDocument};

#[path = "workspace/editor_types.rs"]
mod editor_types;
#[path = "workspace/editor_workspace.rs"]
mod editor_workspace;
#[path = "workspace/file_sidebar.rs"]
mod file_sidebar;
#[path = "workspace/gui_file_browser.rs"]
mod gui_file_browser;
#[path = "workspace/gui_types.rs"]
mod gui_types;
#[path = "workspace/gui_workspace.rs"]
mod gui_workspace;
#[path = "workspace/path_helpers.rs"]
mod path_helpers;

pub use editor_types::*;
pub use file_sidebar::*;
pub use gui_file_browser::*;
pub use gui_types::*;
pub use gui_workspace::*;
pub(crate) use path_helpers::{document_display_name, temporary_config_path};
