//! Rendering helpers for the terminal UI.

use std::env;
use std::io::{self, Write};
use std::path::PathBuf;

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::queue;
use crossterm::style::{
    Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
};
use crossterm::terminal::{size, Clear, ClearType};
use syntect::highlighting::Style as SyntectStyle;
use unicode_width::UnicodeWidthChar;

use crate::tui::menu::{MenuGroup, MenuItem, MenuState, MENU_GROUPS};
use crate::tui::theme::EditorTheme;
use kfnotepad::{
    case_insensitive_match_ranges, expand_range_to_grapheme_boundaries, Cursor, EditorSettings,
    EditorThemeId, SearchMode, SyntaxHighlightCacheState, SyntaxHighlightedLines,
    SyntaxHighlighter, TabStripItem, TextDocument,
};

const TAB_WIDTH: usize = 4;
const EDITOR_BODY_PADDING: usize = 1;

include!("render/entry.rs");
include!("render/chrome.rs");
include!("render/editor_lines.rs");
include!("render/status_text.rs");
mod syntax_colors;
mod viewport_wrapping;

pub(crate) use syntax_colors::*;
pub(crate) use viewport_wrapping::*;
include!("render/tests.rs");
