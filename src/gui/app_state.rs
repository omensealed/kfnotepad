//! GUI state and action model shared by the Iced GUI state machine.

// Message/state variants are often constructed by Iced subscriptions, Tasks,
// or tests in specific targets, so rustc reports false positives elsewhere.
#![allow(dead_code)]

use std::path::PathBuf;

use iced::advanced::input_method;
use iced::widget::{pane_grid, text_editor};
use iced::{window, Size};
use iced_swdir_tree::DirectoryTreeEvent;
use kfnotepad::{GuiLeftPanelMode, GuiTileId, TextDocument};
use nerd_font_symbols as nf;

use super::{
    GuiEditorDragEdge, GuiEditorReplacementInput, GuiEditorReplacementMousePoint,
    GuiEditorScrollbarModel, GuiExternalFileCheckResult,
};

include!("app_state/types.rs");
include!("app_state/labels.rs");
include!("app_state/icons.rs");
include!("app_state/layout_constants.rs");
include!("app_state/help_text.rs");
include!("app_state/messages.rs");
