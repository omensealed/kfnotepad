//! Pane, theme, reader-mode, and pane-movement shortcuts.

use super::*;

#[path = "pane_theme_shortcuts/matcher.rs"]
mod matcher;
#[path = "pane_theme_shortcuts/move_pane.rs"]
mod move_pane;
#[path = "pane_theme_shortcuts/pane.rs"]
mod pane;
#[path = "pane_theme_shortcuts/theme_reader.rs"]
mod theme_reader;

use matcher::shortcut_character_matches;
use move_pane::move_pane_shortcut_message;
use pane::pane_size_shortcut_message;
use theme_reader::theme_reader_shortcut_message;

pub(super) fn pane_theme_reader_shortcut_message(event: &Event) -> Option<Message> {
    matcher::pane_theme_reader_shortcut_message(event)
}
