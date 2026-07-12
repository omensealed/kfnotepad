//! Character, word, line, and range deletion behavior.

use super::*;

#[path = "delete/char_backspace.rs"]
mod char_backspace;
#[path = "delete/range.rs"]
mod range;
#[path = "delete/word_line.rs"]
mod word_line;
