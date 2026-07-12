//! Geometry-only layout parsing and structural validation.

use super::*;

#[path = "parse/layout.rs"]
mod layout;
#[path = "parse/node.rs"]
mod node;
#[path = "parse/ordinals.rs"]
mod ordinals;

pub use layout::parse_gui_layout;
use node::parse_gui_layout_node;
use ordinals::parse_layout_ordinals;
