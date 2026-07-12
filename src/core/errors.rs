//! Error types shared across file, save, and managed-note operations.

#[path = "errors/managed_notes.rs"]
mod managed_notes;
#[path = "errors/open.rs"]
mod open;
#[path = "errors/save.rs"]
mod save;

pub use managed_notes::ManagedNotesError;
pub use open::OpenError;
pub use save::SaveError;
