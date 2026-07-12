//! Geometry-only GUI layout parsing, serialization, and persistence.

use super::*;

#[path = "gui_layout/parse.rs"]
mod parse;
#[path = "gui_layout/save.rs"]
mod save;
#[path = "gui_layout/serialize.rs"]
mod serialize;

pub use parse::parse_gui_layout;
pub use save::save_gui_layout;
pub use serialize::serialize_gui_layout;
