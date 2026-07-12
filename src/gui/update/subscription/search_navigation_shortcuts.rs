//! Search, help, clipboard, and document navigation shortcuts.

use super::*;

pub(super) fn search_navigation_shortcut_message(event: &Event) -> Option<Message> {
    match event {
        Event::Keyboard(keyboard::Event::KeyPressed {
            key,
            physical_key,
            modifiers,
            ..
        }) if modifiers.control()
            && key
                .to_latin(*physical_key)
                .is_some_and(|character| character == 'f') =>
        {
            Some(Message::SearchNext)
        }
        Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
            if modifiers.control() && matches!(key.as_ref(), Key::Character("f" | "F")) =>
        {
            Some(Message::SearchNext)
        }
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: Key::Named(Named::F1),
            ..
        }) => Some(Message::MenuCommand(GuiMenuCommand::OpenHelp)),
        Event::Keyboard(event) if gui_editor_clipboard_shortcut_command(event).is_some() => {
            gui_editor_clipboard_shortcut_command(event).map(Message::MenuCommand)
        }
        Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
            if matches!(key.as_ref(), Key::Named(Named::F3)) && modifiers.shift() =>
        {
            Some(Message::SearchPrevious)
        }
        Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
            if matches!(key.as_ref(), Key::Named(Named::F3)) && modifiers.is_empty() =>
        {
            Some(Message::SearchNext)
        }
        Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
            if modifiers.control() && matches!(key.as_ref(), Key::Named(Named::Home)) =>
        {
            Some(Message::GoDocumentStart)
        }
        Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
            if modifiers.control() && matches!(key.as_ref(), Key::Named(Named::End)) =>
        {
            Some(Message::GoDocumentEnd)
        }
        Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
            if modifiers.control() && matches!(key.as_ref(), Key::Named(Named::PageUp)) =>
        {
            Some(Message::ScrollActiveEditorViewport(
                -(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES as i32),
            ))
        }
        Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
            if modifiers.control() && matches!(key.as_ref(), Key::Named(Named::PageDown)) =>
        {
            Some(Message::ScrollActiveEditorViewport(
                GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES as i32,
            ))
        }
        Event::Keyboard(keyboard::Event::KeyPressed {
            key,
            physical_key,
            modifiers,
            ..
        }) if modifiers.control()
            && key
                .to_latin(*physical_key)
                .is_some_and(|character| character == 'g') =>
        {
            Some(Message::GoToLineRequested)
        }
        Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
            if modifiers.control() && matches!(key.as_ref(), Key::Character("g" | "G")) =>
        {
            Some(Message::GoToLineRequested)
        }
        _ => None,
    }
}
