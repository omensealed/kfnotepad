//! Semantic syntax roles, source-color classification, and theme palettes.

use super::*;

#[path = "roles/classification.rs"]
mod role_classification;
#[path = "roles/palettes.rs"]
mod role_palettes;
#[path = "roles/types.rs"]
mod role_types;

pub(super) use role_classification::*;
pub(super) use role_palettes::*;
pub(super) use role_types::*;
