//! Workspace prompt keyboard handling.

use super::*;

pub(crate) fn handle_workspace_prompt_key_event(
    workspace: &mut EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
    event: KeyEvent,
) {
    match event.code {
        KeyCode::Esc => {
            runtime.workspace_prompt = None;
            runtime.workspace_query.clear();
            runtime.workspace_pending_open = None;
            runtime.workspace_pending_delete = None;
            runtime.workspace_prompt_candidates.clear();
            runtime.workspace_prompt_candidate_index = None;
            runtime.workspace_open_confirmation_pending = false;
            runtime.status = String::from("Workspace prompt cancelled");
        }
        KeyCode::Up => {
            select_workspace_prompt_candidate(runtime, -1);
        }
        KeyCode::Down => {
            select_workspace_prompt_candidate(runtime, 1);
        }
        KeyCode::Backspace => {
            runtime.workspace_query.pop();
            runtime.workspace_prompt_candidate_index = None;
            refresh_workspace_prompt_status(runtime);
        }
        KeyCode::Enter => {
            apply_workspace_prompt(workspace, runtime);
        }
        KeyCode::Char(value)
            if event.modifiers.is_empty() || event.modifiers == KeyModifiers::SHIFT =>
        {
            runtime.workspace_query.push(value);
            runtime.workspace_prompt_candidate_index = None;
            refresh_workspace_prompt_status(runtime);
        }
        _ => {}
    }
}
