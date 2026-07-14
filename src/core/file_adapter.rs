//! File parsing, persistence, and summary helpers.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::{
    resolve_managed_notes_dir, Command, FileSnapshot, FileSummary, ManagedNotesError, OpenError,
    SaveError, TextBuffer, TextDocument, MAX_TEXT_FILE_BYTES,
};

#[path = "file_adapter/cli_help.rs"]
mod cli_help;
#[path = "file_adapter/managed_notes.rs"]
mod managed_notes;
#[path = "file_adapter/read_snapshot.rs"]
mod read_snapshot;
#[path = "file_adapter/save_impl.rs"]
mod save_impl;
#[path = "file_adapter/summary_open_save.rs"]
mod summary_open_save;
#[path = "file_adapter/trash_atomic_helpers.rs"]
mod trash_atomic_helpers;
#[path = "file_adapter/types.rs"]
mod types;

pub use cli_help::*;
pub use managed_notes::*;
use read_snapshot::{
    file_snapshot, read_text_file, read_text_file_with_snapshot, validate_save_target,
    BoundedFileReadError,
};
pub use read_snapshot::{snapshot_text_file, snapshot_text_file_metadata};
pub use save_impl::save_text_snapshot;
use save_impl::{save_text_buffer_for_document, save_text_buffer_inner};
pub use summary_open_save::*;
pub use trash_atomic_helpers::move_path_to_trash;
use trash_atomic_helpers::{
    fingerprint_bytes, is_managed_note_file_name, temporary_sibling_path, write_temp_then_rename,
};
pub use types::*;
