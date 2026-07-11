use std::time::{Instant, SystemTime};

use super::{
    expand_range_to_grapheme_boundaries, find_case_insensitive_range,
    find_last_case_insensitive_range, Cursor, CursorMove, SearchMode, MAX_UNDO_BYTES,
    MAX_UNDO_HISTORY, TYPING_UNDO_COALESCE_WINDOW,
};

include!("text_buffer/instrumentation.rs");
include!("text_buffer/types.rs");
include!("text_buffer/grapheme_columns.rs");
include!("text_buffer/constructors.rs");
include!("text_buffer/cursor.rs");
include!("text_buffer/editing.rs");
include!("text_buffer/undo_search.rs");
include!("text_buffer/snapshot_history.rs");
include!("text_buffer/search_helpers.rs");
