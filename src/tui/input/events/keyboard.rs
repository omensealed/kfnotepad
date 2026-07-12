//! Keyboard dispatch for editor, workspace, palette, menus, and sidebar.

use super::*;

mod command_palette;
mod editor_dispatch;
mod menu;
mod sidebar;
mod workspace_menu;
mod workspace_shortcuts;

pub(crate) use command_palette::*;
pub(crate) use editor_dispatch::*;
pub(crate) use menu::*;
pub(crate) use sidebar::*;
pub(crate) use workspace_menu::*;
pub(crate) use workspace_shortcuts::*;
