//! Persisted editor, search, reader, and GUI preference values and limits.

use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditorSettings {
    pub show_line_numbers: bool,
    pub theme_id: EditorThemeId,
    pub syntax_theme_id: EditorThemeId,
    pub wrap_lines: bool,
    pub search_case_sensitive: bool,
    pub gui_restore_last_workspace: bool,
    pub gui_reader_mode_enabled: bool,
    pub gui_reader_lines_per_minute: u16,
    pub gui_font_family: GuiFontFamily,
    pub gui_font_size: u16,
    pub gui_ui_font_size: u16,
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            show_line_numbers: true,
            theme_id: EditorThemeId::Nocturne,
            syntax_theme_id: EditorThemeId::Nocturne,
            wrap_lines: false,
            search_case_sensitive: false,
            gui_restore_last_workspace: false,
            gui_reader_mode_enabled: false,
            gui_reader_lines_per_minute: DEFAULT_GUI_READER_LINES_PER_MINUTE,
            gui_font_family: GuiFontFamily::Monospace,
            gui_font_size: DEFAULT_GUI_FONT_SIZE,
            gui_ui_font_size: DEFAULT_GUI_UI_FONT_SIZE,
        }
    }
}

pub const MIN_GUI_FONT_SIZE: u16 = 10;
pub const DEFAULT_GUI_FONT_SIZE: u16 = 16;
pub const DEFAULT_GUI_UI_FONT_SIZE: u16 = 14;
pub const MAX_GUI_FONT_SIZE: u16 = 32;
pub const MIN_GUI_READER_LINES_PER_MINUTE: u16 = 20;
pub const DEFAULT_GUI_READER_LINES_PER_MINUTE: u16 = 60;
pub const MAX_GUI_READER_LINES_PER_MINUTE: u16 = 240;
