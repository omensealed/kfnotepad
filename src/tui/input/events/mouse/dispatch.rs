//! Mouse event routing for sidebar, editor, tabs, menus, and workspaces.

use super::*;

mod editor_scroll;
mod frame;
mod left_click;
mod sidebar;
#[cfg(test)]
mod test_adapter;
mod workspace;

use editor_scroll::handle_editor_scroll_mouse_event;
use frame::mouse_render_frame;
use left_click::handle_workspace_left_click;
use sidebar::handle_sidebar_mouse_event;
#[cfg(test)]
pub(crate) use test_adapter::*;
pub(crate) use workspace::*;
