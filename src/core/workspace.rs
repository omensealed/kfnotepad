//! Workspace and browser state for TUI/GUI flows.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fmt, fs, io};

use super::{EditorTabState, OpenError, TextDocument};

include!("workspace/editor_types.rs");
include!("workspace/gui_types.rs");
include!("workspace/gui_workspace.rs");
include!("workspace/gui_file_browser.rs");
include!("workspace/path_helpers.rs");
include!("workspace/file_sidebar.rs");
include!("workspace/editor_workspace.rs");
