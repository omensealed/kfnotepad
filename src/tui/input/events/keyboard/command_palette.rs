//! Command-palette keyboard handling.

use super::*;

pub(crate) fn handle_command_palette_key_event(
    workspace: &mut EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
    event: KeyEvent,
) -> bool {
    match event.code {
        KeyCode::Esc => {
            runtime.command_palette = None;
            runtime.status = String::from("Command palette closed");
        }
        KeyCode::Up => move_command_palette_selection(runtime, -1),
        KeyCode::Down => move_command_palette_selection(runtime, 1),
        KeyCode::PageUp => move_command_palette_selection(runtime, -5),
        KeyCode::PageDown => move_command_palette_selection(runtime, 5),
        KeyCode::Home => set_command_palette_selection(runtime, 0),
        KeyCode::End => {
            let last = runtime
                .command_palette
                .as_ref()
                .map(|palette| {
                    command_palette_candidates(&palette.query)
                        .len()
                        .saturating_sub(1)
                })
                .unwrap_or(0);
            set_command_palette_selection(runtime, last);
        }
        KeyCode::Backspace => {
            if let Some(palette) = runtime.command_palette.as_mut() {
                palette.query.pop();
            }
            normalize_command_palette_selection(runtime);
        }
        KeyCode::Enter => {
            let command = selected_command_palette_entry(runtime).map(|entry| entry.command);
            runtime.command_palette = None;
            if let Some(command) = command {
                return run_workspace_menu_command(command, workspace, runtime);
            }
            runtime.status = String::from("No matching command");
        }
        KeyCode::Char(value)
            if event.modifiers.is_empty() || event.modifiers == KeyModifiers::SHIFT =>
        {
            if let Some(palette) = runtime.command_palette.as_mut() {
                palette.query.push(value);
            }
            normalize_command_palette_selection(runtime);
        }
        _ => {}
    }
    false
}
