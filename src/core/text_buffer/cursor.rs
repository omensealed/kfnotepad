//! Cursor movement, validation, and word-deletion behavior.

use super::*;

#[path = "cursor/delete_word.rs"]
mod delete_word;
#[path = "cursor/movement.rs"]
mod movement;
#[path = "cursor/validation.rs"]
mod validation;
#[path = "cursor/word_movement.rs"]
mod word_movement;
