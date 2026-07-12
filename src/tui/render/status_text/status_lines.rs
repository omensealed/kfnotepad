//! Status/help rows and editor cursor-cell highlighting.

use super::*;

mod cursor_cell;
mod help;
mod status;

pub(crate) use cursor_cell::{cursor_row_is_visible, write_cursor_cell_highlight};
#[cfg(test)]
pub(crate) use help::compose_help_line;
pub(crate) use help::write_help_line;
pub(crate) use status::{write_status_line, StatusLineView};
