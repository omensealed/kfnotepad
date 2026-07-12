//! Search ranges, line windows, and display-width calculations.

use super::*;

mod display_width;
mod line_window;
mod search_ranges;

pub(super) use display_width::cursor_cell_character;
pub(crate) use display_width::{
    char_column_for_display_column, character_display_width, line_display_width_until,
    line_segment_display_width, text_display_width,
};
pub(crate) use line_window::{print_line_window_with_search, LineWindowSearchView};
pub(crate) use search_ranges::search_match_ranges;
