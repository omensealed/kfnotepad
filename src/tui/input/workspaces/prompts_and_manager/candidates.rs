//! Workspace prompt candidate loading, status, and selection.

use super::*;

pub(crate) fn refresh_workspace_prompt_status(runtime: &mut EditorRuntime) {
    runtime.status = match runtime.workspace_prompt.as_ref() {
        Some(WorkspacePrompt::SaveNamed) => {
            format!("Save workspace as: {}", runtime.workspace_query)
        }
        Some(WorkspacePrompt::OpenNamed) => {
            format!("Open workspace: {}", runtime.workspace_query)
        }
        Some(WorkspacePrompt::DeleteNamed) => {
            format!("Delete workspace: {}", runtime.workspace_query)
        }
        Some(WorkspacePrompt::ConfirmOpen) => {
            format!(
                "Replace dirty workspace? type yes: {}",
                runtime.workspace_query
            )
        }
        Some(WorkspacePrompt::ConfirmDelete) => {
            format!(
                "Delete workspace project? type yes: {}",
                runtime.workspace_query
            )
        }
        None => runtime.status.clone(),
    };
}

pub(crate) fn load_workspace_prompt_candidates(runtime: &mut EditorRuntime) {
    runtime.workspace_prompt_candidates.clear();
    runtime.workspace_prompt_candidate_index = None;
    let Some(projects_dir) = runtime.workspace_projects_dir.as_deref() else {
        return;
    };
    if let Ok(projects) = list_gui_workspace_projects(projects_dir) {
        runtime.workspace_prompt_candidates = projects
            .into_iter()
            .map(|entry| entry.project.name)
            .collect::<Vec<_>>();
    }
}

pub(crate) fn select_workspace_prompt_candidate(runtime: &mut EditorRuntime, delta: isize) {
    if runtime.workspace_prompt_candidates.is_empty() {
        runtime.status = String::from("No workspace projects saved");
        return;
    }

    let len = runtime.workspace_prompt_candidates.len();
    let current = runtime.workspace_prompt_candidate_index.unwrap_or(0);
    let next = if delta.is_negative() {
        current
            .checked_sub(delta.unsigned_abs())
            .unwrap_or(len.saturating_sub(1))
    } else {
        (current + delta as usize) % len
    };
    runtime.workspace_prompt_candidate_index = Some(next);
    runtime.workspace_query = runtime.workspace_prompt_candidates[next].clone();
    refresh_workspace_prompt_status(runtime);
}
