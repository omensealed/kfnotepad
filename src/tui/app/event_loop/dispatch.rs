//! Keyboard, paste, mouse, resize, and modal input dispatch.

use crossterm::event::{Event, KeyEvent};
use kfnotepad::EditorWorkspace;

use super::LoopLayout;
use crate::tui::app::SIDEBAR_WIDTH;
use crate::tui::input::{
    handle_command_palette_key_event, handle_key_event, handle_workspace_key_event,
    handle_workspace_manager_key_event, handle_workspace_menu_key_event,
    handle_workspace_mouse_event, handle_workspace_prompt_key_event,
    handle_workspace_sidebar_key_event, insert_paste, EditorRuntime, InputResult, MouseContext,
};
use crate::tui::render::tab_strip_height_for_width;

pub(super) fn handle_terminal_event(
    event: Event,
    workspace: &mut EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
    layout: &LoopLayout,
) -> InputResult {
    match event {
        Event::Key(event) => handle_terminal_key_event(workspace, runtime, event),
        Event::Paste(text) => {
            let active_tab = workspace.active_tab_mut();
            if insert_paste(
                active_tab.document.as_mut(),
                &mut active_tab.state.cursor,
                runtime,
                &text,
            ) {
                InputResult::Quit
            } else {
                InputResult::Handled
            }
        }
        Event::Mouse(event) => {
            let sidebar_width = runtime.sidebar.as_ref().map_or(0, |_| SIDEBAR_WIDTH);
            let editor_width = layout.terminal_width.saturating_sub(sidebar_width).max(1);
            let body_top = tab_strip_height_for_width(&workspace.tab_strip_items(), editor_width);
            let viewport_start = workspace.active_tab().state.viewport_start;
            let horizontal_offset = workspace.active_tab().state.horizontal_offset;
            handle_workspace_mouse_event(
                workspace,
                runtime,
                event,
                MouseContext {
                    viewport_start,
                    horizontal_offset,
                    visible_rows: layout.visible_rows,
                    gutter_width: layout.gutter_width,
                    terminal_width: layout.terminal_width,
                    sidebar_width,
                    body_top,
                },
            )
        }
        Event::Resize(_, _) => InputResult::Handled,
        _ => InputResult::Ignored,
    }
}

fn handle_terminal_key_event(
    workspace: &mut EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
    event: KeyEvent,
) -> InputResult {
    if handle_workspace_key_event(workspace, runtime, event) {
        return InputResult::Handled;
    }
    if runtime.menu.is_some() {
        return if handle_workspace_menu_key_event(workspace, runtime, event) {
            InputResult::Quit
        } else {
            InputResult::Handled
        };
    }
    if runtime.command_palette.is_some() {
        return if handle_command_palette_key_event(workspace, runtime, event) {
            InputResult::Quit
        } else {
            InputResult::Handled
        };
    }
    if runtime.workspace_prompt.is_some() {
        handle_workspace_prompt_key_event(workspace, runtime, event);
        return InputResult::Handled;
    }
    if runtime.workspace_manager.is_some() {
        handle_workspace_manager_key_event(workspace, runtime, event);
        return InputResult::Handled;
    }
    if runtime.sidebar.is_some() {
        handle_workspace_sidebar_key_event(workspace, runtime, event);
        return InputResult::Handled;
    }
    let active_tab = workspace.active_tab_mut();
    if handle_key_event(
        active_tab.document.as_mut(),
        &mut active_tab.state.cursor,
        runtime,
        event,
    ) {
        InputResult::Quit
    } else {
        InputResult::Handled
    }
}
