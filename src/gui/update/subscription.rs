//! GUI event, shortcut, replacement-editor, and timer subscriptions.

use super::*;

include!("subscription/events.rs");
include!("subscription/file_window_shortcuts.rs");
include!("subscription/search_navigation_shortcuts.rs");
include!("subscription/pane_theme_shortcuts.rs");
include!("subscription/replacement_events.rs");
include!("subscription/timers.rs");

pub(in crate::gui::app::state) fn subscription(state: &KfnotepadGui) -> Subscription<Message> {
    Subscription::batch(gui_subscriptions(state))
}
