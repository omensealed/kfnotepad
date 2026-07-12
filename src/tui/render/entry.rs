//! Editor rendering entrypoints, orchestration, color setup, and cursor placement.

use super::*;

mod api;
mod clear_body;
mod color;
mod cursor;
mod orchestrate;

pub(crate) use api::*;
pub(crate) use clear_body::*;
pub(crate) use color::*;
use cursor::*;
#[cfg(test)]
pub(crate) use orchestrate::render_editor_with_width_and_color;
use orchestrate::render_editor_with_width_color_and_cache;
