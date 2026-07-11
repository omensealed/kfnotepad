//! File parsing, persistence, and summary helpers.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::{
    resolve_managed_notes_dir, Command, FileSnapshot, FileSummary, ManagedNotesError, OpenError,
    SaveError, TextBuffer, TextDocument, MAX_TEXT_FILE_BYTES,
};

include!("file_adapter/types.rs");
include!("file_adapter/cli_help.rs");
include!("file_adapter/summary_open_save.rs");
include!("file_adapter/managed_notes.rs");
include!("file_adapter/save_impl.rs");
include!("file_adapter/read_snapshot.rs");
include!("file_adapter/trash_atomic_helpers.rs");
