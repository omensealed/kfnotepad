//! GUI layout, startup restore, and workspace/project persistence helpers.
//!
//! This module centralizes geometry path resolution, workspace project restore,
//! and pane-layout serialization/deserialization support.

use super::*;

#[path = "layout/app_chrome.rs"]
mod app_chrome;
#[path = "layout/panes.rs"]
mod panes;
#[path = "layout/paths_and_external.rs"]
mod paths_and_external;
#[path = "layout/serialization.rs"]
mod serialization;
#[path = "layout/workspace_restore.rs"]
mod workspace_restore;

pub(super) use app_chrome::*;
pub(super) use panes::*;
pub(super) use paths_and_external::*;
pub(super) use serialization::*;
pub(super) use workspace_restore::*;
