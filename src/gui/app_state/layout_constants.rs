//! Stable GUI dimensions, spacing, timing, and rendering limits.

use super::Size;

pub(crate) const GUI_BROWSER_WIDTH_DEFAULT: f32 = 220.0;
pub(crate) const GUI_BROWSER_WIDTH_MIN: f32 = 160.0;
pub(crate) const GUI_BROWSER_WIDTH_MAX: f32 = 360.0;
pub(crate) const GUI_PANE_GRID_SPACING: f32 = 5.0;
pub(crate) const GUI_PANE_GRID_MIN_SIZE: f32 = 120.0;
pub(crate) const GUI_PANE_GRID_REFERENCE_SIZE: Size = Size {
    width: 1000.0,
    height: 800.0,
};
pub(crate) const GUI_MENU_DROPDOWN_WIDTH: f32 = 190.0;
pub(crate) const GUI_MENU_DROPDOWN_RADIUS: f32 = 6.0;
pub(crate) const GUI_MENU_ITEM_RADIUS: f32 = 4.0;
pub(crate) const GUI_MENU_ROOT_HORIZONTAL_PADDING: f32 = 3.0;
pub(crate) const GUI_MENU_ROOT_VERTICAL_PADDING: f32 = 1.0;
pub(crate) const GUI_MENU_ROOT_HEIGHT: f32 = 24.0;
pub(crate) const GUI_MENU_BAR_SPACING: u32 = 1;
pub(crate) const GUI_HEADER_ACTION_SPACING: u32 = 3;
pub(crate) const GUI_HEADER_GROUP_SPACING: u32 = 6;
pub(crate) const GUI_HEADER_SPLIT_SPACING: u32 = 3;
pub(crate) const GUI_MENU_ITEM_PADDING: [u16; 2] = [3, 5];
pub(crate) const GUI_CHROME_PADDING: [u16; 2] = [3, 5];
pub(crate) const GUI_ICON_LINE_HEIGHT: f32 = 1.0;
pub(crate) const GUI_FIND_HISTORY_LIMIT: usize = 10;
pub(crate) const GUI_READER_TICK_MS: u64 = 500;
pub(crate) const GUI_HELP_DOCUMENT_PATH: &str = "kfnotepad-help.md";
pub(crate) const GUI_ROOT_PADDING: u16 = 8;
pub(crate) const GUI_PANEL_PADDING_LEFT: f32 = 2.0;
pub(crate) const GUI_PANEL_PADDING_RIGHT: f32 = 4.0;
pub(crate) const GUI_PANEL_PADDING_VERTICAL: f32 = 6.0;
pub(crate) const GUI_PANEL_CONTROL_SPACING: u32 = 5;
pub(crate) const GUI_PANEL_SECTION_SPACING: u32 = 6;
pub(crate) const GUI_PANEL_PATH_MAX_CHARS: usize = 34;
pub(crate) const GUI_PANEL_TREE_TOP_PADDING: f32 = 4.0;
pub(crate) const GUI_FILE_TREE_INDENT: f32 = 14.0;
pub(crate) const GUI_FILE_TREE_ROW_SPACING: u32 = 2;
pub(crate) const GUI_FILE_TREE_MAX_DEPTH: usize = 8;
pub(crate) const GUI_TILE_BODY_PADDING: u16 = 2;
pub(crate) const GUI_TILE_TITLE_PADDING: u16 = 3;
pub(crate) const GUI_TILE_CONTROL_SPACING: u32 = 1;
pub(crate) const GUI_EDITOR_PADDING: u16 = 2;
pub(crate) const GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES: usize = 32;
pub(crate) const GUI_EDITOR_RENDER_LINE_BUDGET: usize = 512;
pub(crate) const GUI_LINE_NUMBER_GUTTER_HORIZONTAL_PADDING: f32 = 6.0;
pub(crate) const GUI_EDITOR_LINE_HEIGHT: f32 = 1.3;
pub(crate) const GUI_LINE_NUMBER_SEPARATOR_WIDTH: f32 = 1.0;
pub(crate) const GUI_TAB_WIDTH: usize = 4;
pub(crate) const GUI_EDITOR_SCROLLBAR_WIDTH: f32 = 6.0;
pub(crate) const GUI_EDITOR_SCROLLBAR_THUMB_MIN_HEIGHT: f32 = 18.0;
pub(crate) const GUI_REPLACEMENT_DRAG_TICK_MS: u64 = 40;
pub(crate) const GUI_TILE_RADIUS: f32 = 3.0;
pub(crate) const GUI_HEADER_SPLIT_WIDTH: f32 = 1180.0;
pub(crate) const GUI_SEARCH_SPLIT_WIDTH: f32 = 760.0;
