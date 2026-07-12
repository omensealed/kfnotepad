//! Ordered routing across GUI message domains.

use super::*;

#[path = "router/browser_files.rs"]
mod browser_files;
#[path = "router/misc.rs"]
mod misc;
#[path = "router/panes.rs"]
mod panes;
#[path = "router/replacement.rs"]
mod replacement;
#[path = "router/search_editor.rs"]
mod search_editor;
#[path = "router/types.rs"]
mod types;
#[path = "router/workspace_preferences.rs"]
mod workspace_preferences;

use browser_files::dispatch_browser_and_files;
use misc::dispatch_miscellaneous;
use panes::dispatch_panes;
use replacement::dispatch_replacement_editor;
use search_editor::dispatch_search_and_editor;
use types::{handled_none, GuiDispatchResult};
use workspace_preferences::dispatch_workspace_and_preferences;

pub(super) fn update(state: &mut KfnotepadGui, message: Message) -> Task<Message> {
    let message = match dispatch_browser_and_files(state, message) {
        GuiDispatchResult::Handled(task) => return task,
        GuiDispatchResult::Unhandled(message) => message,
    };
    let message = match dispatch_workspace_and_preferences(state, message) {
        GuiDispatchResult::Handled(task) => return task,
        GuiDispatchResult::Unhandled(message) => message,
    };
    let message = match dispatch_panes(state, message) {
        GuiDispatchResult::Handled(task) => return task,
        GuiDispatchResult::Unhandled(message) => message,
    };
    let message = match dispatch_search_and_editor(state, message) {
        GuiDispatchResult::Handled(task) => return task,
        GuiDispatchResult::Unhandled(message) => message,
    };
    let message = match dispatch_replacement_editor(state, message) {
        GuiDispatchResult::Handled(task) => return task,
        GuiDispatchResult::Unhandled(message) => message,
    };
    match dispatch_miscellaneous(state, message) {
        GuiDispatchResult::Handled(task) => task,
        GuiDispatchResult::Unhandled(_) => Task::none(),
    }
}
