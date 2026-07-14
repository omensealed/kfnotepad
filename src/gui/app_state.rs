//! GUI state and action model shared by the Iced GUI state machine.

// Message/state variants are often constructed by Iced subscriptions, Tasks,
// or tests in specific targets, so rustc reports false positives elsewhere.
use std::collections::HashSet;
use std::path::PathBuf;

use iced::advanced::input_method;
use iced::widget::pane_grid;
#[cfg(test)]
use iced::widget::text_editor;
use iced::{window, Size};
use kfnotepad::{FileSidebarEntryKind, GuiLeftPanelMode, GuiTileId, TextDocument};
use nerd_font_symbols as nf;

use super::{
    GuiEditorDragEdge, GuiEditorReplacementInput, GuiEditorReplacementMousePoint,
    GuiEditorScrollbarModel, GuiExternalFileCheckResult,
};

#[path = "app_state/help_text.rs"]
mod help_text;
#[path = "app_state/icons.rs"]
mod icons;
#[path = "app_state/labels.rs"]
mod labels;
#[path = "app_state/layout_constants.rs"]
mod layout_constants;
#[path = "app_state/messages.rs"]
mod messages;
#[path = "app_state/types.rs"]
mod types;

pub(super) use help_text::*;
pub(super) use icons::*;
pub(super) use labels::*;
pub(super) use layout_constants::*;
pub(super) use messages::*;
pub(super) use types::*;
