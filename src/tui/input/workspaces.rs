//! Workspace project prompts, persistence, restore, and manager actions.

use super::*;

include!("workspaces/prompts_and_manager.rs");
mod project_persistence;
mod project_restore;

pub(crate) use project_persistence::*;
pub(crate) use project_restore::*;
