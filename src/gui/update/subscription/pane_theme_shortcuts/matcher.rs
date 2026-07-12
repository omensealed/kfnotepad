//! Combined pane/theme/reader shortcut matching and character normalization.

use super::*;

pub(super) fn pane_theme_reader_shortcut_message(event: &Event) -> Option<Message> {
    pane_size_shortcut_message(event)
        .or_else(|| theme_reader_shortcut_message(event))
        .or_else(|| move_pane_shortcut_message(event))
}

pub(super) fn shortcut_character_matches(
    key: &Key,
    physical_key: keyboard::key::Physical,
    expected: char,
) -> bool {
    key.to_latin(physical_key)
        .is_some_and(|character| character == expected)
        || matches!(
            key.as_ref(),
            Key::Character(value) if value.eq_ignore_ascii_case(&expected.to_string())
        )
}
