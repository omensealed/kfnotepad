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

include!("settings/types.rs");
include!("settings/paths.rs");
include!("settings/parse_helpers.rs");
include!("settings/editor_config.rs");
include!("settings/gui_layout.rs");
include!("settings/workspace_projects.rs");
include!("settings/io_helpers.rs");
