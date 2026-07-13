use std::collections::VecDeque;
use std::time::{Instant, SystemTime};

use super::{
    expand_range_to_grapheme_boundaries, find_case_insensitive_range,
    find_last_case_insensitive_range, Cursor, CursorMove, SearchMode, MAX_TEXT_FILE_BYTES,
    MAX_UNDO_BYTES, MAX_UNDO_HISTORY, TYPING_UNDO_COALESCE_WINDOW,
};

#[path = "text_buffer/constructors.rs"]
mod constructors;
#[path = "text_buffer/cursor.rs"]
mod cursor;
#[path = "text_buffer/editing.rs"]
mod editing;
#[path = "text_buffer/grapheme_columns.rs"]
mod grapheme_columns;
#[path = "text_buffer/instrumentation.rs"]
mod instrumentation;
#[path = "text_buffer/search_helpers.rs"]
mod search_helpers;
#[path = "text_buffer/snapshot_history.rs"]
mod snapshot_history;
#[path = "text_buffer/types.rs"]
mod types;
#[path = "text_buffer/undo_search.rs"]
mod undo_search;

use grapheme_columns::*;
#[cfg(all(test, feature = "gui"))]
pub(crate) use instrumentation::{
    from_text_call_count, reset_from_text_call_count, reset_to_text_call_count, to_text_call_count,
};
#[cfg(all(test, feature = "gui"))]
use instrumentation::{FROM_TEXT_CALL_COUNT, TO_TEXT_CALL_COUNT};
use search_helpers::*;
use snapshot_history::buffer_bytes;
pub(crate) use snapshot_history::{pop_history_entry, push_history_entry, push_history_snapshot};
pub(crate) use types::BufferSnapshot;
pub use types::{BufferError, FileSnapshot, TextBuffer};
use types::{CompoundEditState, InsertUndoGroup};
pub(crate) use types::{EditDelta, HistoryEntry};
