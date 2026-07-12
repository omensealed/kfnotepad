//! Message routing and focused state-transition handlers.

use super::*;

#[path = "dispatch/browser.rs"]
mod browser;
#[path = "dispatch/editor.rs"]
mod editor;
#[path = "dispatch/files.rs"]
mod files;
#[path = "dispatch/panes.rs"]
mod panes;
#[path = "dispatch/preferences.rs"]
mod preferences;
#[path = "dispatch/router.rs"]
mod router;
#[path = "dispatch/search.rs"]
mod search;
#[path = "dispatch/workspaces.rs"]
mod workspaces;

use browser::*;
use editor::*;
use files::*;
use panes::*;
use preferences::*;
use search::*;
use workspaces::*;

pub(in crate::gui::app::state) fn update(
    state: &mut KfnotepadGui,
    message: Message,
) -> Task<Message> {
    router::update(state, message)
}
