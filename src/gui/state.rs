//! Iced GUI state machine, view/update wiring, and module includes.

// GUI state is binary-driven but compiled through the library target for
// feature checks and tests, which leaves false dead-code positives per target.
use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;
use std::{env, fs, io};

use iced::advanced::{
    input_method, layout as advanced_layout, overlay as advanced_overlay,
    renderer as advanced_renderer,
    widget::{Operation as AdvancedOperation, Tree, Widget},
    Clipboard as AdvancedClipboard, Layout as AdvancedLayout, Shell as AdvancedShell,
};
use iced::keyboard::key::{Key, Named};
use iced::widget::{
    button, checkbox, column, container, mouse_area, pane_grid, responsive, rich_text, row,
    scrollable, slider, span, text, text::Wrapping, text_editor, text_input,
};
use iced::{
    clipboard, event, highlighter, keyboard, mouse, window, Alignment, Background, Border, Color,
    Element, Event, Font, Length, Pixels, Rectangle, Shadow, Size, Subscription, Task, Theme,
    Vector,
};
use iced_aw::{menu, Menu, MenuBar};
use kfnotepad::{
    delete_gui_workspace_project, delete_managed_note, delete_next_word, delete_previous_word,
    delete_to_line_end, expand_range_to_grapheme_boundaries, find_case_insensitive_range,
    find_last_case_insensitive_range, go_to_document_end, go_to_document_start, go_to_line,
    gui_workspace_project_path, list_file_sidebar_entries, list_gui_workspace_projects,
    list_managed_notes, load_editor_settings, move_path_to_trash, open_or_create_managed_note,
    open_text_file, parse_gui_layout, parse_gui_workspace_project, redo_document_edit,
    repeat_search_next, repeat_search_previous, save_editor_settings, save_gui_layout,
    save_gui_workspace_project, save_text_buffer, save_text_snapshot, snapshot_text_file,
    snapshot_text_file_metadata, undo_document_edit, BufferError, Cursor as DocumentCursor,
    EditorSettings, EditorTabState, EditorThemeId, FileMetadataSnapshot, FileSidebarEntry,
    FileSidebarEntryKind, FileSnapshot, GoToLineResult, GuiCloseTileResult, GuiFileBrowser,
    GuiFontFamily, GuiLayout, GuiLayoutAxis, GuiLayoutNode, GuiLeftPanelMode, GuiLeftPanelState,
    GuiTileId, GuiTileSaveStatus, GuiWorkspace, GuiWorkspaceProject,
    GuiWorkspaceProjectDeleteResult, GuiWorkspaceProjectEntry, ManagedNoteDeleteResult,
    ManagedNoteEntry, SearchRepeatResult, SyntaxHighlightCacheState, SyntaxHighlighter, TextBuffer,
    TextDocument, UndoRedoResult, MAX_GUI_FONT_SIZE, MAX_GUI_READER_LINES_PER_MINUTE,
    MIN_GUI_FONT_SIZE, MIN_GUI_READER_LINES_PER_MINUTE, VERSION,
};

#[cfg(test)]
use kfnotepad::save_text_document;
use nerd_font_symbols as nf;
use syntect::highlighting::Style as SyntectStyle;
use unicode_width::UnicodeWidthChar;

#[path = "app_state.rs"]
mod app_state;
mod dialogs;
mod editor_adapter;
mod external_watcher;
mod file_browser;
mod layout;
mod preferences;
mod theme;
mod workspace_tiles;
use app_state::*;
use editor_adapter::*;
use external_watcher::*;
use layout::*;
use theme::*;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

#[path = "update.rs"]
mod update;
#[path = "view.rs"]
mod view;
use update::{subscription, update};
use view::view;

#[path = "state/constructors.rs"]
mod constructors;
#[path = "state/launch.rs"]
mod launch;
#[path = "state/types.rs"]
mod types;

pub use launch::run;
use launch::GuiLaunch;
use types::*;
