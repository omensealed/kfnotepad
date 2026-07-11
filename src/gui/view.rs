//! GUI view construction and layout helpers.
//!
//! Rendering and widget composition for the Iced GUI are centralized here.

use super::*;
use crate::GuiDocumentTile;
use iced::widget;

include!("view/shell.rs");
include!("view/top_panels.rs");
include!("view/left_panel.rs");
include!("view/panes.rs");
