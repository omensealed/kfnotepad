//! Prompt submission dispatch.

use super::*;

pub(crate) fn apply_sidebar_prompt(
    workspace: &mut EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
) {
    let Some(prompt) = runtime.sidebar_prompt.clone() else {
        return;
    };

    match prompt {
        SidebarPrompt::CreateFile => create_sidebar_file(runtime),
        SidebarPrompt::CreateDirectory => create_sidebar_directory(runtime),
        SidebarPrompt::DeleteConfirm { entry, recursive } => {
            delete_sidebar_entry(workspace, runtime, &entry, recursive);
        }
    }
}
