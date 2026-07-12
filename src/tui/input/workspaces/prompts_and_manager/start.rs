//! Workspace save/open/delete prompt activation.

use super::*;

pub(crate) fn start_workspace_save_prompt(runtime: &mut EditorRuntime) {
    runtime.workspace_manager = None;
    runtime.workspace_prompt = Some(WorkspacePrompt::SaveNamed);
    runtime.workspace_query.clear();
    load_workspace_prompt_candidates(runtime);
    runtime.workspace_prompt_candidate_index = None;
    runtime.status = if runtime.workspace_prompt_candidates.is_empty() {
        String::from("Save workspace as: ")
    } else {
        String::from("Save workspace as:  (Up/Down picks existing)")
    };
}

pub(crate) fn start_workspace_open_prompt(runtime: &mut EditorRuntime) {
    runtime.workspace_manager = None;
    runtime.workspace_open_confirmation_pending = false;
    runtime.workspace_pending_open = None;
    runtime.workspace_prompt = Some(WorkspacePrompt::OpenNamed);
    load_workspace_prompt_candidates(runtime);
    if runtime.workspace_prompt_candidates.is_empty() {
        runtime.workspace_query.clear();
        runtime.workspace_prompt_candidate_index = None;
        runtime.workspace_prompt = None;
        runtime.status = String::from("No workspace projects saved");
    } else {
        runtime.workspace_prompt_candidate_index = Some(0);
        runtime.workspace_query = runtime.workspace_prompt_candidates[0].clone();
        refresh_workspace_prompt_status(runtime);
    }
}

pub(crate) fn start_workspace_delete_prompt(runtime: &mut EditorRuntime) {
    runtime.workspace_manager = None;
    runtime.workspace_pending_delete = None;
    runtime.workspace_prompt = Some(WorkspacePrompt::DeleteNamed);
    load_workspace_prompt_candidates(runtime);
    if runtime.workspace_prompt_candidates.is_empty() {
        runtime.workspace_query.clear();
        runtime.workspace_prompt_candidate_index = None;
        runtime.workspace_prompt = None;
        runtime.status = String::from("No workspace projects saved");
    } else {
        runtime.workspace_prompt_candidate_index = Some(0);
        runtime.workspace_query = runtime.workspace_prompt_candidates[0].clone();
        refresh_workspace_prompt_status(runtime);
    }
}
