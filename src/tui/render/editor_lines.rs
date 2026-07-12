//! Plain/wrapped editor-line rendering, highlighting, and line helpers.

use super::*;

mod helpers;
mod highlighting;
mod plain_line;
mod wrapped_chunk;
mod wrapped_lines;

use helpers::write_editor_body_padding;
#[cfg(test)]
pub(crate) use highlighting::grapheme_safe_highlight_segments;
pub(crate) use highlighting::highlight_lines_for_render;
use highlighting::{highlighter_lines_for_wrapped_view, write_highlighted_line_window};
pub(crate) use plain_line::write_editor_line;
use wrapped_chunk::{
    clear_remaining_editor_rows, write_wrapped_editor_chunk, WrappedEditorChunkView,
};
pub(crate) use wrapped_lines::write_wrapped_editor_lines;
