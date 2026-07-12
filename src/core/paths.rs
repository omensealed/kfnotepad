//! Filesystem path resolution helpers.
//!
//! This module contains shared platform-aware logic for editor/config/workspace/data
//! paths while keeping explicit base-directory helpers for tests.

#[path = "paths/current.rs"]
mod current;
#[path = "paths/helpers.rs"]
mod helpers;
#[path = "paths/platform.rs"]
mod platform;
#[path = "paths/resolve.rs"]
mod resolve;

pub use current::*;
pub use resolve::*;

#[cfg(test)]
#[path = "paths/tests.rs"]
mod tests;
