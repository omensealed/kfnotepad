//! Workspace, command-palette, and file-sidebar overlay rendering.

use std::io::{self, Write};

use kfnotepad::TabStripItem;

use super::layout::PreparedFrameLayout;
use crate::tui::app::SIDEBAR_WIDTH;
use crate::tui::input::EditorRuntime;
use crate::tui::render::tab_strip_height_for_width;
use crate::tui::sidebar;
use crate::tui::theme::EditorTheme;

pub(super) fn write_runtime_overlays<W: Write>(
    stdout: &mut W,
    runtime: &EditorRuntime,
    tab_items: &[TabStripItem],
    layout: PreparedFrameLayout,
) -> io::Result<()> {
    let sidebar_width = runtime.sidebar.as_ref().map_or(0, |_| SIDEBAR_WIDTH);
    if let Some(manager) = &runtime.workspace_manager {
        sidebar::write_workspace_manager_overlay(
            stdout,
            manager,
            layout.visible_rows,
            sidebar_width,
            layout.terminal_width,
            tab_strip_height_for_width(tab_items, layout.editor_width),
            EditorTheme::for_id(runtime.settings.theme_id),
            layout.no_color,
        )?;
    }
    if let Some(palette) = &runtime.command_palette {
        sidebar::write_command_palette_overlay(
            stdout,
            palette,
            layout.visible_rows,
            sidebar_width,
            layout.terminal_width,
            tab_strip_height_for_width(tab_items, layout.editor_width),
            EditorTheme::for_id(runtime.settings.theme_id),
            layout.no_color,
        )?;
    }
    if let Some(sidebar) = &runtime.sidebar {
        sidebar::render_file_sidebar(
            stdout,
            sidebar,
            layout.visible_rows,
            EditorTheme::for_id(runtime.settings.theme_id),
            layout.no_color,
        )?;
    }
    Ok(())
}
