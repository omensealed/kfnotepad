//! Filesystem path resolution helpers.
//!
//! This module contains shared platform-aware logic for editor/config/workspace/data
//! paths while keeping explicit base-directory helpers for tests.

use std::path::{Path, PathBuf};

use super::ManagedNotesError;

include!("paths/helpers.rs");
include!("paths/resolve.rs");
include!("paths/current.rs");
include!("paths/platform.rs");
include!("paths/tests.rs");
