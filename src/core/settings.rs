//! Settings and config serialization for editor/theme/workspace persistence.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use super::{
    move_path_to_trash, resolve_editor_config_path, resolve_gui_layout_path,
    resolve_gui_workspace_projects_dir, temporary_config_path, GuiLayout, GuiLayoutAxis,
    GuiLayoutNode, GuiWorkspaceProject, GuiWorkspaceProjectDeleteResult, GuiWorkspaceProjectEntry,
};

#[path = "settings/editor_config.rs"]
mod editor_config;
#[path = "settings/gui_layout.rs"]
mod gui_layout;
#[path = "settings/io_helpers.rs"]
mod io_helpers;
#[path = "settings/parse_helpers.rs"]
mod parse_helpers;
#[path = "settings/paths.rs"]
mod paths;
#[path = "settings/types.rs"]
mod types;
#[path = "settings/workspace_projects.rs"]
mod workspace_projects;

pub use editor_config::*;
pub use gui_layout::*;
use io_helpers::{set_private_config_dir_permissions, write_config_temp_then_rename};
use parse_helpers::{parse_config_bool, parse_config_string};
pub use paths::*;
pub use types::*;
pub use workspace_projects::*;
