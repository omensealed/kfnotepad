//! Sidebar and overlay rendering helpers for terminal UI.

// Sidebar helpers are compiled through the library target but driven by the
// TUI binary/runtime tests, which leaves false dead-code positives per target.
use std::io::{self, Write};

use crossterm::cursor::MoveTo;
use crossterm::cursor::Show;
use crossterm::queue;
use crossterm::style::{
    Attribute, Color, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
};
use kfnotepad::FileSidebarState;

use super::input::command_palette_candidates;
use super::menu::{CommandPaletteState, WorkspaceManagerState};
use super::theme::EditorTheme;
use crate::tui::app::{fit_text_end, print_truncated, text_display_width, SIDEBAR_WIDTH};

include!("sidebar/file_sidebar.rs");
include!("sidebar/workspace_overlay.rs");
include!("sidebar/command_palette_overlay.rs");
include!("sidebar/colors.rs");
