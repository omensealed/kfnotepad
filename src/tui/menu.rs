//! TUI menu definitions, navigation, command palette, and workspace manager state.

#[path = "menu/command_palette.rs"]
mod command_palette;
#[path = "menu/commands.rs"]
mod commands;
#[path = "menu/group_items.rs"]
mod group_items;
#[path = "menu/group_navigation.rs"]
mod group_navigation;
#[path = "menu/types.rs"]
mod types;
#[path = "menu/workspace_manager.rs"]
mod workspace_manager;

pub(crate) use command_palette::{CommandPaletteEntry, CommandPaletteState};
pub(crate) use commands::{MenuCommand, MenuItem};
pub(crate) use types::{MenuGroup, MenuState, MENU_GROUPS};
pub(crate) use workspace_manager::{WorkspaceManagerEntry, WorkspaceManagerState};
