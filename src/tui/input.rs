//! Keyboard, mouse, menu, sidebar, and editor command handling for the TUI.

use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};

use super::render::{
    char_column_for_display_column, compose_tab_label, menu_dropdown_column,
    menu_item_display_width, show_menu_bar, text_display_width, visible_text_columns,
    wrapped_line_chunks, RenderFrame, SearchHighlightView,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use crate::tui::menu::{
    CommandPaletteEntry, CommandPaletteState, MenuCommand, MenuGroup, MenuState,
    WorkspaceManagerEntry, WorkspaceManagerState,
};
use crate::tui::theme::EditorTheme;

use kfnotepad::{
    delete_gui_workspace_project, delete_next_word as shared_delete_next_word,
    delete_previous_word as shared_delete_previous_word,
    delete_to_line_end as shared_delete_to_line_end,
    go_to_document_end as shared_go_to_document_end,
    go_to_document_start as shared_go_to_document_start, go_to_line as shared_go_to_line,
    gui_workspace_project_path, list_gui_workspace_projects, load_editor_settings,
    move_document_cursor, move_path_to_trash, open_text_file, page_down as shared_page_down,
    page_up as shared_page_up, parse_gui_workspace_project, redo_document_edit,
    repeat_search_next_with_mode, repeat_search_previous_with_mode, save_editor_settings,
    save_gui_workspace_project, save_text_document, tui_help_document_text, undo_document_edit,
    BufferError, CloseActiveTabResult, Cursor, CursorMove, EditResult, EditorSettings, EditorTab,
    EditorTabDocument, EditorTabState, EditorWorkspace, FileSidebarEntry, FileSidebarEntryKind,
    FileSidebarState, GoToLineResult, GuiWorkspaceProject, GuiWorkspaceProjectDeleteResult,
    SearchMode, SearchRepeatResult, TabStripItem, TextDocument, UndoRedoResult,
    DEFAULT_GUI_READER_LINES_PER_MINUTE, MAX_GUI_READER_LINES_PER_MINUTE,
    MIN_GUI_READER_LINES_PER_MINUTE,
};

const TAB_WIDTH: usize = 4;
const EDITOR_BODY_PADDING: usize = 1;
const MOUSE_WHEEL_ROWS: usize = 3;
pub(crate) const TUI_READER_TICK_MS: u64 = 250;
pub(crate) const TUI_HELP_DOCUMENT_PATH: &str = "kfnotepad-help.md";
pub(crate) const TUI_CURRENT_WORKSPACE_NAME: &str = "current workspace";
const TUI_WORKSPACE_DIR_NAME: &str = "tui";

use crate::tui::menu::MENU_GROUPS;

/// Input/event handling extracted from the monolithic TUI application module.
#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum InputResult {
    Ignored,
    Handled,
    Quit,
}

#[derive(Clone, Copy)]
pub(crate) struct MouseContext {
    pub(crate) viewport_start: usize,
    pub(crate) horizontal_offset: usize,
    pub(crate) visible_rows: usize,
    pub(crate) gutter_width: usize,
    pub(crate) terminal_width: usize,
    pub(crate) sidebar_width: usize,
    pub(crate) body_top: u16,
}

include!("input/events.rs");
include!("input/sidebar.rs");
include!("input/workspaces.rs");
include!("input/editor_commands.rs");
mod runtime;

pub(crate) use runtime::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_at_mouse_snaps_to_grapheme_boundary() {
        let flag = "🇺🇸";
        let document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text(&format!("{flag}x\n")),
        };
        let runtime = EditorRuntime::default();
        let context = MouseContext {
            viewport_start: 0,
            horizontal_offset: 0,
            visible_rows: 10,
            gutter_width: 4,
            terminal_width: 80,
            sidebar_width: 0,
            body_top: 1,
        };

        assert_eq!(
            cursor_at_mouse(&document, 7, 1, &runtime, context),
            Some(Cursor { row: 0, column: 2 })
        );
    }

    #[test]
    fn paste_rejects_text_beyond_file_limit_without_mutation() {
        let limit = usize::try_from(kfnotepad::MAX_TEXT_FILE_BYTES).expect("text limit fits usize");
        let original = "x".repeat(limit);
        let mut document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: kfnotepad::TextBuffer::from_text(&original),
        };
        let mut cursor = Cursor {
            row: 0,
            column: limit,
        };
        let mut runtime = EditorRuntime::default();

        insert_paste(&mut document, &mut cursor, &mut runtime, "y");

        assert_eq!(document.buffer.to_text(), original);
        assert_eq!(cursor.column, limit);
        assert_eq!(runtime.status, format!("Paste exceeds {limit} byte limit"));
        assert!(!document.buffer.is_dirty());
    }
}
