//! Global GUI event routing across shortcut and replacement-editor domains.

use super::*;

pub(super) fn gui_subscription_event_message(
    event: Event,
    status: iced::event::Status,
    window_id: window::Id,
) -> Option<Message> {
    global_event_message(&event)
        .or_else(|| file_window_shortcut_message(&event, window_id))
        .or_else(|| search_navigation_shortcut_message(&event))
        .or_else(|| pane_theme_reader_shortcut_message(&event))
        .or_else(|| replacement_editor_event_message(event, status))
}

fn global_event_message(event: &Event) -> Option<Message> {
    match event {
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
            Some(Message::ReplacementEditorGlobalPointerReleased)
        }
        Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
            if modifiers.is_empty() && matches!(key.as_ref(), Key::Named(Named::Insert)) =>
        {
            Some(Message::ToggleReplacementOverwriteMode)
        }
        _ => None,
    }
}
