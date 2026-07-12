//! Subscription batch assembly and conditional reader/drag timers.

use super::*;

pub(super) fn gui_subscriptions(state: &KfnotepadGui) -> Vec<Subscription<Message>> {
    let replacement_drag_active =
        state.replacement_drag.is_some() || state.replacement_scrollbar_drag.is_some();
    let mut subscriptions = vec![
        event::listen_with(gui_subscription_event_message),
        window::close_requests().map(Message::WindowCloseRequested),
        iced::time::every(Duration::from_secs(1)).map(|_| Message::ExternalFileCheckTick),
    ];
    if state.settings.gui_reader_mode_enabled {
        subscriptions.push(
            iced::time::every(Duration::from_millis(GUI_READER_TICK_MS))
                .map(|_| Message::ReaderScrollTick),
        );
    }
    if replacement_drag_active {
        subscriptions.push(
            iced::time::every(Duration::from_millis(GUI_REPLACEMENT_DRAG_TICK_MS))
                .map(|_| Message::ReplacementEditorDragTick),
        );
    }
    subscriptions
}
