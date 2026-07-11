//! Error types shared across file and document operations.

use std::fmt;
use std::io;
use std::path::PathBuf;

include!("errors/open.rs");
include!("errors/save.rs");
include!("errors/managed_notes.rs");
