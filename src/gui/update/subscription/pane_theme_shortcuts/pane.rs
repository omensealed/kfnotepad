//! Pane minimize and maximize shortcuts.

use super::*;

pub(super) fn pane_size_shortcut_message(event: &Event) -> Option<Message> {
    let Event::Keyboard(keyboard::Event::KeyPressed {
        key,
        physical_key,
        modifiers,
        ..
    }) = event
    else {
        return None;
    };

    if modifiers.control()
        && modifiers.shift()
        && shortcut_character_matches(key, *physical_key, 'm')
    {
        return Some(Message::ToggleActiveMaximize);
    }
    if modifiers.control()
        && !modifiers.shift()
        && shortcut_character_matches(key, *physical_key, 'm')
    {
        return Some(Message::ToggleActiveMinimize);
    }
    None
}
