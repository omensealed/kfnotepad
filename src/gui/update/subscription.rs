//! GUI event, shortcut, replacement-editor, and timer subscriptions.

use super::*;

#[path = "subscription/events.rs"]
mod events;
#[path = "subscription/file_window_shortcuts.rs"]
mod file_window_shortcuts;
#[path = "subscription/pane_theme_shortcuts.rs"]
mod pane_theme_shortcuts;
#[path = "subscription/replacement_events.rs"]
mod replacement_events;
#[path = "subscription/search_navigation_shortcuts.rs"]
mod search_navigation_shortcuts;
#[path = "subscription/timers.rs"]
mod timers;

use events::gui_subscription_event_message;
use file_window_shortcuts::file_window_shortcut_message;
use pane_theme_shortcuts::pane_theme_reader_shortcut_message;
use replacement_events::replacement_editor_event_message;
use search_navigation_shortcuts::search_navigation_shortcut_message;
use timers::gui_subscriptions;

pub(in crate::gui::app::state) fn subscription(state: &KfnotepadGui) -> Subscription<Message> {
    Subscription::batch(gui_subscriptions(state))
}
