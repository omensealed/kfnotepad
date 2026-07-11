//! GUI layout, startup restore, and workspace/project persistence helpers.
//!
//! This module centralizes geometry path resolution, workspace project restore,
//! and pane-layout serialization/deserialization support.

use super::*;

include!("layout/paths_and_external.rs");
include!("layout/workspace_restore.rs");
include!("layout/panes.rs");
include!("layout/serialization.rs");
include!("layout/app_chrome.rs");
