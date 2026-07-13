//! GUI view construction and layout helpers.
//!
//! Rendering and widget composition for the Iced GUI are centralized here.

use super::*;
use crate::GuiDocumentTile;
use iced::widget;

include!("view/shell.rs");
#[path = "view/left_panel.rs"]
mod left_panel;
#[path = "view/panes.rs"]
mod panes;
#[path = "view/top_panels.rs"]
mod top_panels;

use left_panel::*;
use panes::*;
use top_panels::*;
